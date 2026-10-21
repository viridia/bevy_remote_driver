//! `driver.click`: a click on a UI node, or at a point in a window, written as a window backend
//! writes one (DD5.1). Its events wait in [`InputQueue`] and are written one frame apart.

use std::collections::VecDeque;

use bevy_diagnostic::FrameCount;
use bevy_ecs::{
    entity::Entity,
    message::MessageWriter,
    resource::Resource,
    system::{In, Res, ResMut},
};
use bevy_input::{
    ButtonState,
    mouse::{MouseButton, MouseButtonInput},
};
use bevy_math::Vec2;
use bevy_remote::{BrpResult, builtin_methods::parse_some};
use bevy_window::{RawCursorMoved, WindowEvent};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::locate::{ScrollViews, UiNodes, refused};

pub(crate) const CLICK: &str = "driver.click";

/// Input waiting to be written, each event with the frame that reads it, in order.
#[derive(Resource, Default)]
pub(crate) struct InputQueue {
    events: VecDeque<(u32, Queued)>,
    /// The frame that reads the last event queued.
    last: u32,
}

enum Queued {
    /// The cursor moves to `node` as it is drawn by then, or to `position` without one or once
    /// the node has gone.
    Cursor {
        window: Entity,
        node: Option<Entity>,
        position: Vec2,
    },
    Button {
        window: Entity,
        button: MouseButton,
        state: ButtonState,
    },
}

impl InputQueue {
    /// Queues `event` to be read `gap` frames after the last event queued, and not before
    /// `earliest`. Returns the frame that reads it.
    fn push(&mut self, earliest: u32, gap: u32, event: Queued) -> u32 {
        let frame = earliest.max(self.last + gap);
        self.events.push_back((frame, event));
        self.last = frame;
        frame
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClickParams {
    entity: Option<Entity>,
    window: Option<Entity>,
    position: Option<[f32; 2]>,
    #[serde(default)]
    button: Button,
    #[serde(default = "one")]
    hold: u32,
}

fn one() -> u32 {
    1
}

#[derive(Deserialize, Default, Clone, Copy)]
enum Button {
    #[default]
    Left,
    Right,
    Middle,
    Back,
    Forward,
}

impl From<Button> for MouseButton {
    fn from(button: Button) -> Self {
        match button {
            Button::Left => MouseButton::Left,
            Button::Right => MouseButton::Right,
            Button::Middle => MouseButton::Middle,
            Button::Back => MouseButton::Back,
            Button::Forward => MouseButton::Forward,
        }
    }
}

pub(crate) fn process_click(
    In(params): In<Option<Value>>,
    nodes: UiNodes,
    mut views: ScrollViews,
    mut queue: ResMut<InputQueue>,
    frame: Res<FrameCount>,
) -> BrpResult {
    let ClickParams {
        entity,
        window,
        position,
        button,
        hold,
    } = parse_some(params)?;
    if hold == 0 {
        return Err(refused("hold is at least one frame".into()));
    }
    // BRP handles requests after `Last`, where `FrameCount` advances, so it already counts the
    // frame that reads what is written now.
    let mut earliest = frame.0;
    let (window, node, position) = match (entity, window, position) {
        (Some(entity), None, None) => {
            let node = nodes.locate(entity, &entity.to_string())?;
            if views.scroll_into_view(entity, node.center, node.size) {
                // Layout moves the node on the next frame; the cursor follows it there.
                earliest += 1;
            }
            (node.window, Some(entity), node.center)
        }
        (None, Some(window), Some([x, y])) => {
            if !nodes.is_window(window) {
                return Err(refused(format!("{window} is not a window")));
            }
            (window, None, Vec2::new(x, y))
        }
        _ => {
            return Err(refused(
                "a click takes an entity, or a window and a position".into(),
            ));
        }
    };
    let button = MouseButton::from(button);
    let cursor = Queued::Cursor {
        window,
        node,
        position,
    };
    let press = Queued::Button {
        window,
        button,
        state: ButtonState::Pressed,
    };
    let release = Queued::Button {
        window,
        button,
        state: ButtonState::Released,
    };
    queue.push(earliest, 1, cursor);
    queue.push(earliest, 1, press);
    let frame = queue.push(earliest, hold, release);
    Ok(json!({ "frame": frame }))
}

/// Writes the events that are due as `WindowEvent`s, which picking reads in `First` and
/// `bevy_window` splits into typed messages in `PreUpdate`.
pub(crate) fn write_due(
    mut queue: ResMut<InputQueue>,
    frame: Res<FrameCount>,
    nodes: UiNodes,
    mut window_events: MessageWriter<WindowEvent>,
) {
    while queue.events.front().is_some_and(|(due, _)| *due <= frame.0) {
        let Some((_, event)) = queue.events.pop_front() else {
            break;
        };
        window_events.write(match event {
            Queued::Cursor {
                window,
                node,
                position,
            } => {
                let position = node.and_then(|node| nodes.center(node)).unwrap_or(position);
                WindowEvent::CursorMoved(RawCursorMoved {
                    window,
                    physical_position: position.as_dvec2(),
                })
            }
            Queued::Button {
                window,
                button,
                state,
            } => WindowEvent::MouseButtonInput(MouseButtonInput {
                button,
                state,
                window,
            }),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::locate::tests::{row, scroll_view};
    use bevy_ecs::{message::Messages, world::World};
    use bevy_ui::{ComputedUiTargetCamera, UiGlobalTransform};
    use bevy_window::{PrimaryWindow, Window};

    /// A world at frame 10, with a primary window.
    fn world() -> (World, Entity) {
        let mut world = World::new();
        world.init_resource::<InputQueue>();
        world.init_resource::<Messages<WindowEvent>>();
        world.insert_resource(FrameCount(10));
        let window = world.spawn((Window::default(), PrimaryWindow)).id();
        (world, window)
    }

    fn node(world: &mut World, x: f32, y: f32) -> Entity {
        world
            .spawn((
                UiGlobalTransform::from_translation(Vec2::new(x, y)),
                ComputedUiTargetCamera::default(),
            ))
            .id()
    }

    fn click(world: &mut World, params: Value) -> BrpResult {
        world
            .run_system_cached_with(process_click, Some(params))
            .unwrap()
    }

    /// Runs frames from the current one through `last`, and returns what each wrote, with the
    /// frame that reads it.
    fn frames_through(world: &mut World, last: u32) -> Vec<(u32, String)> {
        let mut written = Vec::new();
        loop {
            let frame = world.resource::<FrameCount>().0;
            world.run_system_cached(write_due).unwrap();
            let events = world
                .resource_mut::<Messages<WindowEvent>>()
                .drain()
                .collect::<Vec<_>>();
            written.extend(events.into_iter().map(|event| (frame, described(event))));
            if frame >= last {
                return written;
            }
            world.resource_mut::<FrameCount>().0 += 1;
        }
    }

    fn described(event: WindowEvent) -> String {
        match event {
            WindowEvent::CursorMoved(moved) => {
                let at = moved.physical_position;
                format!("move {} {}", at.x, at.y)
            }
            WindowEvent::MouseButtonInput(input) => format!("{:?} {:?}", input.state, input.button),
            other => format!("{other:?}"),
        }
    }

    fn events(list: &[(u32, &str)]) -> Vec<(u32, String)> {
        list.iter()
            .map(|&(frame, event)| (frame, event.to_string()))
            .collect()
    }

    #[test]
    fn a_click_is_three_events_on_three_frames() {
        let (mut world, _) = world();
        let start = node(&mut world, 400.0, 300.0);

        let found = click(&mut world, json!({ "entity": start })).unwrap();

        assert_eq!(found, json!({ "frame": 12 }));
        assert_eq!(
            frames_through(&mut world, 13),
            events(&[
                (10, "move 400 300"),
                (11, "Pressed Left"),
                (12, "Released Left")
            ])
        );
    }

    #[test]
    fn hold_is_the_frames_from_press_to_release() {
        let (mut world, _) = world();
        let start = node(&mut world, 400.0, 300.0);

        let found = click(&mut world, json!({ "entity": start, "hold": 3 })).unwrap();

        assert_eq!(found, json!({ "frame": 14 }));
        assert_eq!(
            frames_through(&mut world, 14),
            events(&[
                (10, "move 400 300"),
                (11, "Pressed Left"),
                (14, "Released Left")
            ])
        );
    }

    #[test]
    fn clicks_queue_one_after_another() {
        let (mut world, _) = world();
        let start = node(&mut world, 400.0, 300.0);
        let quit = node(&mut world, 400.0, 500.0);

        click(&mut world, json!({ "entity": start })).unwrap();
        let found = click(&mut world, json!({ "entity": quit })).unwrap();

        assert_eq!(found, json!({ "frame": 15 }));
        let written = frames_through(&mut world, 15);
        assert_eq!(
            written[3..],
            events(&[
                (13, "move 400 500"),
                (14, "Pressed Left"),
                (15, "Released Left")
            ])
        );
    }

    #[test]
    fn a_point_in_a_window_is_clicked_with_any_button() {
        let (mut world, window) = world();

        let params = json!({ "window": window, "position": [5.0, 6.0], "button": "Right" });
        click(&mut world, params).unwrap();

        assert_eq!(
            frames_through(&mut world, 12),
            events(&[
                (10, "move 5 6"),
                (11, "Pressed Right"),
                (12, "Released Right")
            ])
        );
    }

    #[test]
    fn a_scrolled_node_is_clicked_where_layout_moves_it() {
        let (mut world, _) = world();
        let view = scroll_view(&mut world, 0.0, 100.0, 400.0);
        let start = row(&mut world, "Start", view, 240.0);

        let found = click(&mut world, json!({ "entity": start })).unwrap();
        // What layout does on frame 10, having seen the scroll: the row's bottom meets the view's.
        frames_through(&mut world, 10);
        world
            .entity_mut(start)
            .insert(UiGlobalTransform::from_translation(Vec2::new(100.0, 90.0)));
        world.resource_mut::<FrameCount>().0 += 1;

        assert_eq!(found, json!({ "frame": 13 }));
        assert_eq!(
            frames_through(&mut world, 13),
            events(&[
                (11, "move 100 90"),
                (12, "Pressed Left"),
                (13, "Released Left")
            ])
        );
    }

    #[test]
    fn a_click_needs_one_target_and_a_hold() {
        let (mut world, window) = world();
        let start = node(&mut world, 400.0, 300.0);
        let other = world.spawn_empty().id();

        for params in [
            json!({}),
            json!({ "entity": start, "window": window, "position": [1.0, 1.0] }),
            json!({ "window": window }),
            json!({ "entity": start, "hold": 0 }),
            json!({ "entity": other }),
            json!({ "window": other, "position": [1.0, 1.0] }),
            json!({ "entity": start, "button": "Thumb" }),
        ] {
            assert!(click(&mut world, params.clone()).is_err(), "{params}");
        }
        assert!(frames_through(&mut world, 12).is_empty());
    }
}
