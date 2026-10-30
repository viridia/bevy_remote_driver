# How `bevy_remote_driver` works

A driver for a running Bevy app, for automated tests. Its sections are numbered `DD1`–`DD9`, and its
requirements are [requirements.md](./requirements.md)'s `DR` numbers. Where a claim rests on reading
Bevy's source rather than on running something, it says so.

The worked examples are the driver's own testbed, `testbed/src/main.rs`: a menu with a Start button
that opens a panel, which Escape closes, and a line showing what is typed. Its plans are in
`testbed/plans/`.

---

## 1. Shape

An app and a client, joined by BRP over HTTP:

```
app under test                          client (Python, standard library only)
  RemotePlugin, RemoteHttpPlugin          launches the app, waits for the port
  FrameTimeDiagnosticsPlugin      <-->  runs a plan step by step
                                          writes input with BRP's own methods
                                          saves screenshots, writes the report, closes the app
```

The driver is only the client. Selecting, input and the frame count are all BRP's built-in
methods (DR1.3, DD8), so the app needs no code of the driver's: it adds BRP's two plugins and
`FrameTimeDiagnosticsPlugin` in its `main` (DD7).

The app adds them only when the environment variable `BEVY_REMOTE_DRIVER_PORT` is set, on that
port, so a build started normally opens no port (DR1.5). A value that is not a port panics at
startup, since running without the port would leave the client timing out with no reason given.

## 2. What BRP already does

Measured on macOS against a small example app with the `bevy/bevy_remote` feature, except where
marked read:

| Need | BRP | Finding |
| --- | --- | --- |
| Write a key | `world.write_message`, `bevy_input::keyboard::KeyboardInput` | reaches the app with or without focus |
| Screenshot | `world.spawn_entity` a `Screenshot`, then `world.observe+watch` `ScreenshotCaptured` | empty when the window is covered: every byte zero, alpha included, and the same inside the app |
| Bring the window forward | `world.mutate_components`, `Window.focused` to `true` | `bevy_winit` calls winit's `focus_window`; a capture 0.1 s later is correct |
| Click | `world.write_message`, `WindowEvent::CursorMoved` then `MouseButtonInput` | read, from Bevy's `examples/remote/integration_test.rs` |
| Quit | `world.write_message`, `bevy_app::AppExit` | read: `AppExit` is a reflected `Message` |
| Connect a gamepad | `world.spawn_entity` an empty entity, then `world.write_message` `GamepadConnectionEvent` naming it | `gamepad_connection_system` inserts `Gamepad` onto the entity the event names (DD5.3) |
| Write app state | `world.insert_components` / `world.insert_resources` whole; `world.mutate_components` / `world.mutate_resources` by reflection path | a mutate path is reflection's, which steps into a newtype (`.0`) and a tuple variant, where the JSON BRP returns has neither |

Two behaviours matter for every test. A covered window runs at about half the frame rate, so timing
is measured in frames (DD4.3). And a window brought forward takes focus from whatever had it, the
editor included, which is the cost of a correct screenshot.

`KeyboardFocusLost` fires when the window loses focus, and Bevy's keyboard input releases every held
key on it (read, `bevy_input`'s `keyboard_input_system`). So a test that holds a key while someone
clicks another window sees the key released, which is Bevy behaving correctly rather than the driver
failing.

## 3. Reading the app

### 3.1 Selectors

A selector is `world.query`'s `name_path` filter (bevy#26250), which returns every entity the path
matches (DR2.1). A path is an array. The last name matches an entity's `Name` exactly; each earlier
name must match one of that entity's ancestors, in order from the root down, at any depth.
`["Settings", "Jump", "Rebind"]` is an entity named `Rebind` with an ancestor `Jump` that itself
has an ancestor `Settings`. A number in the path is a child index: `["Settings", 0]` is the first
child of `Settings`. A path cannot start with one.

An array needs no escaping, since a `Name` may contain any character, and the server needs no
parser. The client also takes a path as a string, `"Settings/Jump/Rebind"`, and splits it on `/`
before sending it; a name containing `/`, and an index, are written in the array form.

A step that needs one entity waits until exactly one matches, and at its timeout reports the
matches with their full name paths (DR2.2), joined with `/`. The query returns entities only, so
the client builds each path from every entity's `Name` and `ChildOf`, read in one more query, and
only when the step fails.

### 3.2 Readiness

A client needs no readiness of its own (DR3.2): waiting for the entity a step needs is enough.
BSN applies a queued scene, an entity and all its descendants, in one call from an exclusive system
in `SpawnScene`, and BRP answers in `RemoteLast`, so a request sees a scene whole or not at all.
Until then the scene's entity is empty, and a path of the names it gives matches nothing.
`testbed/tests/scene_readiness.rs` shows this after every frame, and after every system of `SpawnScene`.

Two cases show part of a scene first: an entity named outside its scene, then given one with
`queue_apply_scene`, and an entity given related scenes. Each shows its own entity before the rest,
so a plan waits for the entity its step needs, a descendant, not the root. Every step that names an
entity waits for it (3.1), so this is what a plan does without trying.

No event marks when assets a scene names have loaded: BSN's `Ready` waits only for scenes the
scene inherits, not for a `Handle` it holds, and an asset a system requests later is not covered
either. A plan waits for those with `seconds` (DR3.6). Loading runs on IO threads rather than per frame, so
time is the right measure there, where the app's own logic is measured in frames.

### 3.3 Where a node is drawn

The driver does not locate nodes itself. BRP's `input.click` (5.1) finds the window a UI node is
drawn in and its centre in physical pixels, and first scrolls a node that a scrolled ancestor
hides into view, with `bevy_ui`'s `ScrollIntoView`, which moves every `overflow: scroll` ancestor
(bevy#26262). A node drawn to an image rather than a window is refused.

### 3.4 Diagnostics

Every wait measured in frames reads the `frame_count` diagnostic with BRP's own `diagnostics.get`
(bevy#25824). `FrameTimeDiagnosticsPlugin` records it from `FrameCount`, which is not reflected
and so not readable itself. That plugin is not in `DefaultPlugins`, so the app's setup adds it,
unless the app already does. A value is an `f64`, which holds a frame count exactly for longer
than any test runs.

## 4. Plans and the client

The client is its own, rather than an existing harness such as Playwright or Jasmine. What makes
Playwright worth having is its browser half, which cannot reach a native window, so the rest would
be a Node toolchain wrapped around the same BRP calls.

Who writes a test decides what its client should be. A probe written on the spot, by a developer or
an agent, and then thrown away, wants no project to set up: that is this client, Python needing
only the standard library (DR7.3), which also runs the JSON form. An acceptance test in CI wants
the project's own language and its test runner: a Rust client on `BrpClient`
([bevy#25837](https://github.com/bevyengine/bevy/pull/25837)), each test an ordinary test. The rules
in DD4.5 are what any client follows, so a JSON plan runs the same on either.

### 4.1 A plan

A plan names the target to launch and lists the steps to run against it. The vocabulary is the same
whichever way it is written: as a Python program, which is the default (DD4.2), or as JSON, which
the client reads with only the standard library and no build step (DR7.1, DR7.3):

```json
{
  "bin": "testbed",
  "steps": [
    { "click": "Menu/Start" },
    { "present": "Panel" },
    { "key": "Escape" },
    { "absent": "Panel" }
  ]
}
```

A plan names a `bin` or an `example`. It names its `package` when cargo needs one to tell two
targets of that name apart, and its `features` when the target declares `required-features`: cargo
skips such a target rather than refusing to build it, so a plan leaving them out fails as a target
that does not exist.

The step reference is the client's docstrings: `class Driver` in `src/bevy_remote_driver/driver.py` for selectors,
timeouts (DR3.4) and the JSON form, and each step method for itself. The instructions for writing a
plan are [writing-plans.md](writing-plans.md) (DR7.4).

The three checks (`present`, `absent`, `expect`) wait rather than look once, because what follows an
input can land a frame or more later: a menu that animates out, a value set by the next system to
run. A timeout of `{ "frames": 0 }` looks once, so checking that something is still gone later is a
`frames` step followed by an `absent` that looks once.

`pad` has no `hold` because it does not need one: a control that carries a value is left at it, and
returned to `0.0` by a second step, which is what spans frames for a pad. A key or a mouse button
has no value, so `hold` is the only thing that can span them, for `key` and for `click`.

### 4.2 A Python plan

A plan is a Python program by default, calling the same functions the JSON runner calls (DR7.2). The
client's API is the step vocabulary: each step is a method on a `Driver` object with the same name
and arguments. The program sets `BIN` or `EXAMPLE`, and `PACKAGE` and `FEATURES` where a JSON plan
would, and defines `run(driver)`; it runs through the same command as a JSON plan, and a failure's
report adds the line of the program that made the failing call.

A loop, a branch or a computed value needs a program. So does most of what a test is actually made
of: a sequence it repeats, which becomes a named function instead of the same six steps written out
three times, and a step whose point is not its arguments, which gets a sentence beside it. JSON says
neither. `testbed/plans/typing.py` types one character at a time and checks the text after each, a
loop that JSON would spell out character by character.

JSON stays for a test that is a flat list of steps and reads as one, and for DR7.1's own sake: it
runs without compiling or importing anything, which `testbed/plans/smoke.json` is kept to show.

### 4.3 A run

`bevy-remote-driver <plan>` is the one command (DR6.1). It:

1. builds the target in the Cargo workspace that holds the plan, running cargo in the plan's
   directory (in the working directory, for a plan outside any workspace), with
   `cargo build --bin` or `--example`, adding `-p` and `--features` where the plan names them, and
   fails fast on a build error. Cargo's JSON messages give the binary's path and its package's
   directory;
2. picks a free port, and starts the binary with `BEVY_REMOTE_DRIVER_PORT` set, its output going to
   a log file. `CARGO_MANIFEST_DIR` is set to the package's directory, where Bevy looks for
   `assets/` as it would under `cargo run`. `BEVY_SETTINGS_DIR` is set to `settings/` in the run's
   output directory, so `bevy_settings` reads and writes there instead of the developer's own
   settings, starting empty on every run (DR6.5). On macOS `DYLD_FALLBACK_LIBRARY_PATH` names the
   toolchain's libstd and the Bevy dylib, which `dynamic_linking` needs;
3. polls until the app answers a query for its primary window, which every key step names;
4. runs the steps;
5. writes `AppExit`, waits up to three seconds, then kills the process. This runs in a `finally`, so
   an interrupted run closes the app too (DR6.2).

Everything a run produces goes to `remote-driver/<target>/<plan name>/` under Cargo's target
directory (from `cargo metadata`, so `CARGO_TARGET_DIR` is respected), emptied first: the log, the
screenshots, the app's saved settings and `report.txt`. The report is also printed. A failure gives the step's number and
text, what it expected, what it found, whether the app had exited, and those paths (DR6.3). The exit
status is 0 on a pass, 1 on a failure and 2 when the plan could not run.

Given several plans, or a directory standing for its `.py` and `.json` files, the command runs
them in turn, one app at a time, so a project's whole suite takes one command and one approval.
A failure does not stop the rest; an interruption does. A summary of one line per plan follows
the last report, and the exit status is the worst of the plans'. The run then says it is over,
once, since a window closing does not tell a developer working elsewhere whether another will come
forward: on macOS a sound (Glass for a pass, Basso otherwise) and a notification, elsewhere the
terminal bell.

Consecutive inputs land on distinct frames (DR3.5). After writing an input the client reads
`frame_count`, and it does not write the next input until the count has passed that value. The read
is answered no earlier than the frame the input was, since BRP answers requests in order, so a
larger count is a later frame.

### 4.4 Screenshots

A covered window captures an empty image (DD2), so the step first sets `Window.focused` and waits a
tenth of a second. It then opens a `world.observe+watch` stream on `ScreenshotCaptured`, spawns a
`Screenshot` of the primary window, and takes the event whose entity is the one spawned. Watching
first means the capture cannot finish before anything is listening. The image arrives as BGRA bytes
in a JSON array, which the client reorders and writes as a PNG with `zlib`. A capture whose every
byte is zero fails the step rather than saving a blank file.

### 4.5 Rules every client follows

What made the driver's tests reliable, and what another client must do for a plan to mean the same:

- **Each input lands on its own frame** (DD4.3). A press and a release in one frame are missed by a
  system that reads `pressed()`.
- **App logic is timed in frames, loading in seconds** (DD3.2). A covered window can run at half its
  frame rate, while assets load on IO threads regardless.
- **A screenshot brings its window forward first, and an all-zero capture fails** (DD4.4).
- **Clicks, keys and text go through BRP's `input.*` methods**, which build Bevy's own events, so
  a change to their shape is Bevy's to keep working rather than every client's (DD5.1, DD5.2).
- **A virtual gamepad is an empty entity**, a `GamepadConnectionEvent` naming it, then
  `RawGamepadEvent`s, reporting no vendor or product id (DD5.3).

## 5. Input

### 5.1 Clicks

A click is BRP's `input.click { entity, hold }` (bevy#26333), which queues three `WindowEvent`s,
each on its own frame: `CursorMoved` to the node's centre, the press, then the release `hold`
frames later. Picking needs the cursor to have moved before the press, and a press and a release
in the same frame are indistinguishable from a tap the app may treat differently. A hidden node is
scrolled into view first (3.3). The method returns `{ frame }`, the frame that reads the release.

`input.key` and `input.type` share its queue, so one plan's inputs land in order, a frame apart.
Messages the client writes itself, such as a gamepad's (5.3), are not on that queue, so the client
sends each input on the frame after the one the last input returned, whichever way it was sent.

### 5.2 Keys and text

A key is BRP's `input.key { key_code, hold }` and text is `input.type { text }` (bevy#26342), both
to the primary window. Without a `logical_key`, the server takes it and `text` from a US layout,
and `input.type` holds Shift where a character needs it, so `type` produces what a real keyboard
would. A test that needs another layout supplies `logical_key` and `text` itself. `press`,
`release` and `exits` send a `state` alone: a press with no release, or a release.

### 5.3 Gamepads

The client spawns an empty entity, writes a `GamepadConnectionEvent` naming it, then writes
`RawGamepadEvent`s for its buttons and axes. `bevy_input` does the rest as it would for a gilrs
device: the connection is what puts `Gamepad` on the entity, and the frame between writes (DR3.5) is
enough for that to have happened before the first control arrives. A real pad plugged in at the same
time is a second gamepad, not a conflict.

A button and an axis are separate enums with no name in common, so the client tells them apart by a
set of the six `GamepadAxis` variants and takes every other name for a button. A name in neither
enum fails to deserialize, and the error names the word, so there is no table to keep in step with
Bevy's.

The pad reports no vendor or product id, which resolves to whatever a game shows for a pad it does
not recognize. That is the stable answer for a test: the alternative is the brand of whatever is
plugged into the machine the plan runs on.

## 6. An app's own methods

A crate the app uses can register server methods of its own, for state BRP cannot read from
reflected components, and the driver never depends on it (DR1.2). `bevy_action_map`, an input
mapper, is the example: behind a `remote` feature it adds `action_map.dump`, each input context's
actions with their phase and value by path, so a test asks whether an action fired without the app
printing anything; and `action_map.authority`, which sets a delegated action's value on a context
entity.

Raw input needs no such method: a message BRP writes is sampled as a real one, so a test of an
ordinary bound action sends a key. The client has no step for calling an app's methods yet (the
ledger's X2).

## 7. Names in an app

An id is a `Name`, given in a scene with BSN's `#Name`, which already inserts one (DR2.3). No
component of the driver's own: `Name` is already there, already reflected, and already what an
inspector shows. Names are given to what a plan needs to select, and not to every entity. The
testbed names four: `#Menu`, `#Start`, `#Typed` and `#Panel`.

A screen built from data in a loop has no `#Name` to write, and no way to name one row without
naming all of them: the name is a computed `Name::new`, and the rule becomes which *kinds* of entity
are named. What such a name is computed *from* matters more than how many there are. It should be an
identity the screen already holds, such as a key or an index, rather than the text it draws: display
text may be translated, and two rows can show the same label.

An app adds BRP in its `main`, with the lines the [README](../README.md) gives, and
enables the `bevy` crate's `bevy_remote` feature. Without the feature `Image` has no serialization
registered, and the screenshot step panics the app inside `bevy_remote` rather than failing the
step.

## 8. Where each piece ends up

| Piece | Home | Why |
| --- | --- | --- |
| Selection by name path | upstream, done: a filter on `world.query` (bevy#26250) | any app's test wants it, and it needs only `Name` and `ChildOf` |
| Waiting for a scene | the client, by name | a request sees a scene whole or not at all (3.2), so no readiness state is needed (bevy#26120 withdrawn) |
| Locating a UI node | upstream, done: inside `input.click` (bevy#26333), scrolling with `ScrollIntoView` (bevy#26262) | a click is what needs a node's position, so the click finds it |
| Diagnostics over BRP | upstream, done: `diagnostics.get` (bevy#25824) | BRP took a method rather than reflecting the store |
| Clicks, keys and text | upstream, done: `input.click` (bevy#26333), `input.key` and `input.type` (bevy#26342) | every client otherwise re-derives the messages, and breaks at runtime when their shape changes |
| Gamepads | the client | the messages are reflected and writable (bevy#25904) |
| Bringing the window forward before a screenshot | the client | a policy for tests, not a property of screenshots |
| Plans, the runner, the report | the client | upstream has no reason to want a Python client |
| An app's own state | the app's crates (DD6) | their own types |

Every server method the driver's plugin once added has gone upstream ([upstream.md](./upstream.md)),
and the plugin with them: an app's setup is a few lines of its own, and this project is a client.

## 9. Writing a test

The instructions DR7.4 requires are [writing-plans.md](writing-plans.md), with the client's
docstrings as the step reference. `testbed/plans/` holds a JSON plan (`smoke.json`) and three Python
plans (`typing.py`, `state.py`, and `panel.py`, the README's example), which between them use every
step but `pad`, `message` and `exits`, which the testbed has no use for.
