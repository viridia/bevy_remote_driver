# bevy_remote_driver

End-to-end tests for Bevy apps, driven from outside over the Bevy Remote Protocol (BRP). A test
launches the real app, finds entities by name, sends input, waits for the app to react, checks
components, and takes screenshots.

- **The plugin**: the app adds `RemoteDriverPlugin`. It does nothing unless
  `BEVY_REMOTE_DRIVER_PORT` is set, so it stays in normal builds.
- **The client**: Python, standard library only. It builds the app, launches it, runs a plan,
  reports, and closes the app.

## A plan

From `plans/testbed/panel.py`, against `examples/testbed.rs`:

```python
"""Start opens the panel, typing reaches the text field, and Escape closes the panel."""

EXAMPLE = "testbed"
TEXT = "bevy_ui::widget::text::Text"


def run(driver):
    driver.present("Menu", ready=True)     # wait for the scene to finish spawning
    driver.screenshot("menu")
    driver.click("Menu/Start")             # a UI node, by its path of Names
    driver.present("Panel")                # waits until it appears
    driver.type("Hi!")
    driver.expect("Menu/Typed", TEXT, "Hi!")
    driver.key("Escape")
    driver.absent("Panel")
```

The same steps can be written as JSON when a test is a flat list:

```json
{
  "example": "testbed",
  "steps": [
    { "present": "Menu", "ready": true },
    { "click": "Menu/Start" },
    { "present": "Panel" },
    { "key": "Escape" },
    { "absent": "Panel" }
  ]
}
```

On the app's side, the only change is a `Name` on what a test refers to (`#Start` in a BSN scene).

One command builds the app, launches it, runs the plan and closes it:

```
$ python3 ../bevy_remote_driver/client/run.py plans/testbed/panel.py
plan: plans/testbed/panel.py
passed: 8 steps
log: /path/to/project/target/remote-driver/testbed/panel/app.log
screenshot: /path/to/project/target/remote-driver/testbed/panel/menu.png
```

With `"Hello"` in place of `"Hi!"` in the `expect`, the run fails, naming the step and the line of
the plan that ran it:

```
plan: plans/testbed/panel.py
FAILED
step 6: expect('Menu/Typed', 'bevy_ui::widget::text::Text', 'Hello')
at plans/testbed/panel.py:13
expected: bevy_ui::widget::text::Text on Menu/Typed to be "Hello"
found: "Hi!"
log: /path/to/project/target/remote-driver/testbed/panel/app.log
screenshot: /path/to/project/target/remote-driver/testbed/panel/menu.png
```

## Reading further

1. [docs/proposal.md](docs/proposal.md): what Bevy itself could take, so that a client in any
   language needs no plugin.
2. [docs/requirements.md](docs/requirements.md) and [docs/design.md](docs/design.md): what the
   driver must do, and how it does it.
3. [docs/writing-plans.md](docs/writing-plans.md): writing and running plans.
4. The code: the plugin (`src/`, about 500 lines of Rust with its tests) and the client
   (`client/`, about 750 lines of Python).

[docs/ledger.md](docs/ledger.md) is the development log: what has landed, what is next, and the
decisions so far.

## Status

Early. It tests the examples of [`bevy_action_map`](https://github.com/viridia/bevy_action_map)
and the game Panoply, and has run only on macOS. It builds against Bevy main. Discussion of
end-to-end testing in Bevy is in [bevy#25894](https://github.com/bevyengine/bevy/issues/25894).

Not published to crates.io; to use it, depend on it by git:

```toml
bevy_remote_driver = { git = "https://github.com/viridia/bevy_remote_driver" }
```

## License

MIT or Apache-2.0, at your option.
