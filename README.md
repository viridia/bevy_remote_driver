# bevy-remote-driver

[![CI](https://github.com/viridia/bevy_remote_driver/actions/workflows/ci.yml/badge.svg)](https://github.com/viridia/bevy_remote_driver/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](https://github.com/viridia/bevy_remote_driver#license)
[![Bevy tracking](https://img.shields.io/badge/Bevy%20tracking-main-lightblue)](https://bevy.org/learn/quick-start/plugin-development/#main-branch-tracking)

`bevy-remote-driver` runs end-to-end tests against a Bevy app, driving it from outside over the
Bevy Remote Protocol (BRP). You write a plan, a short Python script, and the driver launches your
real app and works through it: it finds entities by name, clicks and types as a player would,
waits for the app to react, checks the components that result, and takes screenshots along the
way.

The driver is plain Python with no dependencies, and your app needs only a few lines of setup.

## A plan

```python
"""Start opens the panel, typing reaches the text field, and Escape closes the panel."""

BIN = "testbed"
TEXT = "bevy_ui::widget::text::Text"


def run(driver):
    driver.present("Menu")  # wait for the scene to finish spawning
    driver.screenshot("menu")
    driver.click("Menu/Start")  # a UI node, by its path of Names
    driver.present("Panel")  # waits until it appears
    driver.type("Hi!")
    driver.expect("Menu/Typed", TEXT, "Hi!")
    driver.key("Escape")
    driver.absent("Panel")
```

A plan that is a flat list of steps can be JSON instead.

## Installing

The client needs Python 3.11 or later. Install it with [uv](https://docs.astral.sh/uv/):

```sh
uv tool install git+https://github.com/viridia/bevy_remote_driver
```

This puts `bevy-remote-driver` on your `PATH`; `uv tool upgrade bevy-remote-driver` updates it.

## Setting up the app

1. Enable the `bevy` crate's `bevy_remote` feature, which screenshots need. Without it, a
   screenshot step panics the app.

   ```toml
   bevy = { git = "https://github.com/bevyengine/bevy.git", branch = "main", features = ["bevy_remote"] }
   ```

2. Enable BRP in the app's `main` when the driver starts it, on the port it gives in
   `BEVY_REMOTE_DRIVER_PORT`:

   ```rust
   use bevy::{
       diagnostic::FrameTimeDiagnosticsPlugin,
       remote::{RemotePlugin, http::RemoteHttpPlugin},
   };

   if let Ok(port) = std::env::var("BEVY_REMOTE_DRIVER_PORT") {
       let port = port.trim().parse().expect("BEVY_REMOTE_DRIVER_PORT is not a port");
       app.add_plugins((
           RemotePlugin::default(),
           RemoteHttpPlugin::default().with_port(port),
           FrameTimeDiagnosticsPlugin::default(),
       ));
   }
   ```

   A player who starts the game normally gets no open port. Leave out
   `FrameTimeDiagnosticsPlugin` if the app already adds it; the driver counts frames with it.

3. Give a `Name` to each entity a plan refers to: `#Start` in a BSN scene.

## Running plans

```
$ bevy-remote-driver plans/panel.py
plan: plans/panel.py
passed: 8 steps
log: /path/to/project/target/remote-driver/testbed/panel/app.log
screenshot: /path/to/project/target/remote-driver/testbed/panel/menu.png
```

It builds the app, launches it, runs the plan and closes it. A directory runs every plan in it. A
failure names the step and the line of the plan that ran it:

```
plan: plans/panel.py
FAILED
step 6: expect('Menu/Typed', 'bevy_ui::widget::text::Text', 'Hello')
at plans/panel.py:13
expected: bevy_ui::widget::text::Text on Menu/Typed to be "Hello"
found: "Hi!"
```

[Writing and running test plans](https://github.com/viridia/bevy_remote_driver/blob/main/docs/writing-plans.md)
covers the rest: writing a plan that holds up, running several, and reading a failure. Each step
is documented in its docstring in
[`class Driver`](https://github.com/viridia/bevy_remote_driver/blob/main/src/bevy_remote_driver/driver.py).

## Compatibility

| bevy-remote-driver | Bevy                 |
| ------------------ | -------------------- |
| main (git)         | main, from `784b85d` |

Bevy 0.20 lacks the input methods the driver calls; 0.21 will be the first release with them.

## Status

Early, and run only on macOS so far. It tests the examples of
[`bevy_action_map`](https://github.com/viridia/bevy_action_map) and the game Panoply. How it works
is in [docs/design.md](https://github.com/viridia/bevy_remote_driver/blob/main/docs/design.md),
and what Bevy took from it in
[docs/upstream.md](https://github.com/viridia/bevy_remote_driver/blob/main/docs/upstream.md).
Discussion of end-to-end testing in Bevy is in
[bevy#25894](https://github.com/bevyengine/bevy/issues/25894).

## License

MIT or Apache-2.0, at your option.
