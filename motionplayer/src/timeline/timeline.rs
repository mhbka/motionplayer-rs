use slotmap::{SecondaryMap, new_key_type};
use crate::timeline::timeline_control::{TimelineControlAnimatorState, TimelineControlFrame, TimelineControlKeyframe, TimelineControlTrackKey};

new_key_type! { 
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
    pub control_frame_cursor: SecondaryMap<TimelineControlTrackKey, isize>,
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

    /// Schedule the given track's animator to transition to `value` using the weight `ease_weight`.
    pub fn schedule_timeline_control_animator(
        &mut self,
        track: &TimelineControlTrackKey,
        value: f64,
        transition: f64,
        ease_weight: f64
    ) {
        let animator = self.control_track_animators
            .get_mut(*track)
            .expect("track's animator should be present");
        let target_value = value;

        // immediately go to the next value
        if transition <= 0.0 {
            animator.reset(target_value, transition, ease_weight);
            self.control_track_values.insert(*track, target_value);
        }

        // push to queue
        animator.queue.push_back(TimelineControlKeyframe {
            value: target_value,
            duration: transition,
            weight: ease_weight
        });

        // hmm?
        if !animator.active && animator.queue.len() == 1 && animator.progress >= 1.0 {
            animator.start_value = animator.current_value;
            animator.target_value = animator.current_value;
        }
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