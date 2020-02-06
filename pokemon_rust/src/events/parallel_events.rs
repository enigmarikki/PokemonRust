//! Generic event. Orchestrates multiple events so that they happen in parallel.

use amethyst::ecs::World;

use super::{BoxedGameEvent, GameEvent, ShouldDisableInput};

#[derive(Default)]
pub struct ParallelEvents {
    events: Vec<Box<dyn GameEvent + Sync + Send>>,
}

impl ParallelEvents {
    /// Adds the given list of events to this event. It is assumed that the
    /// events' `start()` were already called.
    pub fn add_events(&mut self, events: impl Iterator<Item=Box<dyn GameEvent + Sync + Send>>) {
        self.events.extend(events);
    }
}

impl GameEvent for ParallelEvents {
    fn boxed_clone(&self) -> BoxedGameEvent {
        Box::new(ParallelEvents {
            events: self.events.iter().map(|event| event.boxed_clone()).collect(),
        })
    }

    fn start(&mut self, world: &mut World) -> ShouldDisableInput {
        let mut should_disable_input = false;

        for event in &mut self.events {
            should_disable_input = should_disable_input || event.start(world).0;
        }

        ShouldDisableInput(should_disable_input)
    }

    fn tick(&mut self, world: &mut World, disabled_inputs: bool) {
        self.events = self.events
            .drain(..)
            .filter_map(|mut event| {
                event.tick(world, disabled_inputs);

                if event.is_complete(world) {
                    None
                } else {
                    Some(event)
                }
            })
            .collect();
    }

    fn is_complete(&self, _world: &mut World) -> bool {
        self.events.is_empty()
    }
}
