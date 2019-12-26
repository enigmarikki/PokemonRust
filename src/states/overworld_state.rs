use amethyst::{
    animation::{
        AnimationBundle,
        AnimationCommand,
        AnimationControlSet,
        AnimationSet,
        AnimationSetPrefab,
        ControlState,
        EndControl,
        get_animation_set,
    },
    assets::{
        Handle,
        Loader,
        PrefabData,
        PrefabLoader,
        PrefabLoaderSystemDesc,
        ProgressCounter,
        RonFormat,
    },
    core::{ArcThreadPool, bundle::SystemBundle, Transform},
    derive::PrefabData,
    ecs::{
        Dispatcher,
        DispatcherBuilder,
        Entities,
        Entity,
        Join,
        ReadStorage,
        world::{Builder, EntitiesRes},
        World,
        WriteStorage,
    },
    Error,
    input::InputEvent,
    prelude::*,
    renderer::{
        Camera,
        ImageFormat,
        sprite::prefab::SpriteScenePrefab,
        SpriteRender,
        SpriteSheet,
        SpriteSheetFormat,
    },
};

use crate::{
    entities::player::{Player, initialise_player},
};

use serde::{Deserialize, Serialize};
use std::ops::Deref;

pub fn initialise_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_translation_xyz(400., 300., 1.0);

    world
        .create_entity()
        .with(Camera::standard_2d(800., 600.))
        .with(transform)
        .build();
}

pub fn load_sprite_sheet(world: &World) -> Handle<SpriteSheet> {
    let loader = world.read_resource::<Loader>();

    let texture_handle = loader.load(
        "sprites/player.png",
        ImageFormat::default(),
        (),
        &world.read_resource(),
    );

    loader.load(
        "sprites/player.ron",
        SpriteSheetFormat(texture_handle),
        (),
        &world.read_resource(),
    )
}

#[derive(Eq, PartialOrd, PartialEq, Hash, Debug, Copy, Clone, Deserialize, Serialize)]
enum AnimationId {
    Walk,
    Run,
}

#[derive(Debug, Clone, Deserialize, PrefabData)]
struct MyPrefabData {
    sprite_scene: SpriteScenePrefab,
    animation_set: AnimationSetPrefab<AnimationId, SpriteRender>,
}

#[derive(Default)]
pub struct OverworldState<'a, 'b> {
    pub dispatcher: Option<Dispatcher<'a, 'b>>,
    pub progress_counter: Option<ProgressCounter>,
}

impl SimpleState for OverworldState<'_, '_> {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        println!("Welcome to Pokémon Rust!");

        let mut dispatcher_builder = DispatcherBuilder::new()
            .with(
                PrefabLoaderSystemDesc::<MyPrefabData>::default().build(data.world),
                "scene_loader",
                &[],
            )
            .with_pool(data.world.read_resource::<ArcThreadPool>().deref().clone());

        AnimationBundle::<AnimationId, SpriteRender>::new(
            "sprite_animation_control",
            "sprite_sampler_interpolation",
        ).build(data.world, &mut dispatcher_builder)
            .expect("Failed to build AnimationBundle");

        let mut dispatcher = dispatcher_builder.build();
        dispatcher.setup(data.world);
        self.dispatcher = Some(dispatcher);

        let mut progress_counter = ProgressCounter::new();
        let player_prefab = data.world.exec(|loader: PrefabLoader<'_, MyPrefabData>| {
            loader.load(
                "sprites/player.ron",
                RonFormat,
                &mut progress_counter,
            )
        });
        // Creates new entities with components from MyPrefabData
        data.world.create_entity().with(player_prefab).build();
        self.progress_counter = Some(progress_counter);

        // data.world.register::<Player>();
        // let sprite_sheet = load_sprite_sheet(data.world);
        // initialise_player(data.world, sprite_sheet.clone());
        initialise_camera(data.world);
    }

    fn update(&mut self, data: &mut StateData<'_, GameData<'_, '_>>) -> SimpleTrans {
        let world = &mut data.world;

        if let Some(dispatcher) = &mut self.dispatcher {
            dispatcher.dispatch(world);
        }

        if let Some(progress_counter) = &self.progress_counter {
            if progress_counter.is_complete() {
                let entities = world.read_resource::<EntitiesRes>();
                let animation_sets = world.read_storage::<AnimationSet<AnimationId, SpriteRender>>();
                let mut control_sets = world.write_storage::<AnimationControlSet<AnimationId, SpriteRender>>();

                for (entity, animation_set) in (&entities, &animation_sets).join() {
                    get_animation_set(&mut control_sets, entity)
                        .unwrap()
                        .add_animation(
                            AnimationId::Walk,
                            &animation_set.get(&AnimationId::Walk).unwrap(),
                            EndControl::Loop(None),
                            1.0,
                            AnimationCommand::Init,
                        )
                        .add_animation(
                            AnimationId::Run,
                            &animation_set.get(&AnimationId::Run).unwrap(),
                            EndControl::Loop(None),
                            1.0,
                            AnimationCommand::Init,
                        );
                }

                self.progress_counter = None;
            }
        }

        Trans::None
    }

    fn handle_event(&mut self, data: StateData<'_, GameData<'_, '_>>, event: StateEvent) -> SimpleTrans {
        if let StateEvent::Input(event) = event {
            match event {
                InputEvent::ActionPressed(action) if action == "action" => {
                    let entities = data.world.read_resource::<EntitiesRes>();
                    let animation_sets = data.world.read_storage::<AnimationSet<AnimationId, SpriteRender>>();
                    let mut control_sets = data.world.write_storage::<AnimationControlSet<AnimationId, SpriteRender>>();

                    for (_, _, control_set) in (&entities, &animation_sets, &mut control_sets).join() {
                        control_set.pause(AnimationId::Walk);

                        control_set.animations
                            .iter_mut()
                            .filter(|a| a.0 == AnimationId::Run)
                            .for_each(|a| {
                                a.1.state = ControlState::Requested;
                                a.1.command = AnimationCommand::Start;
                            });
                    }
                },
                InputEvent::ActionReleased(action) if action == "action" => {
                    let entities = data.world.read_resource::<EntitiesRes>();
                    let animation_sets = data.world.read_storage::<AnimationSet<AnimationId, SpriteRender>>();
                    let mut control_sets = data.world.write_storage::<AnimationControlSet<AnimationId, SpriteRender>>();

                    for (_, _, control_set) in (&entities, &animation_sets, &mut control_sets).join() {
                        control_set.pause(AnimationId::Run);

                        control_set.animations
                            .iter_mut()
                            .filter(|a| a.0 == AnimationId::Walk)
                            .for_each(|a| {
                                a.1.state = ControlState::Requested;
                                a.1.command = AnimationCommand::Start;
                            });
                    }
                },
                _ => {},
            }
        }

        Trans::None
    }
}
