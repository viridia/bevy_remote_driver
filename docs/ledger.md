# bevy_remote_driver ledger

What has landed, what is next, the decisions that stand, and what is deferred and why. Requirements
are [requirements.md](requirements.md)'s `DR` numbers; the design is [design.md](design.md)'s `DD`
sections. Decisions here are `D<n>`, deferrals `X<n>`.

## Landed

| Commit | Change |
| --- | --- |
| (pending) | Extracted into its own repo (2026-10-11). Union of two copies: `bevy_action_map`'s `crates/bevy_remote_driver` at chunk 196 (Bevy main; `driver.locate` returns physical pixels, since `CursorMoved` takes a `physical_position`), and Panoply's (`bin` targets in `run.py`; `message` and `exits` steps). `run.py` runs from the project's directory, not from inside it. Verified at Bevy `c9c21c2b`: clippy clean, 12 unit tests, `plans/testbed/smoke.json` (17 steps) and `typing.py` (9) pass. |
| (pending) | User docs: `docs/writing-plans.md` (DR7.4) is the entry point for writing and running plans; `class Driver`'s docstring covers selectors, timeouts and the JSON form; `src/lib.rs` documents the `bevy_remote` feature screenshots need. `design.md` drops its step tables and §9's instructions for pointers. |
| (pending) | README for reviewers (Bevy maintainers weighing it as a model): a plan in Python and JSON, a passing and a failing run's real output, a reading order, status. `plans/testbed/panel.py` is the README's plan, verified passing. |
| (pending) | Docs stand alone. `design.md`'s examples are the testbed's; DD2's focus note describes Bevy's own keyboard input; DD6 and DD7 are general, with `bevy_action_map` one example; new DD4.5 (rules every client follows) and clients in DD4; DD8 matches the plan for clicks and keys. `proposal.md` becomes `upstream.md`: landed, proposed (`name_path`, readiness as state), under consideration (input as server methods). `requirements.md` no longer names `bevy_action_map`. |
| (pending) | Prose polish pass, to `bevy_action_map`'s house style: "nothing" as a subject rephrased, user docs free of citations and internal references, `upstream.md`'s proposals marked as such, the README's app-side summary and setup made accurate (setup now lives only in `writing-plans.md`, with the Bevy line), `key`'s docstring says what `code`, `logical_key` and `text` take. |
| (pending) | The two proposals filed as Bevy tickets by the author: `name_path` as bevy#26121, readiness as state as bevy#26120. `upstream.md` links them in place of the full text, and drops input as server methods until items 1 and 2 prototype it. DD8 cites the tickets. |
| (pending) | A run isolated from the developer's saved settings (new DR6.5): `run.py` sets `BEVY_SETTINGS_DIR` to `settings/` in the run's output directory, which is emptied each run. Whether an app wrote there is not checked or reported: only a plan about settings cares, and X5 is its step. DD4.3 and `writing-plans.md` (rule 4, the run's output) say so. Testbed plans pass unchanged (it saves no settings); the first app to save there is `arbor_day`'s settings chunk (its C2). |
| (pending) | A click scrolls its node into view (DD3.3): `driver.locate` moves the nearest hiding scrolled ancestor's `ScrollPosition` and answers `{ scrolled: true }`; the client's `click` waits two frames and asks again, up to 8 times. `bevy_math` moves to dependencies. First user: `arbor_day`'s C16, whose panel outgrew the window. Verified: clippy clean, 13 unit tests (one new), testbed `smoke.json` (17), `typing.py` (9), `panel.py` (8); `arbor_day`'s three plans pass against this checkout. |
| (pending) | The plan's location picks the workspace built (D3): `run.py` runs cargo in the plan's directory, falling back to the working directory for a plan outside any workspace. `writing-plans.md` (Running a plan) and DD4.3 say so. Not copied to `bevy_action_map` (D4): its plans run inside its own workspace, where the two agree. Verified from Panoply's directory with absolute plan paths: testbed `smoke.json` (17) and `typing.py` (9), and `arbor_day`'s four plans, each building its own workspace. |

| (pending) | Several plans in one run: `run.py` takes any number of plans, a directory standing for its `.py` and `.json` files, and runs them in turn, so a project's suite is one command and one approval. A failure, or a plan that is missing or does not load, does not stop the rest; an interruption does. A summary follows; the exit status is the worst. `writing-plans.md` (Running a plan, Reading a failure) and DD4.3 say so. Not copied to `bevy_action_map` (D4) unless asked. Verified: `plans/testbed` (3 plans), `arbor_day`'s `plans` (7), and two missing plans (status 2, both reported). |
| (pending) | Press and hold: `click` takes `hold`, the frames between press and release, as `key` does (default 1, as before), for a control that acts while held. DD4.1 says so. First user: `arbor_day`'s HUD camera buttons (its C57). Not copied to `bevy_action_map` (D4) unless asked. Verified: testbed `panel.py` (8), `smoke.json` (17), `typing.py` (9); `arbor_day`'s four plans, `smoke.py` holding each camera button. |
| (pending) | A path into the value `expect` compares: `at`, in Bevy's reflection path syntax (`"limbs[0].angle.shape"`) or as a list of keys and indices, walked over the JSON BRP returns, so lists are reached by index rather than compared whole. Client only. First user: `arbor_day`'s C27, whose plans read species fields inside an array of limb levels. Richer entity selectors are X7. Verified: testbed `panel.py` (8), `smoke.json` (17), `typing.py` (9); `arbor_day`'s four plans. |
| (pending) | State past the UI: `get` and `get_resource` return a component or resource, or the part at `at`, so a Python plan can take a snapshot; `set` and `set_resource` write one whole (BRP's insert) or one field by reflection path (BRP's mutate), on its own frame as an input lands. A rejected write fails the step, as now does a rejected `message`. For plans about what a setting does rather than how it is edited (`writing-plans.md`, rule 8). Client only. First user: `arbor_day`, whose species is a resource today and a document entity's component later. Verified: testbed `state.py` (14, new), `panel.py` (8), `smoke.json` (17), `typing.py` (9). |
| (pending) | The end of a run announced: after the last plan, on macOS a sound (Glass for a pass, Basso otherwise) and a notification giving the count passed, elsewhere the terminal bell, so a developer working elsewhere knows no more windows will come forward; a window closing between plans looked like the end. DD4.3 and `writing-plans.md` (Running a plan) say so. Client only. Not copied to `bevy_action_map` (D4) unless asked. First user: `arbor_day`. |
| (pending) | `driver.locate`'s upstream path: `upstream.md` gains "To propose", scrolling into view with `ScrollIntoView` acting on every scrolled ancestor and reflected (X6's ticket); DD8 gains a row for locating a node, which moves to the client once that lands. Docs only. |
| (pending) | Bevy main bumped to `2614884` (D2), with `bevy_action_map`, `bevy_immediate` and `arbor_day`; no code changes. `name_path` landed upstream as bevy#26250, so `upstream.md` and DD8 list it as landed and Next gains an item deleting `driver.select`. Scrolling into view is filed as bevy#26262, so `upstream.md` lists it as proposed and X6 is gated on it. Verified: clippy clean; testbed `panel.py` (8), `smoke.json` (17), `state.py` (14), `typing.py` (9); `arbor_day`'s four plans. |
| (pending) | A click as a server method (DD5.1), the first half of the input proposal drafted for Bevy on 2026-10-20: `driver.click { entity, button, hold }` or `{ window, position }` queues `CursorMoved`, the press and the release as `WindowEvent`s built from Bevy's types, written one frame apart from `RemoteLast`, and returns `{ frame }`, the frame that reads the release. Scrolling into view takes every hiding ancestor in one pass (DD3.3), shared with `driver.locate`; the click's cursor event waits a frame for layout, so the client's retry and `SCROLL_TRIES` go. `bevy_input` joins the dependencies. Verified: clippy clean, 20 unit tests (seven new); testbed `panel.py` (8), `smoke.json` (17), `state.py` (14), `typing.py` (9); `arbor_day`'s four plans against this checkout. |

## Next

1. **Keys and text as server methods**, the second half of the input proposal, after the click.
   bevy#25894 wants a runner in any language that speaks HTTP and JSON, so whatever a runner
   builds for itself, every runner builds again. `driver.py` writes `KeyboardInput` over BRP and
   derives `logical_key` and `text` from the key code, for a US layout (DD5.2), so every runner
   re-derives them and breaks when the message changes. Server methods in the plugin, on
   `driver.click`'s queue (DD5.1), returning the same `{ frame }`, written as
   `WindowEvent::KeyboardInput`, which `bevy_window` splits into `KeyboardInput` before
   `InputSystems`:
   - `driver.key { key_code, logical_key, text, hold }`, or `state` in place of `hold` to press
     or release alone (a modifier held across calls). The caller names both keys; the plugin
     derives nothing (the alternative in the author's Bevy ticket on synthetic keyboard input,
     2026-10-21, chosen to keep the proposal small). The client's `key`, `press`, `release`
     and `exits` fill them from `keys.py`'s `logical`, a convenience for plan writers that stays
     client-side.
   - `driver.type { text }`: the string gives each character's logical key and text; a US table
     in the plugin gives its key code and Shift, pressed and released around it. A character
     outside the table is sent as `KeyCode::Unidentified`, with no Shift, so it still types into
     a text field. The client's `CHARACTERS` table moves into the plugin.
   - `bevy_input` gains its `serialize` feature, for `KeyCode` and `Key` in the params.
   - Not doing: modifier state (a held Shift does not change `key`'s text; the caller names it);
     other layouts; gamepads. Verified by headless tests of the written messages, including
     Shift and an unknown character, and `plans/testbed/typing.py`. Then revise the proposal
     from what the two taught, and open it as a Bevy PR (`input.click`, `input.key`,
     `input.type`; a sub-issue of bevy#25894), listed in [upstream.md](upstream.md).
2. **Delete `driver.diagnostics`.** Past bevy#25824, BRP's own `diagnostics.get` reads
   `frame_count` as `driver.diagnostics` does (DD3.4). The client switches to it and the method
   goes. `FrameTimeDiagnosticsPlugin` is still the driver's to add, since it records the count and
   is not in `DefaultPlugins`. Verified by any plan end to end, since every wait reads the count.
3. **Delete `driver.select`.** Past bevy#26250, `world.query` takes a `name_path` filter
   (DD3.1). Its segments also take a number, the Nth child, which covers part of X7. It returns
   entities only, without `driver.select`'s per-match name path, so the client builds a report's
   path from `Name` and `ChildOf` queried with it, or reports entity ids. Update DD3.1, DD8's
   row, `writing-plans.md` where it describes selectors, and X7. Verified by the testbed plans,
   since every click selects.

## Decisions

- **D1** Unpublished (`publish = false`). Rust and a Python client together do not ship as a
  crate. Users depend on it by git. The Rust half is meant for Bevy (upstream.md); when Bevy has
  taken it, this repo becomes a Python-only project.
- **D2** Bevy is main, named by `git` and `branch = "main"`, the commit held by `Cargo.lock`. Every
  repo using the driver names Bevy the same way, since Cargo treats a branch and a rev as different
  sources and would build two Bevys. Bump all users together.
- **D3** `run.py` runs cargo in the plan's directory, so the workspace that holds the plan is the
  one built, wherever the command is run from; a plan with no `Cargo.toml` at or above it builds
  the working directory's. Output goes under `cargo metadata`'s `target_directory`, which respects `CARGO_TARGET_DIR`. A plan
  names a `bin` or an `example`, exactly one.
- **D4** `bevy_action_map` keeps its own copy (`crates/bevy_remote_driver`), since it is a published
  crate whose tests others may want to run without a second repo. This repo is the canonical copy;
  a fix that crate needs is copied across by hand, and the reverse.
- **D5** The client uses Python's standard library only (DD1), so a plan runs with nothing
  installed.

## Deferred

- **X1** What the driver becomes. [upstream.md](upstream.md) says what Bevy could take:
  `name_path` has landed (bevy#26250; Next item 3), readiness as state (bevy#26120) has not.
  What Bevy takes leaves the plugin; what it declines stays here; the client stays here either
  way. *Gate:* bevy#26120 decided.
- **X2** Calling an app's own server methods from a plan. Apps can register methods (DD6), but the
  client has no step for them. First a generic `call` step, which polls while it carries an
  expected `value` (a read) and invokes once without one (a write), matching with `expect`'s
  subset; it is planned in `bevy_action_map`'s chunk 149, with that crate's `remote` feature.
  Then, if raw `call` steps read badly, a Python helper over it or a step of the app's own.
  *Gate:* a plan that needs one of an app's methods, or chunk 149 landing there (port it here).
- **X3** A virtual pad that says which pad it is. The connection event carries `name`,
  `vendor_id` and `product_id`, and the client fills all three with a pad no vendor table knows,
  so every machine sees the same fallback. Exposing them is three optional arguments. *Gate:* a
  plan whose subject is brand-specific presentation.
- **X4** A second virtual pad, and disconnecting one. The client holds one pad's entity and
  connects it on first use; a second is another entity and a `pad` step that says which, and a
  disconnect is one more message. *Gate:* a plan driving two players, or one whose subject is a
  pad going away.
- **X5** Client methods for testing saved settings, on `Driver` beside the other steps: one that
  returns a settings file from the run's `settings/` parsed, and one that compares a key's value
  as `expect` does for a component. They read the file the app wrote, so a plan calls them after
  the app has saved (on exit, or once its deferred save has run). Python's `tomllib` (3.11 and
  later) parses TOML, so D5 holds. *Gate:* a plan whose subject is what an app saves.

- **X6** Scroll into view with Bevy's `ScrollIntoView` in place of the driver's own `ScrollPosition`
  arithmetic (DD3.3), triggering the event and comparing ancestors' positions to tell whether
  anything moved. Today the event acts only on the nearest `ScrollArea`, which many games'
  scrolled nodes are not. bevy#26262 makes it act on every `overflow: scroll` ancestor, reflected,
  in `bevy_ui`; [upstream.md](upstream.md) lists it. *Gate:* bevy#26262 merged, and a bump
  bringing it in.
- **X7** Richer entity selectors than a path of `Name`s (DD3.1): an index for the Nth match or
  child, given as a number in the list form, or a CSS-like syntax, the model Bevy UI authors know
  best. Value paths are `expect`'s `at`, which needed none of this. *Gate:* a plan that must reach
  an entity it cannot name, such as one row of a list built from data without computed names.
- **X8** Drags, and plans that record a demo or tutorial (the author, 2026-10-26). A drag is
  `driver.click`'s press, a `CursorMoved` each frame along a path, and the release, on the same
  queue (DD5.1), which picking turns into drag start, drag and end. For tests, a few frames in a
  straight line; for a recording, eased in and out (cubic) along a slight arc, its duration
  growing with distance as Fitts's law has it, so short and long drags both read as a hand's.
  Synthetic input moves no OS cursor, so a recording shows the driver's pointer as a cursor image
  the plugin draws when the plan asks, rather than warping the real cursor, which fights the
  user's mouse and needs the window in front. Recorded by capturing each frame in the app
  (Bevy's `Screenshot`, then ffmpeg), for an even frame rate with no focus needed; holds between
  steps, and perhaps captions, for pacing. First users: `arbor_day`'s curve editor, whose point
  drag, rectangle selection and Escape mid-drag are checked by hand, its turntable gestures, and a
  video of handle dragging (its X62). *Gate:* the first plan or video wanting a drag.

## Notes

- Origin: built in `bevy_action_map` from its chunk 145 (a BRP spike) onward; Panoply forked it at
  `bevy_action_map` commit `f641559`. Items 1–4 of Next were that repo's chunks 197, 202, 198 and
  199; X1–X4 were its X45, X32, X33 and X34. `bevy_action_map`'s copy stays (D4), so those entries
  may stay in its roadmap for its own copy. `upstream.md` began as the author's gist of
  2026-09-26, a supplement to bevy#25894.
