use std::collections::VecDeque;

use slotmap::new_key_type;

new_key_type! { pub struct TimelineKey; }

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
    pub control_track_values: Vec<f64>,
    pub control_track_animators: Vec<TimelineControlAnimatorState>,
    pub blend_animator: TimelineControlAnimatorState,
}

impl TimelineState {
    /// Update the timeline state if there's no corresponding control binding for it.
    /// 
    /// Returns whether to keep playing it.
    pub fn no_control_binding_update(&mut self, delta: f64) -> bool {
        self.current_time += delta;
        if self.total_frames > 0.0 && self.current_time >= self.total_frames {
            if !wrap_timeline(&mut self.current_time, self.total_frames, self.loop_time) {
                self.playing = false;
                return false;
            }
        }
        return true;
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

/// The state of an animator within a timeline.
pub struct TimelineControlAnimatorState {
    active: bool,
    current_value: f32,
    start_value: f32,
    target_value: f32,
    progress: f32,
    duration: f32,
    weight: f32,
    queue: VecDeque<TimelineControlKeyframe>
}

/// A keyframe control?
pub struct TimelineControlKeyframe {
    value: f32,
    duration: f32,
    weight: f32
}
