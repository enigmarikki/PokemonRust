use amethyst::{
    core::{math::Vector3, Transform},
    ecs::Entity,
    input::{InputEvent, StringBindings},
    renderer::{palette::Srgba, resources::Tint, SpriteRender},
};

use crate::{
    battle::{
        backend::{
            BattleBackend,
            event::InitialSwitchIn,
            Team,
        },
        rng::StandardBattleRng,
    },
    constants::BATTLE_CAMERA_POSITION,
};

use super::super::{BattleSystemData, FrontendEvent};

// TODO: move these window-related constants somewhere else
const WINDOW_WIDTH: f32 = 800.;
const WINDOW_HEIGHT: f32 = 600.;

// TODO: move to a better place
const SWITCH_IN_ANIMATION_TIME: f32 = 0.5;

const P1_SPRITE_Y: f32 = BATTLE_CAMERA_POSITION.1 - WINDOW_HEIGHT / 4.;
const P2_SPRITE_Y: f32 = BATTLE_CAMERA_POSITION.1 + WINDOW_HEIGHT / 4.;

// Both initial positions should be off-screen to improve the animation
const P1_SPRITE_INITIAL_X: f32 = BATTLE_CAMERA_POSITION.0 - WINDOW_WIDTH / 2. - 128.;
const P2_SPRITE_INITIAL_X: f32 = BATTLE_CAMERA_POSITION.0 + WINDOW_WIDTH / 2. + 128.;

const P1_SPRITE_FINAL_X: f32 = BATTLE_CAMERA_POSITION.0 - WINDOW_WIDTH / 3.;
const P2_SPRITE_FINAL_X: f32 = BATTLE_CAMERA_POSITION.0 + WINDOW_WIDTH / 3.;

fn get_p1_sprite_transform() -> Transform {
    let mut transform = Transform::default();
    transform.set_translation_xyz(P1_SPRITE_INITIAL_X, P1_SPRITE_Y, 0.);
    transform.set_scale(Vector3::new(2., 2., 2.));

    transform
}

fn get_p2_sprite_transform() -> Transform {
    let mut transform = Transform::default();
    transform.set_translation_xyz(P2_SPRITE_INITIAL_X, P2_SPRITE_Y, 0.);
    transform.set_scale(Vector3::new(1.8, 1.8, 1.8));

    transform
}

pub enum InitialSwitchInEvent {
    PendingStart {
        event_data: InitialSwitchIn,
    },
    Started {
        event_data: InitialSwitchIn,
        pokemon_entity: Entity,
        elapsed_time: f32,
    },
}

impl FrontendEvent for InitialSwitchInEvent {
    fn start(
        &mut self,
        backend: &BattleBackend<StandardBattleRng>,
        system_data: &mut BattleSystemData,
    ) {
        if let InitialSwitchInEvent::PendingStart { event_data } = self {
            let BattleSystemData {
                sprite_renders,
                transforms,
                tints,
                entities,
                resources,
                ..
            } = system_data;

            let (sprite_sheet, transform) = if event_data.team == Team::P1 {
                (resources.gen1_back.clone(), get_p1_sprite_transform())
            } else {
                (resources.gen1_front.clone(), get_p2_sprite_transform())
            };

            let pokemon_species = backend.get_species(event_data.pokemon);

            let sprite_render = SpriteRender {
                sprite_sheet,
                sprite_number: pokemon_species.national_number - 1,
            };

            let pokemon_entity = entities
                .build_entity()
                .with(sprite_render, sprite_renders)
                .with(transform, transforms)
                .with(Tint(Srgba::new(1.0, 1.0, 1.0, 0.1)), tints)
                .build();

            let elapsed_time = if event_data.is_already_sent_out {
                SWITCH_IN_ANIMATION_TIME
            } else {
                0.
            };

            *self = InitialSwitchInEvent::Started {
                event_data: event_data.clone(),
                pokemon_entity,
                elapsed_time,
            };
        }
    }

    fn tick(
        &mut self,
        _input_events: Vec<InputEvent<StringBindings>>,
        _backend: &BattleBackend<StandardBattleRng>,
        system_data: &mut BattleSystemData,
    ) -> bool {
        if let InitialSwitchInEvent::Started { event_data, pokemon_entity, elapsed_time } = self {
            let BattleSystemData {
                transforms,
                time,
                ..
            } = system_data;

            let transform = transforms
                .get_mut(*pokemon_entity)
                .expect("Failed to retrieve Transform");

            let x = {
                let (initial_x, final_x) = match event_data.team {
                    Team::P1 => (P1_SPRITE_INITIAL_X, P1_SPRITE_FINAL_X),
                    Team::P2 => (P2_SPRITE_INITIAL_X, P2_SPRITE_FINAL_X),
                };
                let progress = (*elapsed_time / SWITCH_IN_ANIMATION_TIME).min(1.);

                initial_x + (final_x - initial_x) * progress
            };
            transform.set_translation_x(x);

            if *elapsed_time >= SWITCH_IN_ANIMATION_TIME {
                true
            } else {
                *elapsed_time += time.delta_seconds();
                false
            }
        } else {
            panic!("Called tick() before start()");
        }
    }
}