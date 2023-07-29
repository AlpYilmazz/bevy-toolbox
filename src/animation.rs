use std::time::Duration;

use bevy::{
    prelude::{
        debug, Component, Entity, Event, Events, Plugin, Query, Res, ResMut, Transform, Update,
        Vec3,
    },
    time::Time,
};
use interpolation::{Ease, EaseFunction};

pub enum AnimationCurve {
    EaseFunction(EaseFunction),
    Linear,
    Step(f32),
    Custom(fn(f32) -> f32),
}

impl AnimationCurve {
    pub fn eval(&self, progress: f32) -> f32 {
        match self {
            AnimationCurve::EaseFunction(ease_func) => Ease::calc(progress, *ease_func),
            AnimationCurve::Linear => progress,
            AnimationCurve::Step(cutoff) => {
                if *cutoff < progress {
                    0.0
                } else {
                    1.0
                }
            }
            AnimationCurve::Custom(func) => (func)(progress),
        }
    }
}

impl From<EaseFunction> for AnimationCurve {
    fn from(value: EaseFunction) -> Self {
        Self::EaseFunction(value)
    }
}

pub mod curves {
    pub fn linear(x: f32) -> f32 {
        x
    }

    pub fn second_order(x: f32) -> f32 {
        x * x
    }

    pub fn third_order(x: f32) -> f32 {
        x * x * x
    }
}

// pub trait Animate {
//     fn get_duration(&self) -> f32;
//     fn tick(&mut self, progress: f32) -> bool;
// }

pub struct Animation {
    pub duration: Duration,
    pub curve: AnimationCurve,
}

pub struct Delay {
    pub duration: Duration,
}

pub enum AnimationStep<TLens: AnimationLens> {
    Animation(Animation, TLens),
    Delay(Delay),
}

// impl<TLens: AnimationLens> AnimationStep<TLens> {
//     pub fn animation(animation: Animation, lens: TLens) -> Self {
//         Self::Animation(animation, lens, AnimId::None)
//     }

//     pub fn delay(delay: Delay) -> Self {
//         Self::Delay(delay, AnimId::None)
//     }

//     fn set_id(&mut self, anim_id: AnimId) {
//         match self {
//             AnimationStep::Animation(_, _, id) => *id = anim_id,
//             AnimationStep::Delay(_, id) => *id = anim_id,
//         }
//     }

//     pub fn with_id(mut self, id: f32) -> Self {
//         self.set_id(AnimId::Some(id));
//         self
//     }
// }

// pub struct AnimationSequence {
//     seq: Vec<AnimationEnum>,
//     current: usize,
// }

// impl AnimationSequence {
//     pub fn new() -> Self {
//         Self {
//             seq: Vec::new(),
//             current: 0,
//         }
//     }

//     pub fn push_sequence<const N: usize>(mut self, seq: [AnimationEnum; N]) -> Self {
//         for e in seq {
//             self.seq.push(e)
//         }
//         self
//     }

//     pub fn push_animation(mut self, animation: Animation) -> Self {
//         self.seq.push(AnimationEnum::Animation(animation));
//         self
//     }

//     pub fn push_delay(mut self, delay: Delay) -> Self {
//         self.seq.push(AnimationEnum::Delay(delay));
//         self
//     }

//     pub fn get_duration(&self) -> Duration {
//         match &self.seq[self.current] {
//             AnimationEnum::Animation(anim) => anim.duration,
//             AnimationEnum::Delay(delay) => delay.duration,
//         }
//     }

//     pub fn update(&mut self, repeat: Repeat, state: &mut AnimationState) {}

//     pub fn eval(&self, progress: f32) -> f32 {
//         match &self.seq[self.current] {
//             AnimationEnum::Animation(anim) => {}
//             AnimationEnum::Delay(delay) => {}
//         }
//     }
// }

#[derive(Clone, Copy)]
pub enum AnimationDirection {
    Forward,
    Backward,
}

impl AnimationDirection {
    fn start_point(&self) -> f32 {
        match self {
            Self::Forward => 0.0,
            Self::Backward => 1.0,
        }
    }

    fn factor(&self) -> f32 {
        match self {
            Self::Forward => 1.0,
            Self::Backward => -1.0,
        }
    }
}

impl std::ops::Not for AnimationDirection {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Forward => Self::Backward,
            Self::Backward => Self::Forward,
        }
    }
}

struct AnimationState {
    // TODO
    completed: bool,
    direction: AnimationDirection,
    progress: f32,
}

#[derive(Clone, Copy)]
pub enum Repeat {
    Once,
    Always,
    Mirrored,
}

pub trait AnimationLens: Send + Sync + 'static {
    type C: Component;
    fn lerp(&self, target: &mut Self::C, progress: f32);
}

#[derive(Component)]
pub struct Animator<TLens: AnimationLens> {
    id: Option<u32>,
    state: AnimationState,
    animation: Animation,
    repeat: Repeat,
    lens: TLens,
}

impl<TLens: AnimationLens> Animator<TLens> {
    pub fn new(animation: Animation, repeat: Repeat, lens: TLens) -> Self {
        Self {
            id: None,
            state: AnimationState {
                completed: false,
                direction: AnimationDirection::Forward,
                progress: 0.0,
            },
            animation,
            repeat,
            lens,
        }
    }

    pub fn new_with_direction(
        animation: Animation,
        direction: AnimationDirection,
        repeat: Repeat,
        lens: TLens,
    ) -> Self {
        Self {
            id: None,
            state: AnimationState {
                completed: false,
                direction,
                progress: direction.start_point(),
            },
            animation,
            repeat,
            lens,
        }
    }

    pub fn with_id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    fn tick(
        &mut self,
        target: &mut TLens::C,
        time_elapsed: f32,
        entity: Entity,
        events: &mut Events<AnimationCompleted>,
    ) -> f32 {
        if self.state.completed {
            return match self.state.direction {
                AnimationDirection::Forward => 1.0,
                AnimationDirection::Backward => 0.0,
            };
        }

        let full_duration = self.animation.duration.as_secs_f32();
        let progress_made = time_elapsed / full_duration;
        self.state.progress += progress_made * self.state.direction.factor();

        match self.repeat {
            Repeat::Once => {
                if self.state.progress > 1.0 {
                    self.state.completed = true;
                    self.state.progress = 1.0;
                    events.send(AnimationCompleted {
                        entity,
                        animator_id: self.id,
                        animation_id: 0,
                    });
                } else if self.state.progress < 0.0 {
                    self.state.completed = true;
                    self.state.progress = 0.0;
                    events.send(AnimationCompleted {
                        entity,
                        animator_id: self.id,
                        animation_id: 0,
                    });
                }
            }
            Repeat::Always => {
                if self.state.progress > 1.0 {
                    let over = self.state.progress - 1.0;
                    self.state.progress = 0.0 + over;
                } else if self.state.progress < 0.0 {
                    let over = 0.0 - self.state.progress;
                    self.state.progress = 1.0 - over;
                }
            }
            Repeat::Mirrored => {
                if self.state.progress > 1.0 {
                    let over = self.state.progress - 1.0;
                    self.state.progress = 1.0 - over;
                    self.state.direction = !self.state.direction;
                } else if self.state.progress < 0.0 {
                    let over = 0.0 - self.state.progress;
                    self.state.progress = 0.0 + over;
                    self.state.direction = !self.state.direction;
                }
            }
        }

        let time_progress = self.state.progress;
        let anim_progress = self.animation.curve.eval(time_progress);
        debug!("time: {}, anim: {}", time_progress, anim_progress);
        self.lens.lerp(target, anim_progress);

        time_progress
    }
}

#[derive(Component)]
pub struct SequenceAnimator<TLens: AnimationLens> {
    id: Option<u32>,
    state: AnimationState,
    current: usize,
    seq: Vec<AnimationStep<TLens>>,
    repeat: Repeat,
}

impl<TLens: AnimationLens> SequenceAnimator<TLens> {
    pub fn new(seq: Vec<AnimationStep<TLens>>, repeat: Repeat) -> Self {
        let completed = if seq.is_empty() { true } else { false };
        Self {
            id: None,
            state: AnimationState {
                completed,
                direction: AnimationDirection::Forward,
                progress: 0.0,
            },
            current: 0,
            seq: seq,
            repeat,
        }
    }

    pub fn new_with_direction<const N: usize>(
        seq: Vec<AnimationStep<TLens>>,
        direction: AnimationDirection,
        repeat: Repeat,
    ) -> Self {
        let completed = if seq.is_empty() { true } else { false };
        Self {
            id: None,
            state: AnimationState {
                completed,
                direction,
                progress: direction.start_point(),
            },
            current: match direction {
                AnimationDirection::Forward => 0,
                AnimationDirection::Backward => seq.len() - 1,
            },
            seq,
            repeat,
        }
    }

    pub fn with_id(mut self, id: u32) -> Self {
        self.id = Some(id);
        self
    }

    /// Does not take overtime into account
    fn next_animation(&mut self) {
        let last = self.seq.len() - 1;
        match (self.repeat, self.state.direction, self.current) {
            (Repeat::Once, AnimationDirection::Forward, i) if i == last => {
                self.state.completed = true;
                self.state.progress = 1.0;
            }
            (Repeat::Once, AnimationDirection::Forward, _) => {
                self.current += 1;
                self.state.progress = 0.0;
            }
            (Repeat::Once, AnimationDirection::Backward, 0) => {
                self.state.completed = true;
                self.state.progress = 0.0
            }
            (Repeat::Once, AnimationDirection::Backward, _) => {
                self.current -= 1;
                self.state.progress = 1.0;
            }
            // --
            (Repeat::Always, AnimationDirection::Forward, i) if i == last => {
                self.current = 0;
                self.state.progress = 0.0;
            }
            (Repeat::Always, AnimationDirection::Forward, _) => {
                self.current += 1;
                self.state.progress = 0.0;
            }
            (Repeat::Always, AnimationDirection::Backward, 0) => {
                self.current = last;
                self.state.progress = 1.0;
            }
            (Repeat::Always, AnimationDirection::Backward, _) => {
                self.current -= 1;
                self.state.progress = 1.0
            }
            // --
            (Repeat::Mirrored, AnimationDirection::Forward, i) if i == last => {
                // self.current = i;
                self.state.direction = AnimationDirection::Backward;
                self.state.progress = 1.0;
            }
            (Repeat::Mirrored, AnimationDirection::Forward, _) => {
                self.current += 1;
                self.state.progress = 0.0;
            }
            (Repeat::Mirrored, AnimationDirection::Backward, 0) => {
                // self.current = 0;
                self.state.direction = AnimationDirection::Forward;
                self.state.progress = 0.0;
            }
            (Repeat::Mirrored, AnimationDirection::Backward, _) => {
                self.current -= 1;
                self.state.progress = 1.0;
            }
        }
    }

    pub fn tick(
        &mut self,
        target: &mut TLens::C,
        time_elapsed: f32,
        entity: Entity,
        events: &mut Events<AnimationCompleted>,
    ) {
        if self.state.completed {
            return;
        }

        let mut overtime = 0.0;
        match &self.seq[self.current] {
            AnimationStep::Animation(anim, lens) => {
                let full_duration = anim.duration.as_secs_f32();
                let progress_made = time_elapsed / full_duration;
                self.state.progress += progress_made * self.state.direction.factor();

                let time_progress = self.state.progress.clamp(0.0, 1.0);
                let anim_progress = anim.curve.eval(time_progress);
                lens.lerp(target, anim_progress);

                if self.state.progress > 1.0 {
                    overtime = (self.state.progress - 1.0) * full_duration;
                    events.send(AnimationCompleted {
                        entity,
                        animator_id: self.id,
                        animation_id: self.current,
                    });
                    self.next_animation();
                } else if self.state.progress < 0.0 {
                    overtime = (0.0 - self.state.progress) * full_duration;
                    events.send(AnimationCompleted {
                        entity,
                        animator_id: self.id,
                        animation_id: self.current,
                    });
                    self.next_animation();
                }
            }
            AnimationStep::Delay(delay) => {
                let delay_duration = delay.duration.as_secs_f32();
                let progress_made = time_elapsed / delay_duration;
                self.state.progress += progress_made * self.state.direction.factor();

                if self.state.progress > 1.0 {
                    overtime = (self.state.progress - 1.0) * delay_duration;
                    events.send(AnimationCompleted {
                        entity,
                        animator_id: self.id,
                        animation_id: self.current,
                    });
                    self.next_animation();
                } else if self.state.progress < 0.0 {
                    overtime = (0.0 - self.state.progress) * delay_duration;
                    events.send(AnimationCompleted {
                        entity,
                        animator_id: self.id,
                        animation_id: self.current,
                    });
                    self.next_animation();
                }
            }
        }

        // Tick once more for the overtime
        if overtime != 0.0 {
            self.tick(target, overtime, entity, events);
        }
    }
}

#[derive(Event)]
pub struct AnimationCompleted {
    pub entity: Entity,
    pub animator_id: Option<u32>,
    pub animation_id: usize,
}

pub fn animation_tick_system<TComponent, TLens>(
    time: Res<Time>,
    mut entities: Query<(Entity, &mut TComponent, &mut Animator<TLens>)>,
    mut events: ResMut<Events<AnimationCompleted>>,
) where
    TComponent: Component,
    TLens: AnimationLens<C = TComponent>,
{
    for (entity, mut component, mut animator) in entities.iter_mut() {
        animator.tick(&mut component, time.delta_seconds(), entity, &mut events);
    }
}

pub fn animation_sequence_tick_system<TComponent, TLens>(
    time: Res<Time>,
    mut entities: Query<(Entity, &mut TComponent, &mut SequenceAnimator<TLens>)>,
    mut events: ResMut<Events<AnimationCompleted>>,
) where
    TComponent: Component,
    TLens: AnimationLens<C = TComponent>,
{
    for (entity, mut component, mut animator) in entities.iter_mut() {
        animator.tick(&mut component, time.delta_seconds(), entity, &mut events);
    }
}

pub struct TransformTranslationLens {
    pub start: Vec3,
    pub end: Vec3,
}
impl AnimationLens for TransformTranslationLens {
    type C = Transform;

    fn lerp(&self, target: &mut Self::C, progress: f32) {
        target.translation = self.start + (self.end - self.start) * progress;
    }
}

pub struct TransformScaleLens {
    pub start: Vec3,
    pub end: Vec3,
}
impl AnimationLens for TransformScaleLens {
    type C = Transform;

    fn lerp(&self, target: &mut Self::C, progress: f32) {
        target.scale = self.start + (self.end - self.start) * progress;
    }
}

pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<AnimationCompleted>()
            .add_systems(
                Update,
                animation_tick_system::<Transform, TransformTranslationLens>,
            )
            .add_systems(
                Update,
                animation_sequence_tick_system::<Transform, TransformTranslationLens>,
            )
            .add_systems(
                Update,
                animation_tick_system::<Transform, TransformScaleLens>,
            )
            .add_systems(
                Update,
                animation_sequence_tick_system::<Transform, TransformScaleLens>,
            );
    }
}
