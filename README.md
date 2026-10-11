# bevy_remote_driver

Drive a running Bevy app from outside, for end-to-end tests, over the Bevy Remote Protocol.

- **The plugin**: add `RemoteDriverPlugin` to the app. It does nothing unless
  `BEVY_REMOTE_DRIVER_PORT` is set, so it can stay in normal builds.
- **The client**: a Python script, standard library only, that builds the app, launches it, runs a
  plan's steps (select entities by `Name` path, wait for conditions, press keys, click UI nodes,
  take screenshots), reports, and closes the app.

```sh
# from the Cargo workspace holding the app
python3 ../bevy_remote_driver/client/run.py plans/smoke.py
```

A plan is a Python module (`BIN` or `EXAMPLE`, optionally `PACKAGE` and `FEATURES`, and
`run(driver)`) or a JSON list of steps. See [docs/design.md](docs/design.md), section 9, for
writing one.

## Status

Not published (`publish = false`): use it by git dependency. The Rust half is proposed for Bevy
itself ([docs/proposal.md](docs/proposal.md)); once Bevy takes it, this becomes a Python-only
project. It builds against Bevy main; see [docs/ledger.md](docs/ledger.md).

```toml
bevy_remote_driver = { git = "https://github.com/viridia/bevy_remote_driver" }
```

`bevy_action_map` carries its own copy, so its tests run from that repository alone.

## License

MIT or Apache-2.0, at your option.
