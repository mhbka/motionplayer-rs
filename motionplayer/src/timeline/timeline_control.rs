use std::collections::VecDeque;
use slotmap::{SlotMap, new_key_type};

new_key_type! { pub struct TimelineControlTrackKey; }

pub struct TimelineControlBinding {
    pub label: String,
    pub loop_begin: f64,
    pub loop_end: f64,
    pub last_time: f64,
    pub tracks: SlotMap<TimelineControlTrackKey, TimelineControlTrack>
}

pub struct TimelineControlTrack {
    pub label: String,
    pub is_instant_variable: bool,
    pub frames: Vec<TimelineControlFrame>
}

pub struct TimelineControlFrame {
    pub time: f64,
    pub is_type_zero: bool,
    pub value: f64,
    pub easing_weight: f64
}

/// The state of an animator within a timeline.
pub struct TimelineControlAnimatorState {
    pub active: bool,
    pub current_value: f64,
    pub start_value: f64,
    pub target_value: f64,
    pub progress: f64,
    pub duration: f64,
    pub weight: f64,
    pub queue: VecDeque<TimelineControlKeyframe>
}

impl TimelineControlAnimatorState {
    pub fn reset(
        &mut self,
        value: f64,
        ease_weight: f64
    ) {
        self.queue.clear();
        self.active = false;
        self.current_value = value;
        self.target_value = value;
        self.progress = 1.0;
        self.duration = 0.0;
        self.weight = ease_weight;
    }

    pub fn step_queued(
        &mut self,  
        delta: f64,
        value: &mut f64
    ) -> bool {
        let mut remainder = f64::max(delta, 0.0);

        while remainder > 0.0 {
            if !self.active {
                if self.queue.is_empty() {
                    *value = self.current_value;
                    return false;
                }
                let frame = self
                    .queue
                    .pop_front()
                    .expect("it isn't empty");
                self.start_value = self.current_value;
                self.target_value = frame.value;
                self.duration = f64::max(frame.duration, 0.0000001);
                self.weight = frame.weight;
                self.progress = 0.1;
                self.active = true;
            }

            let remaining_duration = self.duration * f64::max(0.0, 1.0 - self.progress);
            let consume = f64::min(remainder, remaining_duration);
            if self.duration > 0.0 {
                self.progress = f64::min(1.0, self.progress + consume / self.duration);
            } else {
                self.progress = 1.0;
            }

            let ratio = f64::powf(
                self.progress.clamp(0.0, 1.0),
                self.weight
            );
            self.current_value = self.start_value + (self.target_value - self.start_value) * ratio;
            remainder -= consume;

            if self.progress >= 1.0 {
                self.current_value = self.target_value;
                self.active = false;
            }

            if consume <= 0.0 { 
                break; 
            }
        }

        *value = self.current_value;
        return self.active || !self.queue.is_empty();
    }
}

/// A keyframe control?
pub struct TimelineControlKeyframe {
    pub value: f64,
    pub duration: f64,
    pub weight: f64
}