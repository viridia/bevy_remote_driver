"""Runs plans: for each, builds the target, launches it, runs the steps, closes it and reports.

    bevy-remote-driver <plan.json | plan.py | directory>...

A directory stands for every `.py` and `.json` file in it. Plans run one after another; a failed
plan does not stop the rest, and a summary follows the last report.

The workspace built is the one that holds the plan, from any working directory; a plan outside
any Cargo workspace builds the working directory's. A JSON plan names its `bin` or its `example`,
optionally its `package`, and its `steps`. A Python plan sets `BIN` or `EXAMPLE`, optionally
`PACKAGE`, and defines `run(driver)`.
"""

import importlib.util
import json
import os
import shutil
import socket
import subprocess
import sys
import time
import traceback
import urllib.error
from pathlib import Path

from .driver import BrpError, Driver, StepFailed

STARTUP_SECONDS = 60
EXIT_SECONDS = 3


class PlanError(Exception):
    pass


def cargo_dir(plan_path):
    """Where cargo runs: the plan's directory, if a `Cargo.toml` is at or above it, otherwise the
    working directory."""
    start = plan_path.resolve().parent
    if any((folder / "Cargo.toml").is_file() for folder in (start, *start.parents)):
        return start
    return Path.cwd()


def target_dir(cwd):
    """Cargo's target directory for the workspace at `cwd`, so `CARGO_TARGET_DIR` is respected."""
    result = subprocess.run(
        ["cargo", "metadata", "--format-version=1", "--no-deps"],
        check=False,
        stdout=subprocess.PIPE,
        text=True,
        cwd=cwd,
    )
    if result.returncode != 0:
        raise PlanError(f"`cargo metadata` failed in {cwd}")
    return Path(json.loads(result.stdout)["target_directory"])


def target_of(bin_name, example):
    """The plan's target as (kind, name): a binary or an example, exactly one."""
    if (bin_name is None) == (example is None):
        raise PlanError("a plan names exactly one of `bin` and `example`")
    return ("bin", bin_name) if bin_name is not None else ("example", example)


def load(plan_path):
    """The plan's target, package, features, and a function running its steps on a driver."""
    if not plan_path.is_file():
        raise PlanError(f"no plan at {plan_path}")
    if plan_path.suffix == ".py":
        spec = importlib.util.spec_from_file_location(plan_path.stem, plan_path)
        module = importlib.util.module_from_spec(spec)
        try:
            spec.loader.exec_module(module)
        except Exception as error:
            # Reported as the plan's alone, so the plans after it still run.
            raise PlanError(f"the plan did not load: {error!r}") from error
        return (
            target_of(getattr(module, "BIN", None), getattr(module, "EXAMPLE", None)),
            getattr(module, "PACKAGE", None),
            getattr(module, "FEATURES", None),
            module.run,
        )
    plan = json.loads(plan_path.read_text())
    steps = plan["steps"]

    def run(driver):
        for index, entry in enumerate(steps):
            names = [key for key in entry if getattr(getattr(Driver, key, None), "is_step", False)]
            if len(names) != 1:
                raise PlanError(f"step {index + 1} names {len(names)} steps: {json.dumps(entry)}")
            name = names[0]
            options = {key: value for key, value in entry.items() if key != name}
            getattr(driver, name)(entry[name], **options)

    target = target_of(plan.get("bin"), plan.get("example"))
    return target, plan.get("package"), plan.get("features"), run


def build(target, package, features, cwd):
    """Builds the target in the workspace at `cwd`, and returns its binary and its package's
    directory."""
    kind, name = target
    command = ["cargo", "build", f"--{kind}", name, "--message-format=json-render-diagnostics"]
    if package:
        command += ["-p", package]
    # An example declaring `required-features` is skipped rather than refused without them, so
    # leaving these out ends at "the build produced no example named …" below.
    if features:
        command += ["--features", ",".join(features)]
    result = subprocess.run(command, check=False, stdout=subprocess.PIPE, text=True, cwd=cwd)
    if result.returncode != 0:
        raise PlanError(f"`{' '.join(command)}` failed in {cwd}")
    for line in result.stdout.splitlines():
        message = json.loads(line)
        built = message.get("target", {})
        if (
            message.get("reason") == "compiler-artifact"
            and built.get("name") == name
            and kind in built.get("kind", [])
        ):
            return Path(message["executable"]), Path(message["manifest_path"]).parent
    raise PlanError(f"the build produced no {kind} named {name}")


def free_port():
    with socket.socket() as probe:
        probe.bind(("127.0.0.1", 0))
        return probe.getsockname()[1]


def environment(binary, package_dir, port, settings_dir):
    env = dict(os.environ)
    env["BEVY_REMOTE_DRIVER_PORT"] = str(port)
    # Where Bevy looks for `assets/`, as under `cargo run`.
    env["CARGO_MANIFEST_DIR"] = str(package_dir)
    # Bevy's `preferences_dir` honours it only when absolute, which a path under Cargo's target
    # directory is (DR6.5).
    env["BEVY_SETTINGS_DIR"] = str(settings_dir)
    if sys.platform == "darwin":
        # `dynamic_linking` leaves the binary without an rpath to the toolchain's libstd, or to the
        # Bevy dylib next to it.
        libdir = subprocess.run(
            ["rustc", "--print", "target-libdir"], stdout=subprocess.PIPE, text=True, check=True
        ).stdout.strip()
        paths = [libdir, str(binary.parent.parent / "deps"), env.get("DYLD_FALLBACK_LIBRARY_PATH")]
        env["DYLD_FALLBACK_LIBRARY_PATH"] = ":".join(p for p in paths if p)
    return env


class App:
    """The plan's app: launched, launched again by the `restart` step, and closed. Every launch has
    the same port, environment and settings directory, and writes to the same log."""

    def __init__(self, binary, package_dir, env, log, log_path, port):
        self.binary = binary
        self.package_dir = package_dir
        self.env = env
        self.log = log
        self.log_path = log_path
        self.port = port
        self.process = None

    def launch(self):
        self.process = subprocess.Popen(
            [self.binary],
            cwd=self.package_dir,
            env=self.env,
            stdout=self.log,
            stderr=subprocess.STDOUT,
        )

    def connect(self, attach):
        """Calls `attach` until the app answers, and returns what it returns."""
        deadline = time.monotonic() + STARTUP_SECONDS
        while time.monotonic() < deadline:
            if self.process.poll() is not None:
                raise PlanError(
                    f"the app exited during startup with code {self.process.returncode}; "
                    f"see {self.log_path}"
                )
            try:
                return attach()
            except (urllib.error.URLError, ConnectionError):
                time.sleep(0.1)
        raise PlanError(
            f"the app did not answer on port {self.port} in {STARTUP_SECONDS} s; "
            f"see {self.log_path}"
        )

    def restart(self, attach, seconds):
        """For the `restart` step, once the app has been asked to exit: waits up to `seconds` for
        it to close, launches it again, and calls `attach` once it answers."""
        try:
            self.process.wait(seconds)
        except subprocess.TimeoutExpired:
            raise StepFailed(f"the app to exit within {seconds} s", "it still running") from None
        self.log.write(f"\n--- restarted (exit code {self.process.returncode}) ---\n\n".encode())
        self.log.flush()
        self.launch()
        try:
            self.connect(attach)
        except PlanError as error:
            raise StepFailed("the app to start again", str(error)) from None

    def close(self, driver):
        if self.process.poll() is None and driver is not None and not driver.exited:
            try:
                driver.exit()
                self.process.wait(EXIT_SECONDS)
            except (OSError, BrpError, subprocess.TimeoutExpired):
                pass
        if self.process.poll() is None:
            self.process.kill()
            self.process.wait()


def failed_step(driver, plan_path, error):
    """Which step failed: its number and text, and for a Python plan, the line that called it."""
    lines = [f"step {len(driver.steps)}: {driver.steps[-1]}" if driver.steps else "before any step"]
    if error is not None and plan_path.suffix == ".py":
        for frame in reversed(traceback.extract_tb(error.__traceback__)):
            if Path(frame.filename).resolve() == plan_path.resolve():
                lines.append(f"at {plan_path}:{frame.lineno}")
                break
    return lines


def plans_in(paths):
    """The plans named by `paths`, a directory standing for its own `.py` and `.json` files, in name
    order."""
    plans = []
    for path in map(Path, paths):
        if path.is_dir():
            plans += sorted(p for p in path.iterdir() if p.suffix in (".py", ".json"))
        else:
            plans.append(path)
    return plans


def main(argv=None):
    """The `bevy-remote-driver` command: runs the plans `argv` names, and returns the exit
    status."""
    plans = plans_in(sys.argv[1:] if argv is None else argv)
    if not plans:
        print(__doc__.strip(), file=sys.stderr)
        return 2
    results = []
    for plan_path in plans:
        status = run_plan(plan_path)
        results.append((status, plan_path))
        if status == 130:
            break
    passed = sum(status == 0 for status, _ in results)
    if len(plans) > 1:
        words = {0: "passed", 1: "FAILED", 2: "could not run", 130: "interrupted"}
        print(f"\n{passed} of {len(plans)} plans passed")
        for status, plan_path in results:
            print(f"{words[status]}: {plan_path}")
    announce(passed, len(plans))
    return max(status for status, _ in results)


def announce(passed, total):
    """Says the run is over, once, after the last app has closed, so a developer working
    elsewhere knows that no more windows will come forward: on macOS a sound, Glass for a pass and
    Basso otherwise, and a notification; elsewhere the terminal bell."""
    if sys.platform != "darwin":
        print("\a", end="", flush=True)
        return
    sound = "Glass" if passed == total else "Basso"
    subprocess.Popen(["afplay", f"/System/Library/Sounds/{sound}.aiff"])
    message = f"{passed} of {total} plans passed"
    subprocess.Popen(
        ["osascript", "-e", f'display notification "{message}" with title "Remote driver run over"']
    )


def run_plan(plan_path):
    """Runs one plan, prints its report and returns its exit status."""
    cwd = cargo_dir(plan_path)
    try:
        target, package, features, run = load(plan_path)
        out_dir = target_dir(cwd) / "remote-driver" / target[1] / plan_path.stem
    except PlanError as error:
        print(f"could not run: {error}", file=sys.stderr)
        return 2
    shutil.rmtree(out_dir, ignore_errors=True)
    out_dir.mkdir(parents=True)
    log_path = out_dir / "app.log"
    settings_dir = out_dir / "settings"

    report = [f"plan: {plan_path}"]
    status = 0
    driver = None
    try:
        binary, package_dir = build(target, package, features, cwd)
        port = free_port()
        with open(log_path, "wb") as log:
            env = environment(binary, package_dir, port, settings_dir)
            app = App(binary, package_dir, env, log, log_path, port)
            app.launch()
            try:
                driver = app.connect(lambda: Driver(port, out_dir, app))
                run(driver)
                report.append(f"passed: {len(driver.steps)} steps")
            except (StepFailed, BrpError, OSError) as error:
                status = 1
                report.append("FAILED")
                report += failed_step(driver, plan_path, error) if driver else []
                if isinstance(error, StepFailed):
                    report += [f"expected: {error.expected}", f"found: {error.found}"]
                else:
                    report.append(f"error: {error}")
                if app.process.poll() is not None:
                    report.append(f"the app had exited, with code {app.process.returncode}")
            except KeyboardInterrupt:
                status = 130
                report.append("interrupted")
                report += failed_step(driver, plan_path, None)[:1] if driver else []
            finally:
                app.close(driver)
    except PlanError as error:
        status = 2
        report.append(f"could not run: {error}")

    report.append(f"log: {log_path}")
    for path in driver.screenshots if driver else []:
        report.append(f"screenshot: {path}")
    text = "\n".join(report) + "\n"
    (out_dir / "report.txt").write_text(text)
    # Flushed, so each report comes out after its own build's output, which goes to stderr.
    print(text, end="", flush=True)
    return status


if __name__ == "__main__":
    sys.exit(main())
