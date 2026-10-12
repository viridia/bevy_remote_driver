//! `driver.locate`: where a UI node is drawn, so a client can click on it, scrolling it into view
//! first if a scrolled ancestor hides it (DD3.3).

use bevy_camera::{NormalizedRenderTarget, RenderTarget};
use bevy_ecs::{
    entity::{ContainsEntity, Entity},
    hierarchy::ChildOf,
    query::With,
    system::{In, Query, SystemParam},
};
use bevy_math::Vec2;
use bevy_remote::{BrpError, BrpResult, builtin_methods::parse_some, error_codes};
use bevy_ui::{
    ComputedNode, ComputedUiTargetCamera, Node, OverflowAxis, ScrollPosition, UiGlobalTransform,
};
use bevy_window::{PrimaryWindow, Window};
use serde_json::{Value, json};

use crate::select::{PathParams, Selector};

pub(crate) const METHOD: &str = "driver.locate";

fn refused(message: String) -> BrpError {
    BrpError {
        code: error_codes::INVALID_PARAMS,
        message,
        data: None,
    }
}

pub(crate) fn process_request(
    In(params): In<Option<Value>>,
    selector: Selector,
    nodes: Query<(
        &UiGlobalTransform,
        &ComputedUiTargetCamera,
        Option<&ComputedNode>,
    )>,
    mut views: ScrollViews,
    targets: Query<&RenderTarget>,
    primary: Query<Entity, With<PrimaryWindow>>,
    windows: Query<&Window>,
) -> BrpResult {
    let PathParams { path } = parse_some(params)?;
    let shown = path.join("/");
    let entity = selector.select_one(&path)?;
    let Ok((transform, camera, computed)) = nodes.get(entity) else {
        return Err(refused(format!("{shown} is not a UI node")));
    };
    let size = computed.map_or(Vec2::ZERO, ComputedNode::size);
    if views.scroll_into_view(entity, transform.translation, size) {
        // Layout moves the node on a later frame; the client asks again.
        return Ok(json!({ "scrolled": true }));
    }
    let primary = primary.single().ok();
    let window = match camera.get() {
        Some(camera) => match targets.get(camera).ok().and_then(|t| t.normalize(primary)) {
            Some(NormalizedRenderTarget::Window(window)) => Some(window.entity()),
            _ => return Err(refused(format!("{shown} is not drawn to a window"))),
        },
        None => primary,
    };
    let Some(window) = window.filter(|&w| windows.contains(w)) else {
        return Err(refused(format!("{shown} has no window to be drawn in")));
    };
    // Physical pixels, as the transform and `WindowEvent::CursorMoved` both are.
    let physical = transform.translation;
    Ok(json!({ "window": window, "physical": [physical.x, physical.y] }))
}

/// The nodes that scroll their content, and the hierarchy to find a node's.
#[derive(SystemParam)]
pub(crate) struct ScrollViews<'w, 's> {
    parents: Query<'w, 's, &'static ChildOf>,
    views: Query<
        'w,
        's,
        (
            &'static Node,
            &'static ComputedNode,
            &'static UiGlobalTransform,
            &'static mut ScrollPosition,
        ),
    >,
}

impl ScrollViews<'_, '_> {
    /// Scrolls the nearest ancestor that hides the node, centred at `center` and `size` big (both
    /// physical pixels), by the least that shows all of it, or its start if it is bigger than the
    /// view. Returns whether anything scrolled. Only the nearest such ancestor moves: scrolling it
    /// moves the node, so any further ancestor is judged on the next call, after layout.
    fn scroll_into_view(&mut self, entity: Entity, center: Vec2, size: Vec2) -> bool {
        let (start, end) = (center - size * 0.5, center + size * 0.5);
        for ancestor in self.parents.iter_ancestors(entity) {
            let Ok((node, view, view_transform, mut position)) = self.views.get_mut(ancestor)
            else {
                continue;
            };
            let view_start = view_transform.translation - view.size() * 0.5;
            let view_end = view_start + view.size();
            let axes = [node.overflow.x, node.overflow.y].map(|axis| axis == OverflowAxis::Scroll);
            let mut delta = Vec2::ZERO;
            for i in 0..2 {
                if !axes[i] {
                    continue;
                }
                if start[i] < view_start[i] {
                    delta[i] = start[i] - view_start[i];
                } else if end[i] > view_end[i] {
                    delta[i] = (end[i] - view_end[i]).min(start[i] - view_start[i]);
                }
            }
            if delta == Vec2::ZERO {
                continue;
            }
            // `ScrollPosition` is in logical pixels; layout is physical.
            let scale = view.inverse_scale_factor();
            let limit = (view.content_size() - view.size()).max(Vec2::ZERO) * scale;
            let target = (position.0 + delta * scale).clamp(Vec2::ZERO, limit);
            if target == position.0 {
                // Scrolled as far as it goes; a further ancestor may still help.
                continue;
            }
            position.0 = target;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_ecs::{name::Name, world::World};
    use bevy_ui::Overflow;
    use bevy_window::WindowResolution;

    fn locate(world: &mut World, path: &str) -> BrpResult {
        world
            .run_system_cached_with(process_request, Some(json!({ "path": [path] })))
            .unwrap()
    }

    #[test]
    fn the_centre_is_reported_in_physical_pixels() {
        let mut world = World::new();
        let resolution = WindowResolution::new(1600, 1200).with_scale_factor_override(2.0);
        let window = world
            .spawn((
                Window {
                    resolution,
                    ..Default::default()
                },
                PrimaryWindow,
            ))
            .id();
        world.spawn((
            Name::new("Start"),
            UiGlobalTransform::from_translation(Vec2::new(400.0, 300.0)),
            ComputedUiTargetCamera::default(),
        ));

        let found = locate(&mut world, "Start").unwrap();

        assert_eq!(found["window"], json!(window));
        assert_eq!(found["physical"], json!([400.0, 300.0]));
    }

    #[test]
    fn a_node_scrolled_out_of_view_is_scrolled_back_first() {
        let mut world = World::new();
        world.spawn((Window::default(), PrimaryWindow));
        // A view 100 px tall at the top of the window, over 400 px of content.
        let view = world
            .spawn((
                Node {
                    overflow: Overflow::scroll_y(),
                    ..Default::default()
                },
                ComputedNode {
                    size: Vec2::new(200.0, 100.0),
                    content_size: Vec2::new(200.0, 400.0),
                    inverse_scale_factor: 1.0,
                    ..Default::default()
                },
                UiGlobalTransform::from_translation(Vec2::new(100.0, 50.0)),
                ScrollPosition::default(),
            ))
            .id();
        // 20 px tall, 240 px down.
        world.spawn((
            Name::new("Start"),
            ChildOf(view),
            ComputedNode {
                size: Vec2::new(200.0, 20.0),
                inverse_scale_factor: 1.0,
                ..Default::default()
            },
            UiGlobalTransform::from_translation(Vec2::new(100.0, 250.0)),
            ComputedUiTargetCamera::default(),
        ));

        let found = locate(&mut world, "Start").unwrap();

        assert_eq!(found, json!({ "scrolled": true }));
        // Just far enough that its bottom edge meets the view's.
        let position = world.get::<ScrollPosition>(view).unwrap();
        assert_eq!(position.0, Vec2::new(0.0, 160.0));
    }

    #[test]
    fn an_entity_that_is_not_a_node_is_refused() {
        let mut world = World::new();
        world.spawn((Window::default(), PrimaryWindow));
        world.spawn(Name::new("Start"));

        let err = locate(&mut world, "Start").unwrap_err();

        assert!(err.message.contains("not a UI node"), "{}", err.message);
    }

    #[test]
    fn several_matches_are_listed_in_the_error() {
        let mut world = World::new();
        world.spawn(Name::new("Start"));
        world.spawn(Name::new("Start"));

        let err = locate(&mut world, "Start").unwrap_err();

        assert!(
            err.message.contains("matched 2 entities"),
            "{}",
            err.message
        );
        assert_eq!(err.data.unwrap().as_array().unwrap().len(), 2);
    }
}
