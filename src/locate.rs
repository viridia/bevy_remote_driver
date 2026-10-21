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

pub(crate) fn refused(message: String) -> BrpError {
    BrpError {
        code: error_codes::INVALID_PARAMS,
        message,
        data: None,
    }
}

pub(crate) fn process_request(
    In(params): In<Option<Value>>,
    selector: Selector,
    nodes: UiNodes,
    mut views: ScrollViews,
) -> BrpResult {
    let PathParams { path } = parse_some(params)?;
    let shown = path.join("/");
    let entity = selector.select_one(&path)?;
    let node = nodes.locate(entity, &shown)?;
    if views.scroll_into_view(entity, node.center, node.size) {
        // Layout moves the node on a later frame; the client asks again.
        return Ok(json!({ "scrolled": true }));
    }
    let physical = node.center;
    Ok(json!({ "window": node.window, "physical": [physical.x, physical.y] }))
}

/// A UI node as it stands: the window it is drawn in, and its centre and size in physical pixels,
/// as the transform and `WindowEvent::CursorMoved` both are.
pub(crate) struct Located {
    pub window: Entity,
    pub center: Vec2,
    pub size: Vec2,
}

/// UI nodes, and the cameras and windows they are drawn to.
#[derive(SystemParam)]
pub(crate) struct UiNodes<'w, 's> {
    nodes: Query<
        'w,
        's,
        (
            &'static UiGlobalTransform,
            &'static ComputedUiTargetCamera,
            Option<&'static ComputedNode>,
        ),
    >,
    targets: Query<'w, 's, &'static RenderTarget>,
    primary: Query<'w, 's, Entity, With<PrimaryWindow>>,
    windows: Query<'w, 's, &'static Window>,
}

impl UiNodes<'_, '_> {
    /// Where `entity` is drawn, or why it cannot be clicked; `shown` names it in the error.
    pub fn locate(&self, entity: Entity, shown: &str) -> Result<Located, BrpError> {
        let Ok((transform, camera, computed)) = self.nodes.get(entity) else {
            return Err(refused(format!("{shown} is not a UI node")));
        };
        let primary = self.primary.single().ok();
        let window = match camera.get() {
            Some(camera) => match self
                .targets
                .get(camera)
                .ok()
                .and_then(|t| t.normalize(primary))
            {
                Some(NormalizedRenderTarget::Window(window)) => Some(window.entity()),
                _ => return Err(refused(format!("{shown} is not drawn to a window"))),
            },
            None => primary,
        };
        let Some(window) = window.filter(|&w| self.windows.contains(w)) else {
            return Err(refused(format!("{shown} has no window to be drawn in")));
        };
        Ok(Located {
            window,
            center: transform.translation,
            size: computed.map_or(Vec2::ZERO, ComputedNode::size),
        })
    }

    /// The centre of `entity` in physical pixels, if it is still a UI node.
    pub fn center(&self, entity: Entity) -> Option<Vec2> {
        self.nodes
            .get(entity)
            .ok()
            .map(|(transform, ..)| transform.translation)
    }

    /// Whether `window` is a window.
    pub fn is_window(&self, window: Entity) -> bool {
        self.windows.contains(window)
    }
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
    /// Scrolls every ancestor that hides the node, centred at `center` and `size` big (both
    /// physical pixels), by the least that shows all of it, or its start if it is bigger than the
    /// view. Returns whether anything scrolled. Nearest first: scrolling a view moves the node
    /// within the views around it, by what the scroll applied, so each further ancestor judges the
    /// node where it will be once layout has run.
    pub fn scroll_into_view(&mut self, entity: Entity, center: Vec2, size: Vec2) -> bool {
        let (mut start, mut end) = (center - size * 0.5, center + size * 0.5);
        let mut scrolled = false;
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
            let applied = (target - position.0) / scale;
            position.0 = target;
            start -= applied;
            end -= applied;
            scrolled = true;
        }
        scrolled
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use bevy_ecs::{name::Name, world::World};
    use bevy_ui::Overflow;
    use bevy_window::WindowResolution;

    fn locate(world: &mut World, path: &str) -> BrpResult {
        world
            .run_system_cached_with(process_request, Some(json!({ "path": [path] })))
            .unwrap()
    }

    /// A view `height` px tall, its top at `top`, over `content` px of content, scrolling on y.
    pub(crate) fn scroll_view(world: &mut World, top: f32, height: f32, content: f32) -> Entity {
        world
            .spawn((
                Node {
                    overflow: Overflow::scroll_y(),
                    ..Default::default()
                },
                ComputedNode {
                    size: Vec2::new(200.0, height),
                    content_size: Vec2::new(200.0, content),
                    inverse_scale_factor: 1.0,
                    ..Default::default()
                },
                UiGlobalTransform::from_translation(Vec2::new(100.0, top + height * 0.5)),
                ScrollPosition::default(),
            ))
            .id()
    }

    /// A node 20 px tall, its top at `top`.
    pub(crate) fn row(world: &mut World, name: &str, parent: Entity, top: f32) -> Entity {
        world
            .spawn((
                Name::new(name.to_string()),
                ChildOf(parent),
                ComputedNode {
                    size: Vec2::new(200.0, 20.0),
                    inverse_scale_factor: 1.0,
                    ..Default::default()
                },
                UiGlobalTransform::from_translation(Vec2::new(100.0, top + 10.0)),
                ComputedUiTargetCamera::default(),
            ))
            .id()
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
        let view = scroll_view(&mut world, 0.0, 100.0, 400.0);
        row(&mut world, "Start", view, 240.0);

        let found = locate(&mut world, "Start").unwrap();

        assert_eq!(found, json!({ "scrolled": true }));
        // Just far enough that its bottom edge meets the view's.
        let position = world.get::<ScrollPosition>(view).unwrap();
        assert_eq!(position.0, Vec2::new(0.0, 160.0));
    }

    #[test]
    fn nested_views_scroll_together() {
        let mut world = World::new();
        world.spawn((Window::default(), PrimaryWindow));
        // An outer view 100 px tall at the top; inside it, 300 px down, an inner view 100 px tall
        // whose row is 150 px down its content.
        let outer = scroll_view(&mut world, 0.0, 100.0, 500.0);
        let inner = scroll_view(&mut world, 300.0, 100.0, 400.0);
        world.entity_mut(inner).insert(ChildOf(outer));
        row(&mut world, "Start", inner, 450.0);

        let found = locate(&mut world, "Start").unwrap();

        assert_eq!(found, json!({ "scrolled": true }));
        // The inner view brings the row to its bottom edge (470 to 400), and the outer view then
        // brings the row, now at 380 to 400, to its own bottom edge.
        assert_eq!(
            world.get::<ScrollPosition>(inner).unwrap().0,
            Vec2::new(0.0, 70.0)
        );
        assert_eq!(
            world.get::<ScrollPosition>(outer).unwrap().0,
            Vec2::new(0.0, 300.0)
        );
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
