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

## Proposed

- **Selecting entities by a path of names**
  ([bevy#26121](https://github.com/bevyengine/bevy/issues/26121)): a `name_path` filter on
  `world.query`. The driver's `driver.select` method does this today (DD3.1).
- **Readiness as state** ([bevy#26120](https://github.com/bevyengine/bevy/issues/26120)):
  `bevy_scene` leaves a reflected `SceneReady` marker on each entity it triggers `Ready` for. The
  driver's global observer does this today (DD3.2).
