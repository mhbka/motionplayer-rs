use std::collections::VecDeque;

use slotmap::{SecondaryMap, SlotMap, new_key_type};

new_key_type! { 
    pub struct TimelineControlTrackKey;
    pub struct TimelineKey; 
}

/// The state of a timeline.
pub struct TimelineState {
    pub label: String,
    pub flags: isize,
    pub playing: bool,
    pub was_playing: bool,
    pub should_loop: bool,
    pub loop_time: f64,
    pub total_frames: f64,
    pub current_time: f64,
    pub blend_ratio: f64,
    pub blend_auto_stop: bool,
    pub control_initialized: bool,
    pub control_last_applied_time: f64,
    pub control_frame_cursor: Vec<isize>,
    pub control_track_values: SecondaryMap<TimelineControlTrackKey, f64>,
    pub control_track_animators: SecondaryMap<TimelineControlTrackKey,TimelineControlAnimatorState>,
    pub blend_animator: TimelineControlAnimatorState,
}

impl TimelineState {
    /// Update the timeline state if there's no corresponding control binding for it.
    /// 
    /// Returns whether to keep playing it.
    pub fn progress_basic(&mut self, delta: f64) -> bool {
        self.current_time += delta;
        if self.total_frames > 0.0 && self.current_time >= self.total_frames {
            if !wrap_timeline(&mut self.current_time, self.total_frames, self.loop_time) {
                self.playing = false;
                return false;
            }
        }
        return true;
    }

    /// Check if to stop playing the timeline.
    /// 
    /// If `binding_last_time` is supplied, also checks if we've passed it.
    /// 
    /// Sets internal state and returns if stopped.
    pub fn maybe_stop_playing(&mut self, binding_last_time: Option<f64>) -> bool {
        let blend_animator_pending = self.blend_animator.active || !self.blend_animator.queue.is_empty();
        if !(self.blend_auto_stop && !blend_animator_pending) {
            return true;
        }
        if let Some(last_time) = binding_last_time {
            if last_time <= self.current_time {
                return true;
            }
            else {
                self.current_time = last_time;
            }
        }
        self.playing = false;
        return false;
    }
}

/// Wraps the timeline for the timeline state.
fn wrap_timeline(current_time: &mut f64, total_frames: f64, loop_time: f64) -> bool {
    if total_frames <= 0.0 || *current_time < total_frames {
        return true;
    }
    if (loop_time < 0.0) || loop_time >= total_frames  {
        *current_time = total_frames;
        return false;
    }

    let mut guard = 0;
    while *current_time >= total_frames && guard + 1 < 1024 {
        guard += 1;
        *current_time += loop_time - total_frames; 
    }

    if *current_time >= total_frames {
        *current_time = total_frames;
        return false;
    }

    return true;
}


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
    time: f64,
    is_type_zero: bool,
    value: f32,
    easing_weight: f64
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

/// A keyframe control?
pub struct TimelineControlKeyframe {
    pub value: f64,
    pub duration: f64,
    pub weight: f64
}
