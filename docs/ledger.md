# bevy_remote_driver ledger

What has landed, what is next, the decisions that stand, and what is deferred and why. Requirements
are [requirements.md](requirements.md)'s `DR` numbers; the design is [design.md](design.md)'s `DD`
sections. Decisions here are `D<n>`, deferrals `X<n>`.

## Landed

| Commit | Change |
| --- | --- |
| (pending) | Extracted into its own repo (2026-10-11). Union of two copies: `bevy_action_map`'s `crates/bevy_remote_driver` at chunk 196 (Bevy main; `driver.locate` returns physical pixels, since `CursorMoved` takes a `physical_position`), and Panoply's (`bin` targets in `run.py`; `message` and `exits` steps). `run.py` runs from the project's directory, not from inside it. |

## Next

1. **A click as a server method.** bevy#25894 wants a runner in any language that speaks HTTP and
   JSON, so whatever a runner builds for itself, every runner builds again. A click today is the
   client writing `WindowEvent` over BRP, a window backend's message rather than a contract for
   tests: bevy#25890 reshaped it, and every click failed at runtime until the client was patched.
   - `driver.click { path }` resolves the node as `driver.locate` does, and queues `CursorMoved`,
     the press and the release into a resource the plugin drains one per frame. It returns once
     queued; the client waits for the frames as it does now. A refusal still comes back from the
     call.
   - The events are built from Bevy's types, so a change to their shape is a compile error.
   - `driver.locate` stays, for a plan that wants a node's position without clicking it.
   - DD8's row for clicks moves from the client to the plugin. [proposal.md](proposal.md) is the
     author's to update.
   - Not doing: keys and text (item 2); gamepads.
   - Verified by a headless `App` test that one call yields the three events on three frames, and
     `plans/testbed/smoke.json` end to end.
2. **Keys and text as server methods**, after item 1, for the same reason. `driver.py` writes
   `KeyboardInput` over BRP and derives `logical_key` and `text` from the key code, for a US layout
   (DD5.2), so every runner re-derives them and breaks when the message changes. `driver.key` and
   `driver.type` take a key code or a string, and the plugin builds the messages from Bevy's types.
   The client's `exits` step goes through it too. Not doing: other layouts; gamepads. Verified by
   a headless `App` test of the derived messages, including text, and `plans/testbed/typing.py`.
3. **Delete `driver.diagnostics`.** Past bevy#25824, BRP's own `diagnostics.get` reads
   `frame_count` as `driver.diagnostics` does (DD3.4). The client switches to it and the method
   goes. `FrameTimeDiagnosticsPlugin` is still the driver's to add, since it records the count and
   is not in `DefaultPlugins`. Verified by any plan end to end, since every wait reads the count.
4. **A run isolated from what the developer has saved.** A plan that changes settings writes the
   developer's real settings file, and one that fails halfway leaves it dirty. Past bevy#25902,
   `preferences_dir` honours an absolute `BEVY_SETTINGS_DIR` on every platform. `environment()` in
   `run.py` sets it to a directory under the run's own output.
   - The run checks the directory was written: a wrong path or variable fails silently, and the
     plan passes while reading the real file.
   - Not doing: a per-run `app_name`, which would need the app to read an env var.
   - Users: Panoply can then drop its separate settings identity under the driver (its D15).
   - Verified by a plan run twice, with the real settings file's modification time unchanged.
5. **Make the docs stand alone.** `design.md` was written inside `bevy_action_map` and still
   leans on its examples (`minimal`, `disasteroids`) and its remote methods (DD6, DD7). Rewrite
   the worked examples against `examples/testbed.rs`, keep `bevy_action_map` as one example of an
   app adding its own methods, and move what only concerns that crate back to it.

## Decisions

- **D1** Unpublished (`publish = false`). Rust and a Python client together do not ship as a
  crate. Users depend on it by git. The Rust half is meant for Bevy (proposal.md); when Bevy has
  taken it, this repo becomes a Python-only project.
- **D2** Bevy is main, named by `git` and `branch = "main"`, the commit held by `Cargo.lock`. Every
  repo using the driver names Bevy the same way, since Cargo treats a branch and a rev as different
  sources and would build two Bevys. Bump all users together.
- **D3** `run.py` runs from the Cargo workspace that holds the target, not from inside it: cargo
  builds there, and output goes under `cargo metadata`'s `target_directory`, which respects
  `CARGO_TARGET_DIR`. A plan names a `bin` or an `example`, exactly one.
- **D4** `bevy_action_map` keeps its own copy (`crates/bevy_remote_driver`), since it is a published
  crate whose tests others may want to run without a second repo. This repo is the canonical copy;
  a fix that crate needs is copied across by hand, and the reverse.
- **D5** The client uses Python's standard library only (DD1), so a plan runs with nothing
  installed.

## Deferred

- **X1** What the driver becomes. The proposal (proposal.md, posted by the author as a gist on
  2026-09-26) says which server methods Bevy should take. What Bevy declines stays here; the
  client stays here either way. *Gate:* the proposal answered.
- **X2** A plan step for an app's own methods, or a Python helper over `call`. Both extension
  points exist: BRP registration in Rust, and a Python plan is a program (DD4.2) that can import a
  helper. A step registry would be a third mechanism where two already reach. *Gate:* a second plan
  whose raw `call` steps are unreadable.
- **X3** A virtual pad that says which pad it is. The connection event carries `name`,
  `vendor_id` and `product_id`, and the client fills all three with a pad no vendor table knows,
  so every machine sees the same fallback. Exposing them is three optional arguments. *Gate:* a
  plan whose subject is brand-specific presentation.
- **X4** A second virtual pad, and disconnecting one. The client holds one pad's entity and
  connects it on first use; a second is another entity and a `pad` step that says which, and a
  disconnect is one more message. *Gate:* a plan driving two players, or one whose subject is a
  pad going away.

## Notes

- Origin: built in `bevy_action_map` from its chunk 145 (a BRP spike) onward; Panoply forked it at
  `bevy_action_map` commit `f641559`. Items 1–4 of Next were that repo's chunks 197, 202, 198 and
  199; X1–X4 were its X45, X32, X33 and X34. `bevy_action_map`'s copy stays (D4), so those entries
  may stay in its roadmap for its own copy.
