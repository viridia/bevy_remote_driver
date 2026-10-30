//! Whether a BRP client can see a scene half applied (DD3.2).
//!
//! BRP answers requests in `RemoteLast`, the last schedule of a frame, so the world after
//! `App::update` returns is what a request sees. Each test queues a scene whose one dependency a
//! [`HeldReader`] holds unloaded, looks after every update, then releases it and looks until the
//! scene is whole.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use bevy::app::{SpawnScene, TaskPoolPlugin};
use bevy::asset::io::memory::{Dir, MemoryAssetReader};
use bevy::asset::io::{
    AssetReader, AssetReaderError, AssetSourceBuilder, AssetSourceId, PathStream, Reader,
};
use bevy::asset::{AssetLoader, LoadContext};
use bevy::ecs::schedule::Stepping;
use bevy::prelude::*;
use bevy::scene::{ScenePatch, ScenePlugin};
use bevy::tasks::futures_lite::future::yield_now;

/// Updates while the dependency is held, enough to show the scene waits.
const HELD_FRAMES: usize = 10;

/// Updates allowed for the scene to become whole once its dependency loads.
const LOAD_FRAMES: usize = 1000;

/// The scene's entities: `Root` with children `A` and `C`, and `B` a child of `A`.
const NAMES: [&str; 4] = ["Root", "A", "B", "C"];

/// Put on an entity by `held.bsn`, the scene each test's scene inherits and so waits for. A
/// `Handle` field alone does not make a scene wait.
#[derive(Component, FromTemplate)]
struct Inherited;

/// Loads any `.bsn` file as a scene holding [`Inherited`].
#[derive(TypePath)]
struct HeldSceneLoader;

impl AssetLoader for HeldSceneLoader {
    type Asset = ScenePatch;
    type Settings = ();
    type Error = std::io::Error;

    async fn load(
        &self,
        _reader: &mut dyn Reader,
        _settings: &(),
        load_context: &mut LoadContext<'_>,
    ) -> Result<ScenePatch, Self::Error> {
        Ok(ScenePatch::load_with(load_context, bsn! { Inherited }))
    }

    fn extensions(&self) -> &[&str] {
        &["bsn"]
    }
}

/// What a client finds of the scene's names.
#[derive(Debug, PartialEq)]
enum Seen {
    Nothing,
    Whole,
    /// Some names, or all of them out of their hierarchy.
    Partial(Vec<&'static str>),
}

/// Reads from memory, holding every read until `released` is set. Bevy's own `GatedReader` is
/// built only for its tests.
struct HeldReader {
    inner: MemoryAssetReader,
    released: Arc<AtomicBool>,
}

impl AssetReader for HeldReader {
    async fn read<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        while !self.released.load(Ordering::Acquire) {
            yield_now().await;
        }
        self.inner.read(path).await
    }

    async fn read_meta<'a>(&'a self, path: &'a Path) -> Result<impl Reader + 'a, AssetReaderError> {
        self.inner.read_meta(path).await
    }

    async fn read_directory<'a>(
        &'a self,
        path: &'a Path,
    ) -> Result<Box<PathStream>, AssetReaderError> {
        self.inner.read_directory(path).await
    }

    async fn is_directory<'a>(&'a self, path: &'a Path) -> Result<bool, AssetReaderError> {
        self.inner.is_directory(path).await
    }
}

/// An app whose asset reads wait until the returned flag is set.
fn app() -> (App, Arc<AtomicBool>) {
    let dir = Dir::default();
    dir.insert_asset_text(Path::new("held.bsn"), "");
    let released = Arc::new(AtomicBool::new(false));
    let reader_released = released.clone();

    let mut app = App::new();
    app.register_asset_source(
        AssetSourceId::Default,
        AssetSourceBuilder::new(move || {
            Box::new(HeldReader {
                inner: MemoryAssetReader { root: dir.clone() },
                released: reader_released.clone(),
            })
        }),
    );
    app.add_plugins((
        TaskPoolPlugin::default(),
        AssetPlugin::default(),
        ScenePlugin,
    ));
    app.finish();
    app.cleanup();
    app.register_asset_loader(HeldSceneLoader);
    (app, released)
}

fn look(world: &mut World) -> Seen {
    let mut named = world.query::<(Entity, &Name, Option<&ChildOf>)>();
    let found: Vec<_> = NAMES
        .iter()
        .filter_map(|&name| {
            named
                .iter(world)
                .find(|(_, n, _)| n.as_str() == name)
                .map(|(entity, _, parent)| (name, entity, parent.map(ChildOf::parent)))
        })
        .collect();
    if found.is_empty() {
        return Seen::Nothing;
    }
    let entity = |name| found.iter().find(|(n, ..)| *n == name).map(|(_, e, _)| *e);
    let parent = |name| {
        found
            .iter()
            .find(|(n, ..)| *n == name)
            .and_then(|(.., p)| *p)
    };
    let whole = found.len() == NAMES.len()
        && parent("A") == entity("Root")
        && parent("C") == entity("Root")
        && parent("B") == entity("A");
    if whole {
        Seen::Whole
    } else {
        Seen::Partial(found.iter().map(|(n, ..)| *n).collect())
    }
}

/// Updates `frames` times, looking after each.
fn watch(app: &mut App, frames: usize) -> Vec<Seen> {
    (0..frames)
        .map(|_| {
            app.update();
            look(app.world_mut())
        })
        .collect()
}

/// Updates until the scene is whole, returning everything seen on the way.
fn watch_until_whole(app: &mut App, mut before_update: impl FnMut(&mut App)) -> Vec<Seen> {
    let mut seen = Vec::new();
    for _ in 0..LOAD_FRAMES {
        before_update(app);
        app.update();
        seen.push(look(app.world_mut()));
        if seen.last() == Some(&Seen::Whole) {
            return seen;
        }
    }
    panic!(
        "the scene was not whole after {LOAD_FRAMES} updates; last seen {:?}",
        seen.last()
    );
}

fn scene() -> impl Scene {
    bsn! {
        :"held.bsn"
        #Root
        Children [
            #A Children [ #B ]
            --
            #C
        ]
    }
}

fn assert_never_partial(seen: &[Seen]) {
    if let Some((frame, partial)) = seen
        .iter()
        .enumerate()
        .find(|(_, s)| matches!(s, Seen::Partial(_)))
    {
        panic!("update {frame} saw {partial:?}");
    }
}

#[test]
fn a_queued_scene_is_seen_whole_or_not_at_all() {
    let (mut app, released) = app();
    app.world_mut().queue_spawn_scene(scene());

    let held = watch(&mut app, HELD_FRAMES);
    assert!(
        held.iter().all(|s| *s == Seen::Nothing),
        "while held: {held:?}"
    );

    released.store(true, Ordering::Release);
    assert_never_partial(&watch_until_whole(&mut app, |_| {}));
}

#[test]
fn a_scene_applied_to_a_named_entity_is_seen_partial() {
    let (mut app, released) = app();
    app.world_mut()
        .spawn(Name::new("Root"))
        .queue_apply_scene(bsn! {
            :"held.bsn"
            Children [
                #A Children [ #B ]
                --
                #C
            ]
        });

    let held = watch(&mut app, HELD_FRAMES);
    assert!(
        held.iter().all(|s| *s == Seen::Partial(vec!["Root"])),
        "while held: {held:?}"
    );

    released.store(true, Ordering::Release);
    watch_until_whole(&mut app, |_| {});
}

#[test]
#[ignore = "bevy#26383: a queued scene list is dropped while its dependencies load"]
fn related_scenes_are_seen_partial() {
    let (mut app, released) = app();
    app.world_mut()
        .spawn(Name::new("Root"))
        .queue_spawn_related_scenes::<Children>(bsn_list! {
            :"held.bsn"
            #A
            Children [ #B ]
            --
            #C
        });

    let held = watch(&mut app, HELD_FRAMES);
    assert!(
        held.iter().all(|s| *s == Seen::Partial(vec!["Root"])),
        "while held: {held:?}"
    );

    released.store(true, Ordering::Release);
    watch_until_whole(&mut app, |_| {});
}

/// With stepping, each update runs one system of `SpawnScene`, so a look after each sees every
/// boundary between them, the finest an outside client can see. In this app no other schedule
/// touches scenes.
///
/// The scene has no dependency: stepping splits `resolve_scene_patches` from `spawn_queued`, and
/// `spawn_queued` takes a dependency's load event before the patch is resolved, so a scene that
/// waits for one never spawns.
#[test]
fn a_stepped_scene_is_seen_whole_or_not_at_all() {
    let (mut app, _) = app();
    let mut stepping = Stepping::new();
    stepping.add_schedule(SpawnScene).enable();
    app.insert_resource(stepping);
    app.world_mut().queue_spawn_scene(bsn! {
        #Root
        Children [
            #A Children [ #B ]
            --
            #C
        ]
    });

    let seen = watch_until_whole(&mut app, |app| {
        app.world_mut().resource_mut::<Stepping>().step_frame();
    });
    assert_never_partial(&seen);
    assert!(seen.len() > 1, "whole after the first step: {seen:?}");
}
