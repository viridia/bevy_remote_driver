# What Bevy could take

End-to-end tests drive a running app over the Bevy Remote Protocol (BRP). This driver needs a
plugin today only because BRP lacks a few things. With them in Bevy, a test client in any language
could drive any app with no plugin at all. Discussion: [bevy#25894](https://github.com/bevyengine/bevy/issues/25894).

## Landed

- **Diagnostics over BRP** ([bevy#25824](https://github.com/bevyengine/bevy/pull/25824)):
  `diagnostics.get` reads `frame_count`, which every wait measured in frames needs. The driver's
  own `driver.diagnostics` goes once the client switches (the ledger's Next).
- **Clicks in physical pixels** ([bevy#25890](https://github.com/bevyengine/bevy/pull/25890)):
  `CursorMoved` takes a `physical_position`, which is what `UiGlobalTransform` holds, so a client
  no longer divides by the scale factor.
- **Writable gamepad messages** ([bevy#25904](https://github.com/bevyengine/bevy/pull/25904)): a
  client can connect a virtual pad and send its buttons and axes with `world.write_message`.

## Proposed

### Selecting entities by a path of names

A test refers to things by name ("the Rebind button in the Jump row"), and `world.query` filters
only by component type. Finding one entity from outside today means fetching every `Name`, then
walking `ChildOf` chains from the client, a round trip per ancestor.

A `name_path` filter on `world.query`:

```json
{ "filter": { "name_path": ["Settings", "Jump", "Rebind"] } }
```

- The last name matches an entity's `Name` exactly.
- Each earlier name matches one of its ancestors, in order from the root down, at any depth.
  Unnamed entities in between are skipped, so a scene names only what a test refers to.
- An array, not a `/`-separated string, so a name may contain any character and the server parses
  nothing.

The result is an ordinary query result, combining with `with` and `without`. A test that wants one
entity treats zero or several matches as its failure, and the driver's `driver.select` also returns
each match's full name path, so the failure can say which match was which. Whether a query should
return that is open.

### Readiness as state

A test has to wait for a scene to finish spawning before touching it. `bevy_scene` announces that
with `Ready`, an entity event: a client that starts watching after the scene spawned misses it, and
deriving `Reflect` on `Ready` would not change that.

`bevy_scene` leaves a reflected marker component on each entity it triggers `Ready` for, so
readiness is state a query sees whenever it looks. The driver does this with a global observer and
calls the marker `SceneReady`. If a component on every scene entity is too much to impose, it can be
opt-in, on a setting of the scene plugin.

`Ready` covers the assets a scene names. An asset requested after the scene spawns, such as an image
a system loads later, is not covered, and a test waits for it by time.

## Under consideration

### Input as server methods

Every client writes input as BRP messages (`WindowEvent`, `KeyboardInput`) and derives what a real
device would add: a node's position for a click, a key's `logical_key` and `text` for a US layout.
Each client repeats that work, and each breaks at runtime when a message's shape changes, as clicks
did with bevy#25890.

The driver is trying methods that take the intent and build the messages from Bevy's own types, so
a change of shape is a compile error: `driver.click { path }` first, then `driver.key` and
`driver.type`. If they work out, they are candidates here too.
