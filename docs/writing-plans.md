# Writing and running test plans

How to write, run and debug end-to-end tests for a Bevy app with this driver.

Reference:

- **Steps**: `class Driver` in [`client/driver.py`](../client/driver.py). The class docstring
  covers selectors, timeouts, type paths and the JSON form; each step's docstring covers that step.
- **App setup**: the crate docs at the top of [`src/lib.rs`](../src/lib.rs).

## Setup

1. Depend on the driver by git, and name Bevy exactly as the driver does. Any other form, such as
   a `rev` or a release, builds a second Bevy.

   ```toml
   bevy = { git = "https://github.com/bevyengine/bevy.git", branch = "main", features = ["bevy_remote"] }
   bevy_remote_driver = { git = "https://github.com/viridia/bevy_remote_driver" }
   ```

2. Add the plugin, as `src/lib.rs` describes.
3. Check out this repository next to the project (`../bevy_remote_driver`), for the client.

## Writing a plan

1. **Find what to select.** Read the app's scenes for `#Name`s. If what the test needs has none,
   add one where the entity is declared, rather than selecting by component or position. Name
   what plans refer to, not every entity. A screen built from data in a loop gets computed names
   (`Name::new`); compute them from an identity the screen holds (a key, an index), not from the
   text it draws, which may be translated.
2. **Write a Python plan** in the project's `plans/`. Set `BIN` or `EXAMPLE` for the target to
   launch, and `PACKAGE` if it is not the root package's. Set `FEATURES` if the target declares
   `required-features`; without them, cargo skips the target and the run fails with "the build
   produced no example named …". Define `run(driver)`, calling the steps as methods. Start with
   `present` on the screen's root, with `ready=True`. Give a step whose point is not obvious a
   comment, and a sequence you repeat a named function.
3. **Write JSON only for a flat list of steps that reads as one.** It names `bin` or `example`
   and lists `steps`. `plans/testbed/smoke.json` is an example, and `panel.py` and `typing.py`
   beside it are Python ones.
4. **Put the app into the state the test assumes, rather than assuming it.** Settings the app
   saves with `bevy_settings` start empty on every run, and your own saved settings are left
   alone. Anything else the app saves, such as files it writes, makes it a different app on its
   second run. Reach a known state through the app's own controls at the start, and end there
   too. A plan that fails halfway leaves its changes behind, so the start is the one that counts.
5. **Wait for a condition, then frames, then seconds.** A covered window runs at about half speed,
   so a margin in seconds for the app's own logic passes in front and fails behind another window.
   Use `seconds` only for assets loaded after a scene is ready, such as icons.
6. **Assert with checks, not screenshots.** `expect`, `absent` and `present` are what make a test
   fail. A screenshot shows that something is drawn, for the person reading the run.
7. **Match a range where an exact value would only restate the app's arithmetic.** In an `expect`
   value, `Between(low, high)` (`from driver import Between`) matches any number in that range,
   both ends included, so a plan checks a count's order of magnitude without breaking on every
   change to how it is computed.

## Running a plan

```sh
python3 ../bevy_remote_driver/client/run.py plans/<plan>.py
```

It builds the target, launches it with the driver's port set, runs the steps, closes the app and
prints a report. Always run it as this one command; do not start the app in one command and drive
it in another. Anything that brings another window forward in between, such as an approval
prompt, covers the app and changes its timing.

The workspace built is the one that holds the plan, so the command works from any directory, given
the plan's path from there. A plan outside any Cargo workspace builds the one you are in.

Screenshot steps bring the app's window to the front, taking focus from whatever had it.

Output goes to `remote-driver/<target>/<plan>/` under Cargo's target directory (usually
`target/`), emptied at the start of each run: `report.txt`, `app.log`, the screenshots, and
`settings/`, where the app's `bevy_settings` files go for the run (Bevy main reads
`BEVY_SETTINGS_DIR`; 0.20 does not).

## Reading a failure

The exit status is 0 on a pass, 1 when a step failed, 2 when the plan or the build could not run.

1. **The report first.** It names the failing step by number, and for a Python plan the line that
   called it, with what the step expected and what it found, and whether the app had exited.
2. **Then the log**, for what the app did: panics, errors, its own logging.
3. **Then the screenshots.**

A key or button released when the plan did not release it is usually focus: something took focus
from the app's window during a hold, and Bevy releases held input when its window loses focus.
Rerun before looking for a bug.
