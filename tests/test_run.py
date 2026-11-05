import json
import tempfile
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

from bevy_remote_driver import Driver
from bevy_remote_driver.run import (
    PlanError,
    chosen,
    duration,
    environment,
    failed_step,
    load,
    plans_in,
)


class Recorder:
    """Stands in for a `Driver`, recording the steps a plan calls."""

    def __init__(self):
        self.calls = []

    def __getattr__(self, name):
        return lambda *args, **kwargs: self.calls.append((name, args, kwargs))


class Loading(unittest.TestCase):
    def setUp(self):
        self.folder = tempfile.TemporaryDirectory()
        self.dir = Path(self.folder.name)

    def tearDown(self):
        self.folder.cleanup()

    def write(self, name, text):
        path = self.dir / name
        path.write_text(text)
        return path

    def test_a_json_plan_runs_its_steps(self):
        steps = [{"click": "Menu/Start", "hold": 3}, {"absent": "Panel"}]
        plan = self.write("plan.json", json.dumps({"bin": "app", "steps": steps}))
        target, package, features, run, driver_class, tags = load(plan)
        self.assertEqual((target, package, features), (("bin", "app"), None, None))
        self.assertIs(driver_class, Driver)
        self.assertEqual(tags, [])
        driver = Recorder()
        run(driver)
        self.assertEqual(
            driver.calls,
            [("click", ("Menu/Start",), {"hold": 3}), ("absent", ("Panel",), {})],
        )

    def test_a_json_step_names_exactly_one_step(self):
        steps = [{"click": "A", "absent": "B"}]
        plan = self.write("plan.json", json.dumps({"example": "app", "steps": steps}))
        _, _, _, run, _, _ = load(plan)
        with self.assertRaisesRegex(PlanError, "names 2 steps"):
            run(Recorder())

    def test_a_python_plan_names_its_target(self):
        plan = self.write(
            "plan.py",
            'EXAMPLE = "app"\nPACKAGE = "pkg"\nFEATURES = ["f"]\nTAGS = ["slow"]\n'
            'def run(driver):\n    driver.present("Menu")\n',
        )
        target, package, features, run, driver_class, tags = load(plan)
        self.assertEqual((target, package, features), (("example", "app"), "pkg", ["f"]))
        self.assertIs(driver_class, Driver)
        self.assertEqual(tags, ["slow"])
        driver = Recorder()
        run(driver)
        self.assertEqual(driver.calls, [("present", ("Menu",), {})])

    def test_a_python_plan_names_a_driver_from_a_module_beside_it(self):
        self.write(
            "_game_driver.py",
            "from bevy_remote_driver import Driver\nclass GameDriver(Driver):\n    pass\n",
        )
        plan = self.write(
            "plan.py",
            'from _game_driver import GameDriver\nBIN = "app"\nDRIVER = GameDriver\n'
            "def run(driver):\n    pass\n",
        )
        *_, driver_class, _ = load(plan)
        self.assertEqual(driver_class.__name__, "GameDriver")
        self.assertTrue(issubclass(driver_class, Driver))

    def test_a_driver_that_is_not_a_subclass_is_refused(self):
        plan = self.write(
            "plan.py",
            'BIN = "app"\nclass Other:\n    pass\nDRIVER = Other\ndef run(driver):\n    pass\n',
        )
        with self.assertRaisesRegex(PlanError, "not a subclass"):
            load(plan)

    def test_a_plan_names_exactly_one_target(self):
        both = self.write("both.json", json.dumps({"bin": "a", "example": "b", "steps": []}))
        neither = self.write("neither.py", "def run(driver):\n    pass\n")
        for plan in (both, neither):
            with self.assertRaisesRegex(PlanError, "exactly one"):
                load(plan)

    def test_a_python_plan_that_does_not_load_is_refused(self):
        plan = self.write("broken.py", "import no_such_module\n")
        with self.assertRaisesRegex(PlanError, "did not load"):
            load(plan)

    def test_a_missing_plan_is_refused(self):
        with self.assertRaisesRegex(PlanError, "no plan"):
            load(self.dir / "missing.py")

    def test_a_directory_stands_for_its_plans_in_name_order(self):
        for name in ("b.py", "a.json", "notes.txt", "_game_driver.py"):
            self.write(name, "")
        self.assertEqual(plans_in([self.dir]), [self.dir / "a.json", self.dir / "b.py"])

    def test_tags_are_a_list_of_strings(self):
        plan = self.write("plan.json", json.dumps({"bin": "app", "tags": "slow", "steps": []}))
        with self.assertRaisesRegex(PlanError, "list of strings"):
            load(plan)


class Choosing(unittest.TestCase):
    def test_every_plan_runs_with_no_tags_chosen(self):
        self.assertTrue(chosen([], False, [], []))
        self.assertTrue(chosen(["slow"], False, [], []))

    def test_only_runs_plans_with_one_of_its_tags(self):
        self.assertTrue(chosen(["physics", "editor"], False, ["editor", "ui"], []))
        self.assertFalse(chosen(["physics"], False, ["editor"], []))
        self.assertFalse(chosen([], True, ["editor"], []))

    def test_exclude_skips_plans_with_any_of_its_tags(self):
        self.assertFalse(chosen(["physics", "slow"], False, [], ["slow"]))
        self.assertFalse(chosen(["slow"], True, [], ["slow"]))

    def test_manual_runs_only_when_named_or_chosen(self):
        self.assertFalse(chosen(["manual"], False, [], []))
        self.assertTrue(chosen(["manual"], True, [], []))
        self.assertTrue(chosen(["manual"], False, ["manual"], []))
        self.assertFalse(chosen(["manual", "perf"], False, ["perf"], []))


class Failures(unittest.TestCase):
    def test_a_failure_names_its_step_and_caption(self):
        driver = SimpleNamespace(steps=["present('A')", "click('B')"], current_caption=None)
        plan = Path("plan.json")
        self.assertEqual(failed_step(driver, plan, None), ["step 2: click('B')"])
        driver.current_caption = "the menu"
        self.assertEqual(
            failed_step(driver, plan, None), ["step 2: click('B')", "caption: the menu"]
        )


class Durations(unittest.TestCase):
    def test_seconds_under_a_minute_minutes_above(self):
        self.assertEqual(duration(14.34), "14.3 s")
        self.assertEqual(duration(59.94), "59.9 s")
        self.assertEqual(duration(221.4), "3 min 41 s")


class Environments(unittest.TestCase):
    def environment(self, platform, binary, inherited=None):
        rustc = SimpleNamespace(stdout="/toolchain/lib\n")
        with (
            mock.patch("sys.platform", platform),
            mock.patch("subprocess.run", return_value=rustc),
            mock.patch.dict("os.environ", inherited or {}, clear=True),
        ):
            return environment(Path(binary), Path("/package"), 1234, Path("/settings"))

    def test_linux_finds_the_dylibs_beside_a_bin_or_above_an_example(self):
        for binary in ("/target/debug/app", "/target/debug/examples/app"):
            paths = self.environment("linux", binary)["LD_LIBRARY_PATH"].split(":")
            self.assertEqual(paths[0], "/toolchain/lib")
            self.assertIn(str(Path(binary).parent / "deps"), paths)
        self.assertIn("/target/debug/deps", paths)

    def test_macos_uses_its_fallback_variable(self):
        env = self.environment("darwin", "/target/debug/app")
        self.assertIn("/target/debug/deps", env["DYLD_FALLBACK_LIBRARY_PATH"].split(":"))
        self.assertNotIn("LD_LIBRARY_PATH", env)

    def test_an_inherited_path_is_kept_last(self):
        env = self.environment("linux", "/target/debug/app", {"LD_LIBRARY_PATH": "/mine"})
        self.assertEqual(env["LD_LIBRARY_PATH"].split(":")[-1], "/mine")

    def test_another_platform_sets_no_library_path(self):
        env = self.environment("win32", "/target/debug/app")
        self.assertNotIn("LD_LIBRARY_PATH", env)
        self.assertNotIn("DYLD_FALLBACK_LIBRARY_PATH", env)


if __name__ == "__main__":
    unittest.main()
