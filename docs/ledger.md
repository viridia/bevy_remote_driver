# bevy_remote_driver ledger

What has landed, what is next, the decisions that stand, and what is deferred and why. Requirements
are [requirements.md](requirements.md)'s `DR` numbers; the design is [design.md](design.md)'s `DD`
sections. Decisions here are `D<n>`, deferrals `X<n>`.

## Landed

| Chunk | Change |
| --- | --- |
| 1 | Extracted into its own repo (2026-10-11). Union of two copies: `bevy_action_map`'s `crates/bevy_remote_driver` at chunk 196 (Bevy main; `driver.locate` returns physical pixels, since `CursorMoved` takes a `physical_position`), and Panoply's (`bin` targets in `run.py`; `message` and `exits` steps). `run.py` runs from the project's directory, not from inside it. Verified at Bevy `c9c21c2b`: clippy clean, 12 unit tests, `plans/testbed/smoke.json` (17 steps) and `typing.py` (9) pass. |
| 2 | User docs: `docs/writing-plans.md` (DR7.4) is the entry point for writing and running plans; `class Driver`'s docstring covers selectors, timeouts and the JSON form; `src/lib.rs` documents the `bevy_remote` feature screenshots need. `design.md` drops its step tables and §9's instructions for pointers. |
| 3 | README for reviewers (Bevy maintainers weighing it as a model): a plan in Python and JSON, a passing and a failing run's real output, a reading order, status. `plans/testbed/panel.py` is the README's plan, verified passing. |
| 4 | Docs stand alone. `design.md`'s examples are the testbed's; DD2's focus note describes Bevy's own keyboard input; DD6 and DD7 are general, with `bevy_action_map` one example; new DD4.5 (rules every client follows) and clients in DD4; DD8 matches the plan for clicks and keys. `proposal.md` becomes `upstream.md`: landed, proposed (`name_path`, readiness as state), under consideration (input as server methods). `requirements.md` no longer names `bevy_action_map`. |
| 5 | Prose polish pass, to `bevy_action_map`'s house style: "nothing" as a subject rephrased, user docs free of citations and internal references, `upstream.md`'s proposals marked as such, the README's app-side summary and setup made accurate (setup now lives only in `writing-plans.md`, with the Bevy line), `key`'s docstring says what `code`, `logical_key` and `text` take. |
| 6 | The two proposals filed as Bevy tickets by the author: `name_path` as bevy#26121, readiness as state as bevy#26120. `upstream.md` links them in place of the full text, and drops input as server methods until items 1 and 2 prototype it. DD8 cites the tickets. |
| 7 | A run isolated from the developer's saved settings (new DR6.5): `run.py` sets `BEVY_SETTINGS_DIR` to `settings/` in the run's output directory, which is emptied each run. Whether an app wrote there is not checked or reported: only a plan about settings cares, and X5 is its step. DD4.3 and `writing-plans.md` (rule 4, the run's output) say so. Testbed plans pass unchanged (it saves no settings); the first app to save there is `arbor_day`'s settings chunk (its C2). |
| 8 | A click scrolls its node into view (DD3.3): `driver.locate` moves the nearest hiding scrolled ancestor's `ScrollPosition` and answers `{ scrolled: true }`; the client's `click` waits two frames and asks again, up to 8 times. `bevy_math` moves to dependencies. First user: `arbor_day`'s C16, whose panel outgrew the window. Verified: clippy clean, 13 unit tests (one new), testbed `smoke.json` (17), `typing.py` (9), `panel.py` (8); `arbor_day`'s three plans pass against this checkout. |
| 9 | The plan's location picks the workspace built (D3): `run.py` runs cargo in the plan's directory, falling back to the working directory for a plan outside any workspace. `writing-plans.md` (Running a plan) and DD4.3 say so. Not copied to `bevy_action_map` (D4): its plans run inside its own workspace, where the two agree. Verified from Panoply's directory with absolute plan paths: testbed `smoke.json` (17) and `typing.py` (9), and `arbor_day`'s four plans, each building its own workspace. |
| 10 | Several plans in one run: `run.py` takes any number of plans, a directory standing for its `.py` and `.json` files, and runs them in turn, so a project's suite is one command and one approval. A failure, or a plan that is missing or does not load, does not stop the rest; an interruption does. A summary follows; the exit status is the worst. `writing-plans.md` (Running a plan, Reading a failure) and DD4.3 say so. Not copied to `bevy_action_map` (D4) unless asked. Verified: `plans/testbed` (3 plans), `arbor_day`'s `plans` (7), and two missing plans (status 2, both reported). |
| 11 | Press and hold: `click` takes `hold`, the frames between press and release, as `key` does (default 1, as before), for a control that acts while held. DD4.1 says so. First user: `arbor_day`'s HUD camera buttons (its C57). Not copied to `bevy_action_map` (D4) unless asked. Verified: testbed `panel.py` (8), `smoke.json` (17), `typing.py` (9); `arbor_day`'s four plans, `smoke.py` holding each camera button. |
| 12 | A path into the value `expect` compares: `at`, in Bevy's reflection path syntax (`"limbs[0].angle.shape"`) or as a list of keys and indices, walked over the JSON BRP returns, so lists are reached by index rather than compared whole. Client only. First user: `arbor_day`'s C27, whose plans read species fields inside an array of limb levels. Richer entity selectors are X7. Verified: testbed `panel.py` (8), `smoke.json` (17), `typing.py` (9); `arbor_day`'s four plans. |
| 13 | State past the UI: `get` and `get_resource` return a component or resource, or the part at `at`, so a Python plan can take a snapshot; `set` and `set_resource` write one whole (BRP's insert) or one field by reflection path (BRP's mutate), on its own frame as an input lands. A rejected write fails the step, as now does a rejected `message`. For plans about what a setting does rather than how it is edited (`writing-plans.md`, rule 8). Client only. First user: `arbor_day`, whose species is a resource today and a document entity's component later. Verified: testbed `state.py` (14, new), `panel.py` (8), `smoke.json` (17), `typing.py` (9). |
| 14 | The end of a run announced: after the last plan, on macOS a sound (Glass for a pass, Basso otherwise) and a notification giving the count passed, elsewhere the terminal bell, so a developer working elsewhere knows no more windows will come forward; a window closing between plans looked like the end. DD4.3 and `writing-plans.md` (Running a plan) say so. Client only. Not copied to `bevy_action_map` (D4) unless asked. First user: `arbor_day`. |
| 15 | `driver.locate`'s upstream path: `upstream.md` gains "To propose", scrolling into view with `ScrollIntoView` acting on every scrolled ancestor and reflected (X6's ticket); DD8 gains a row for locating a node, which moves to the client once that lands. Docs only. |
| 16 | Bevy main bumped to `2614884` (D2), with `bevy_action_map`, `bevy_immediate` and `arbor_day`; no code changes. `name_path` landed upstream as bevy#26250, so `upstream.md` and DD8 list it as landed and Next gains an item deleting `driver.select`. Scrolling into view is filed as bevy#26262, so `upstream.md` lists it as proposed and X6 is gated on it. Verified: clippy clean; testbed `panel.py` (8), `smoke.json` (17), `state.py` (14), `typing.py` (9); `arbor_day`'s four plans. |
| 17 | A click as a server method (DD5.1), the first half of the input proposal drafted for Bevy on 2026-10-20: `driver.click { entity, button, hold }` or `{ window, position }` queues `CursorMoved`, the press and the release as `WindowEvent`s built from Bevy's types, written one frame apart from `RemoteLast`, and returns `{ frame }`, the frame that reads the release. Scrolling into view takes every hiding ancestor in one pass (DD3.3), shared with `driver.locate`; the click's cursor event waits a frame for layout, so the client's retry and `SCROLL_TRIES` go. `bevy_input` joins the dependencies. Verified: clippy clean, 20 unit tests (seven new); testbed `panel.py` (8), `smoke.json` (17), `state.py` (14), `typing.py` (9); `arbor_day`'s four plans against this checkout. |
| 18 | Upstream status: `ScrollIntoView` (bevy#26262, `fa57bd8a5`) and `input.click` (bevy#26333, `b85a36290`) merged on Bevy main; `input.key` and `input.type` filed as bevy#26342, skipping the plugin. `upstream.md` and DD8 say so; Next item 1 becomes deleting `driver.click` and moving keys and text to BRP; X6's gate is the bump; X1 becomes the move to a Python-only project, gated on Next 1–3, X6 and bevy#26120; X8's drag is proposed upstream beside `input.click`; new X9, clicking a sprite or mesh. Docs only. |
| 19 | `input.key` and `input.type` merged as bevy#26342 (`784b85d36`); `upstream.md` and DD8 list them as landed. Docs only. |
| 20 | Bevy main bumped to `784b85d` (D2), with `bevy_action_map`, `bevy_immediate` and `arbor_day`; no code changes. It brings `input.click`, `input.key`, `input.type` and `ScrollIntoView`, so Next items 1 and 4 (was X6) can start. It is the commit before a feathers regression (bevy#26369; `arbor_day`'s X67). Verified: clippy clean, 20 unit tests; `arbor_day`'s five plans; testbed `panel.py` (8), `smoke.json` (17), `state.py` (14), `typing.py` (9) at `54f38f3`, which differs only in that feathers change. |
| 21 | What a BRP client can see of a queued scene, tested in `tests/scene_readiness.rs`: an asset reader holds the scene's inherited `.bsn` unloaded, and the world is checked after every update, which is what a request in `RemoteLast` sees. A queued scene is seen as nothing, then whole; an entity named outside its scene and `queue_apply_scene`d is seen partial, its name alone, until the scene loads; stepping `SpawnScene`, a look after each of its systems sees nothing, then whole. A `Handle` field alone does not make a scene wait, so `Ready` does not cover the assets a scene names, despite `SceneReady`'s doc. The related-list case is ignored (X10); stepping a scene that waits is not tested (X11). The umbrella dev-dependency gains `bevy_debug_stepping`. Verified: clippy clean on all targets, three tests pass and one is ignored, in parallel and on one thread. |
| 22 | Readiness deleted (was Next 5): `SceneReady`, its observer and `ready.rs` go, with the `bevy_scene` dependency, and `present` loses `ready`. Every step that names an entity now waits for exactly one match (the client's `_one`), so a plan acts on what a step needs without waiting for its screen first. DD3.2 says why no readiness is needed; DD3.1, DD8, `upstream.md` (bevy#26120 under Not needed), DR3.6 and `writing-plans.md` follow. X1 loses bevy#26120 from its gate. New X12 (clicks waiting until a node can take them) and X13 (screenshots waiting for loading). `ready=True` removed from the testbed's four plans and `arbor_day`'s seven. Verified: clippy clean on all targets, 19 unit tests, `scene_readiness` (3, one ignored); testbed `panel.py` (8), `smoke.json` (17), `state.py` (14), `typing.py` (9); `arbor_day`'s five plans, against its git copy of the plugin. Its three `plans/gallery/` plans were edited but not run. |
| 23 | The road to a published Python package as Next items 5–8, across repos: X1 becomes item 5; then packaging, moving `bevy_action_map`, `arbor_day` and the testbed onto the package (retiring D4), and publishing on PyPI at Bevy 0.20's release. X8 and X9 point at item 5. Docs only. |
| 24 | Clicks, keys and text from BRP: the client's `click` calls `input.click`, and `key`, `press`, `release`, `exits` and `type` call `input.key` and `input.type`, through one `_input` that keeps each input a frame after the last. `driver.click` and its queue go (`input.rs`), and so does `driver.locate` (`locate.rs`), which nothing called once `input.click` scrolled into view itself; chunk 27 is folded in. The client's `KeyboardInput` messages and `keys.py`'s US table go. Dependencies drop to `bevy_app`, `bevy_diagnostic`, `bevy_ecs` and `bevy_remote`. DD3.3, DD4.5, DD5.1, DD5.2, DD8 and `upstream.md` follow; no step's arguments changed. Verified: clippy clean on all targets, 8 unit tests, `scene_readiness` (3, one ignored); testbed `panel.py` (8), `smoke.json` (17), `state.py` (14), `typing.py` (9); `arbor_day`'s five plans. |
| 25 | The frame count from BRP: the client's `frame` reads `frame_count` with `diagnostics.get`, failing the step if the diagnostic is missing, and `driver.diagnostics` goes (`diagnostics.rs`, with the `bevy_time` dev-dependency). `FrameTimeDiagnosticsPlugin` is still the driver's to add. `installing_registers_the_methods` also checks the built-in methods the client calls: `diagnostics.get`, `input.click`, `input.key` and `input.type`. DD3.4 and `upstream.md` follow. Verified: clippy clean on all targets, 7 unit tests, `scene_readiness` (3, one ignored); testbed `panel.py` (8), `smoke.json` (17), `state.py` (14), `typing.py` (9); `arbor_day`'s five plans. |
| 26 | Selectors from BRP: the client's `_select` sends `world.query` with a `name_path` filter, refusing an empty selector, and `driver.select` goes (`select.rs`), the plugin's last server method. A number in a list selector is now a child index (DD3.1, X7). A failing step's report still gives each match's full path, which the client builds from every entity's `Name` and `ChildOf` in one query, made only when the step fails (`_shown`; `_wait` takes a report as a function). The plugin is now only setup; `bevy_ecs`, `serde` and `serde_json` go. DD1, DD3.1, DD8, `upstream.md` and `class Driver`'s docstring follow. Verified: clippy clean on all targets, 3 unit tests, `scene_readiness` (3, one ignored); a throwaway testbed plan matched `["Menu", 0]` and reported `absent("Start")`'s match as `Menu/Start`; testbed `panel.py` (8), `smoke.json` (17), `state.py` (14), `typing.py` (9); `arbor_day`'s five plans. |
| 28 | A Python-only project (was X1; D1): the Rust crate goes (`Cargo.toml`, `src/lib.rs` and its three unit tests), and an app enables BRP itself, with the lines `writing-plans.md`'s Setup now gives: `RemotePlugin`, `RemoteHttpPlugin` on `BEVY_REMOTE_DRIVER_PORT`, and `FrameTimeDiagnosticsPlugin`, only when the variable is set (DR1.5). The testbed becomes its own package, `testbed/`, with the setup in its `main`, `tests/scene_readiness.rs`, `Cargo.lock` and its plans (was `plans/testbed/`), which name `bin = "testbed"`. DR1.4, DR1.5, DD1, DD3.4, DD7, DD8, the README, AGENTS.md, D1 and D2 follow; chunk 31's gate is Bevy 0.21, the first release with the input methods. Verified: clippy clean on all targets in `testbed/`, `scene_readiness` (3, one ignored); testbed `panel.py` (8), `smoke.json` (17), `state.py` (14), `typing.py` (9). |
| 29 | A Python package, `bevy-remote-driver`: `client/` becomes `src/bevy_remote_driver/` (`driver.py`, `run.py`, and `__init__.py` exporting `Between` and `Driver`), with `pyproject.toml` on `uv_build`, no dependencies (D5), `requires-python >= 3.11`, and `uv.lock`. The command is `bevy-remote-driver`, installed with `uv tool install git+…`; plans import `from bevy_remote_driver import Between`. 21 `unittest` tests in `tests/` cover selectors, field and reflection paths, `walk`, `subset`, `Between`, the PNG writer and plan loading. CI (`.github/workflows/ci.yml`): ruff 0.17.3, pinned since its default rules grow, and the tests on Python 3.11 and 3.14. ruff's fixes: two explicit `check=False`, one `walk` branch merged, formatting. The README is rewritten for users and PyPI (absolute links): a plan, installing, setting up the app (moved from `writing-plans.md`, which points to it), running plans, a compatibility table, status, licence; CI, licence and Bevy-tracking badges, the PyPI ones left to chunk 31. AGENTS.md, DD4.1, DD4.3 and DD7 follow. `arbor_day`'s plans stop running here, since `client/run.py` and `from driver import` are gone, until chunk 30. Verified: ruff clean; the tests on 3.11 and 3.12; from Panoply's directory, `uvx --from <this checkout> bevy-remote-driver testbed/plans`: `panel.py` (8), `smoke.json` (17), `state.py` (14), `typing.py` (9). Once pushed, `uvx --from git+https://github.com/viridia/bevy_remote_driver bevy-remote-driver` built `3fac10c` and printed its usage. |

| 30 | Users on the package. `arbor_day` (its C165): the git dependency goes, a `RemoteDriverPlugin` of its own in `src/remote_driver.rs` holds the setup lines for the app and its example, and plans import `from bevy_remote_driver import Between`. `bevy_action_map` (its chunk 204): its copy, `crates/bevy_remote_driver`, is deleted with `docs/proposals/remote-driver.md` and six workspace pins only the copy used; the twelve examples include `examples/common/remote_driver.rs`; the `orbit` and `disasteroids` plans move to `plans/`, losing `ready=True`; its scripts drop the `DR` and `DD` anchors, and `xref.py` flags any left; its X33, X34 and X45 go (X3, X4 and D1 here). The copy's client steps were a subset of this one's, so nothing was ported. `expect` now looks again when its entity despawns between the query and the read, as a rebuilt row's does, rather than failing the step on BRP's "not found". D4 retires. Verified: `arbor_day` clippy clean, 144 tests, its five plans, with the command installed by `uv tool install git+…`; `bevy_action_map` `scripts/verify.sh`, and `disasteroids/pad.py` (16), `disasteroids/rebind.py` (21), `orbit/capture.py` (15), and `orbit/editor.py` (32) with this checkout's `expect` (`uv run --project`; `uvx --from <path>` served a cached build). Its `orbit/saved.py` fails: it reads back across launches what `editor.py` saved, which per-plan settings (chunk 7) prevents; chunk 33. |

## Next

31. **Publish on PyPI at Bevy 0.21's release**, the first with the input methods, and the
   compatibility table's first row, superseding D1. Trusted Publishing from a release workflow
   run on a tag, so no token is stored. The author's account, recovered or new, with 2FA; check
   `bevy-remote-driver` is free first. Users' commands become `uvx bevy-remote-driver`. The
   README gains the PyPI version and Python versions badges. *Gate:* Bevy 0.21 released.
32. **Panoply on the package.** Panoply builds an old copy of the plugin and client
   (`crates/bevy_remote_driver`, forked before chunk 1) and runs `plans/smoke.py` with it. It
   takes the setup lines, deletes the copy, and runs its plans with `bevy-remote-driver`, as
   `arbor_day` did in chunk 30. Panoply is on Bevy 0.20.0, which lacks the input methods the
   package calls. *Gate:* Panoply on Bevy 0.21 or on main past `784b85d`.
33. **A restart step.** `driver.restart()` writes `AppExit`, waits for the app to close, and
   launches it again with the same port, environment and settings directory, so a plan checks
   that what the app saved comes back, in one plan, with no order between plans. The launch moves
   from `run_plan` to something the driver can call again, and the report's log carries both
   runs. First user: `bevy_action_map`'s `orbit/editor.py`, which absorbs `saved.py` (its chunk
   205).

## Decisions

- **D1** Unpublished: the client installs from git until chunk 31. Bevy took every server
  method the Rust half had (upstream.md), so since chunk 28 this is a Python-only project, and an
  app enables BRP itself.
- **D2** The testbed's Bevy is main, named by `git` and `branch = "main"`, the commit held by
  `testbed/Cargo.lock`. No user depends on Rust here since chunk 30, so an app's Bevy is its own;
  it needs only the input methods (`784b85d` or later).
- **D3** `run.py` runs cargo in the plan's directory, so the workspace that holds the plan is the
  one built, wherever the command is run from; a plan with no `Cargo.toml` at or above it builds
  the working directory's. Output goes under `cargo metadata`'s `target_directory`, which respects `CARGO_TARGET_DIR`. A plan
  names a `bin` or an `example`, exactly one.
- **D4** Retired in chunk 30. `bevy_action_map` kept its own copy (`crates/bevy_remote_driver`)
  while the driver was Rust and Python together; its plans now run on the package.
- **D5** The client uses Python's standard library only (DD1), so the package has no
  dependencies and installing it brings nothing else.

## Deferred

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
- **X7** Richer entity selectors than a path of `Name`s (DD3.1): an index for the Nth match, or
  a CSS-like syntax, the model Bevy UI authors know best. A child index, a number in the list
  form, came with `name_path` (chunk 26). Value paths are `expect`'s `at`, which needed none of this. *Gate:* a plan that must reach
  an entity it cannot name, such as one row of a list built from data without computed names.
- **X8** Drags, and plans that record a demo or tutorial (the author, 2026-10-26). A drag is
  `input.click`'s press, a `CursorMoved` each frame along a path, and the release, on the same
  queue in `bevy_remote`, which picking turns into drag start, drag and end; proposed upstream
  beside `input.click` rather than in the plugin, so chunk 28 holds. bevy#26366 (open, not ours) adds
  `input.hover`, `input.scroll` and `input.drag { from, to, button }`, four events a frame apart:
  move to `from`, press, move to `to`, release. That covers a test's drag and the turntable's
  zoom, but not a path, a held press (Escape mid-drag) or a recording's pacing, which the sketch
  adds on top. Sketch: `input.click` gains
  `state`, as `input.key` has, to send the press or the release alone; `input.move { entity }` or
  `{ window, position }`, with `frames` (default 1) and `easing`, moves the cursor from where the
  queue left it, one `CursorMoved` a frame, and returns `{ frame }`. A drag is a press, a move and
  a release, with an `input.key` for Escape before the release to cancel one; a move alone tests
  hover. `easing` names an `EaseFunction`, now in `bevy_curve`, which `bevy_remote` would depend
  on with its `serialize` feature (plain variants as names, `"CubicInOut"`); the first PR could
  move linearly and leave easing to a second. A client could write the messages
  itself, a frame apart, but each is a round trip, and a recording's pacing needs every frame's
  position written on that frame. For tests, a few frames in a
  straight line; for a recording, eased in and out (cubic) along a slight arc, its duration
  growing with distance as Fitts's law has it, so short and long drags both read as a hand's.
  Synthetic input moves no OS cursor, so a recording shows the driver's pointer as a cursor image
  the plugin draws when the plan asks, rather than warping the real cursor, which fights the
  user's mouse and needs the window in front. Recorded by capturing each frame in the app
  (Bevy's `Screenshot`, then ffmpeg), for an even frame rate with no focus needed; holds between
  steps, and perhaps captions, for pacing. First users: `arbor_day`'s curve editor, whose point
  drag, rectangle selection and Escape mid-drag are checked by hand, its turntable gestures, and a
  video of handle dragging (its X62). *Gate:* the first plan or video wanting a drag; for a
  test's straight drag, a bump past bevy#26366.
- **X9** Clicking a sprite or a mesh (the author, 2026-10-28). `input.click`'s `entity` is a UI
  node; a world entity needs a camera to project it into the window, and a point other than its
  centre when the centre misses it (a ring, a sprite's transparent middle) or something nearer the
  camera covers it. Sketch: `entity` with an optional `camera`, needed only when more than one
  camera draws to a window, and an optional `point` in the entity's local space, defaulting to
  its `Aabb`'s centre (meshes and sprites both have one). The server projects with
  `Camera::world_to_viewport` and adds the viewport's offset. A local point stays on the entity as
  the camera moves, which a pixel offset does not. Whether the click hit it is picking's answer,
  which a plan can check afterwards in the entity's reflected `PickingInteraction`. Belongs
  upstream beside `input.click`, so chunk 28 holds.
  A place rather than a thing, such as ground to walk to, is the app's to answer (X2), or a
  marker entity it spawns there for the click. *Gate:* the first plan that must click into the
  world. First user: Panoply's actors, whose player clicks an actor or fixture to interact with
  it (its Next item 2).
- **X10** A queued scene list dropped while its dependencies load, reported as bevy#26383.
  `QueuedScenes::spawn_queued` (`bevy_scene/src/spawn.rs`, 811 and 834 at `784b85d`) spawns a
  related or plain scene list once its patch asset exists, which `queue_*` makes at once, rather
  than once it is resolved as a scene's is; the spawn fails ("not been resolved yet") and the
  list is gone. `related_scenes_are_seen_partial` is ignored on it. *Gate:* fixed in the Bevy we
  build against; then un-ignore the test.
- **X11** Stepping `SpawnScene` loses a scene that waits, reported as bevy#26384. Stepped, `resolve_scene_patches` and `spawn_queued` run on alternate frames, and
  `spawn_queued` reads the dependency's `LoadedWithDependencies` before the patch is resolved, so
  the entity waits forever; the inspector's stepping would hit it too. The stepped test uses a
  scene without a dependency. *Gate:* fixed in the Bevy we build against; then step the held
  scene.
- **X12** A click that waits until its node can take it, as Playwright's actionability checks
  do: laid out with a size, visible, enabled (no `InteractionDisabled`), and the topmost hit
  under its centre. Steps already wait for their entity, and `input.click` waits a frame for
  layout and scrolls into view. *Gate:* a plan that fails on a node present but not yet
  clickable.
- **X13** A screenshot that waits for loading to settle, so it does not catch a scene before
  its images: BSN's `Ready` does not wait for a `Handle` a scene holds (DD3.2), and plans wait
  with `seconds` instead. Needs the asset server's load states over BRP, which it does not have.
  *Gate:* a screenshot caught with a texture missing.

## Notes

- Origin: built in `bevy_action_map` from its chunk 145 (a BRP spike) onward; Panoply forked it at
  `bevy_action_map` commit `f641559`. Items 1–4 of Next were that repo's chunks 197, 202, 198 and
  199; X1–X4 were its X45, X32, X33 and X34. Its copy went in chunk 30, and with it those entries,
  but for X32, the mapper's side of X2. `upstream.md` began as the author's gist of
  2026-09-26, a supplement to bevy#25894.
