# Agent instructions

`bevy_remote_driver` drives a running Bevy app from outside, over the Bevy Remote Protocol, for
end-to-end tests: a Python client that builds the app, launches it, runs a plan's steps, and
reports. The app enables BRP itself when `BEVY_REMOTE_DRIVER_PORT` is set. `testbed/` is the Rust
app the driver's own plans run against.

## Orient

1. Read `docs/ledger.md`: what has landed, what is next, decisions, deferrals.
2. Check `git log --oneline -5`.
3. `docs/requirements.md` numbers the requirements `DR<section>.<n>`; `docs/design.md` explains
   how the driver works, in sections `DD<n>`. Cite them by number.

## Rules

- Never commit, push, or stage. The author reviews and commits; draft the commit message: a
  short subject line (`feat:` / `fix:` / `refactor:` / `chore:` / `docs:` prefix, under about 72
  characters), a blank line, then a body of bullets, one per change, wrapped at 72 columns.
- One chunk per session: one focused change. Update the ledger in the same chunk (Landed row,
  Next, new decisions or deferrals). A Next item is numbered as a chunk, after the highest
  number used, and keeps its number when it lands. Chunk numbers are plain, cited as "chunk
  18", or "the driver's chunk 18" from another repo.
- Every open item goes in the ledger: in Next, or in Deferred with a gate.
- Propose structural changes (new dependencies, a new server method, a change to the plan
  format) with a recommendation before writing code.
- Bevy is main, named by branch (see `testbed/Cargo.toml`). Read APIs in the local checkout
  `../bevy` at the commit `testbed/Cargo.lock` holds, and its
  `_release-content/migration-guides/`, rather than guessing.
- User docs have one home each: a step's behaviour in its docstring in `client/driver.py`
  (selectors, timeouts and the JSON form in `class Driver`'s), app setup and the workflow in
  `docs/writing-plans.md`. A change updates that home, and nothing else repeats it.
- Prose follows the "House style" section of `../bevy_action_map/CLAUDE.md`. A user-facing doc
  (README, `writing-plans.md`, docstrings) gets a separate polish pass once its content is
  settled.
- Users: `arbor_day` runs the client from a checkout, and depended on the old Rust crate by git
  until it moves (chunk 30). Panoply has a copy of its own (`crates/bevy_remote_driver`).
  `bevy_action_map` keeps a copy too, as a published crate whose tests others may run; a fix
  here that it needs is copied across by hand.

## Verify

- In `testbed/`: `cargo clippy --all-targets`, no errors and no new warnings; `cargo test`.
- End to end, from this directory:
  `python3 client/run.py testbed/plans`, which runs every testbed plan.
  Output goes to `remote-driver/` in Cargo's target directory. Screenshots bring the app window to
  the front.
