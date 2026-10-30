# What Bevy could take

End-to-end tests drive a running app over the Bevy Remote Protocol (BRP). This driver needs a
plugin today only because BRP lacks a few things. With them in Bevy, a test client in any language
could drive any app with no plugin at all.

Discussion: [bevy#25894](https://github.com/bevyengine/bevy/issues/25894).

## Landed

- **Diagnostics over BRP** ([bevy#25824](https://github.com/bevyengine/bevy/pull/25824)):
  `diagnostics.get` reads `frame_count`, which every wait measured in frames needs. The driver's
  own `driver.diagnostics` is due to be removed in its favour.
- **Clicks in physical pixels** ([bevy#25890](https://github.com/bevyengine/bevy/pull/25890)):
  `CursorMoved` takes a `physical_position`, which is what `UiGlobalTransform` holds, so a client
  no longer divides by the scale factor.
- **Writable gamepad messages** ([bevy#25904](https://github.com/bevyengine/bevy/pull/25904)): a
  client can connect a virtual pad and send its buttons and axes with `world.write_message`.
- **Selecting entities by a path of names**
  ([bevy#26250](https://github.com/bevyengine/bevy/pull/26250)): `world.query` takes a
  `name_path`. The driver's own `driver.select` (DD3.1) is due to be removed in its favour.
- **Scrolling a node into view on request**
  ([bevy#26262](https://github.com/bevyengine/bevy/pull/26262)): `ScrollIntoView` is in
  `bevy_ui`, reflected, so a client can send it with `world.trigger_event`, and scrolls every
  `overflow: scroll` ancestor, not only the nearest `ScrollArea`. `input.click` uses it (DD3.3).
- **Clicks as a server method** ([bevy#26333](https://github.com/bevyengine/bevy/pull/26333)):
  `input.click` takes a UI node or a window point, scrolls a hidden node into view, and returns
  the frame that reads the release. The client's `click` calls it (DD5.1).
- **Keys and text as server methods**
  ([bevy#26342](https://github.com/bevyengine/bevy/pull/26342)): `input.key` and `input.type`, on
  `input.click`'s queue. The client's keys and `type` call them (DD5.2).

## Not needed

- **Readiness as state** ([bevy#26120](https://github.com/bevyengine/bevy/issues/26120)): a
  reflected marker on each entity `bevy_scene` triggers `Ready` for, so a client could wait for a
  scene to finish spawning. A BRP request sees a scene whole or not at all (DD3.2), so a client
  waits for the entity it needs by name instead.
