use std::time::Duration;

use bevy::{
    prelude::{Component, Query, Res, Plugin, Transform, Update, Vec3, debug},
    time::Time,
};

pub struct AnimationCurve {
    func: fn(f32) -> f32,
}

impl AnimationCurve {
    pub fn new(func: fn(f32) -> f32) -> Self {
        Self {
            func,
        }
    }

    pub fn eval(&self, progress: f32) -> f32 {
        (self.func)(progress)
    }
}

pub mod curves {
    pub fn linear(x: f32) -> f32 {
        x
    }

    pub fn second_order(x: f32) -> f32 {
        x * x
    }
}

pub struct Animation {
    pub duration: Duration,
    pub curve: AnimationCurve,
}

#[derive(Clone, Copy)]
enum AnimationDirection {
    Forward,
    Backward,
}

impl AnimationDirection {
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

pub enum Repeat {
    Once,
    Mirrored,
}

pub trait AnimationLens: Send + Sync + 'static {
    type C: Component;
    fn lerp(&self, target: &mut Self::C, progress: f32);
}

#[derive(Component)]
pub struct Animator<TLens: AnimationLens> {
    state: AnimationState,
    animation: Animation,
    repeat: Repeat,
    lens: TLens,
}

impl<TLens: AnimationLens> Animator<TLens> {
    pub fn new(animation: Animation, repeat: Repeat, lens: TLens) -> Self {
        Self {
            state: AnimationState {
                completed: false,
                direction: AnimationDirection::Forward,
                progress: 0.0
            },
            animation,
            repeat,
            lens,
        }
    }

    fn tick(&mut self, target: &mut TLens::C, time_elapsed: f32) {
        let full_duration = self.animation.duration.as_secs_f32();
        let progress_made = time_elapsed / full_duration;
        self.state.progress += progress_made * self.state.direction.factor();
        
        match self.repeat {
            Repeat::Once => {
                if self.state.progress >= 1.0 {
                    self.state.completed = true;
                    self.state.progress = 1.0;
                }
            },
            Repeat::Mirrored => {
                if self.state.progress > 1.0 {
                    let over = self.state.progress - 1.0;
                    self.state.progress = 1.0 - over;
                    self.state.direction = !self.state.direction;
                }
                else if self.state.progress < 0.0 {
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
    }
}

pub fn animation_tick_system<TComponent, TLens>(
    time: Res<Time>,
    mut entities: Query<(&mut TComponent, &mut Animator<TLens>)>,
) where
    TComponent: Component,
    TLens: AnimationLens<C = TComponent>,
{
    for (mut component, mut animator) in entities.iter_mut() {
        animator.tick(&mut component, time.delta_seconds());
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

pub struct AnimationPlugin;
impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Update, animation_tick_system::<Transform, TransformTranslationLens>);
    }
}