use slotmap::{SecondaryMap, new_key_type};
use crate::{player::{progress::functions::{apply_timeline_control_window, set_variable_resolved_weight}, state::PlayerStateFlags}, timeline::timeline_control::{TimelineControlAnimatorState, TimelineControlBinding, TimelineControlKeyframe, TimelineControlTrackKey}};

new_key_type! { 
    pub struct TimelineKey; 
}

/// The state of a self.
pub struct TimelineState {
    pub label: String,
    pub flags: TimelineFlags,
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
    /// Progress the timeline state simply, if there's no corresponding control binding for it.
    /// 
    /// Returns whether to keep playing it.
    pub fn progress_simple(&mut self, delta: f64) -> bool {
        self.current_time += delta;
        if self.total_frames > 0.0 && self.current_time >= self.total_frames {
            if !wrap_timeline(&mut self.current_time, self.total_frames, self.loop_time) {
                self.playing = false;
                return false;
            }
        }
        return true;
    }

    /// Check if to stop playing the self.
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

    /// Progress the timeline with its corresponding control binding.
    /// 
    /// Returns whether to keep playing it.
    pub fn progress_with_control_binding(
        &mut self, 
        binding: &TimelineControlBinding, 
        delta: f64
    ) -> bool {
        let keep_playing;

        let loop_begin = binding.loop_begin;
        let loop_end = binding.loop_end;
        let last_time = if binding.last_time >= 0.0 { binding.last_time } else { self.total_frames };

        if !self.control_initialized 
        || self.control_frame_cursor.len() != binding.tracks.len() {
            self.reset_timeline_control_state(
                binding,
                f64::max(self.current_time, 0.0)
            );
        }
        if loop_begin < 0.0 {
            apply_timeline_control_window(
                self, 
                binding, 
                self.current_time + delta, 
                true
            );
            keep_playing = self.maybe_stop_playing(Some(last_time));
        }
        else if loop_end > loop_begin {
            let mut remainder = delta;
            while remainder > 0.0 && self.current_time + remainder >= loop_end {
                let current_time = self.current_time;
                let segment = f64::max(loop_end - current_time, 0.0);
                
                // resetTimelineControlState if we've finished/passed the end of the loop
                if segment <= 0.0 {
                    self.current_time = loop_begin;
                    self.reset_timeline_control_state(binding, loop_begin);
                    remainder -= f64::min(remainder, 1.0);
                    continue;
                }
                apply_timeline_control_window(
                    self, 
                    binding, 
                    loop_end, 
                    false
                );
                remainder -= segment;
                self.current_time = loop_begin;
                self.reset_timeline_control_state(binding, loop_begin);
            }

            // just in case we overran remainder?
            apply_timeline_control_window(
                self, 
                binding, 
                self.current_time + remainder, 
                true
            );
            keep_playing = self.maybe_stop_playing(None);
        }
        // loopBegin >= 0 and loopBegin <= loopEnd:
        // apply timeline control window,
        // then stop playing if end of loop or blend says to stop
        else {
            apply_timeline_control_window(
                self, 
                binding, 
                self.current_time + delta, 
                true
            );
            keep_playing = self.maybe_stop_playing(Some(last_time));
        }

        return keep_playing;
    }

    /// Reset the timeline control state.
    fn reset_timeline_control_state(
        &mut self,
        binding: &TimelineControlBinding,
        time: f64
    ) {
        self.control_frame_cursor.clear();
        self.control_track_values.clear();
        self.control_track_animators.clear();

        for (track_key, track) in &binding.tracks {
            let mut cursor = -1;
            let mut last_non_type_zero_index = None;
            for (index, frame) in track.frames.iter().enumerate() {
                if !frame.is_type_zero {
                    last_non_type_zero_index = Some(index);
                }
                if frame.time <= time {
                    cursor = index as isize;
                    continue;
                }
                break;
            }
            self.control_frame_cursor.insert(track_key, cursor);

            let last_non_type_zero_index = match last_non_type_zero_index {
                None => continue,
                Some(i) => i
            };
            let frame = &track.frames[last_non_type_zero_index];
            let next_index = last_non_type_zero_index + 1;
            let transition = if next_index < track.frames.len() {
                f64::max(track.frames[next_index].time - time - 1.0, 0.0)
            } else {
                0.0
            };
            if self.flags.and_two_isnt_zero() && !track.is_instant_variable {
                self.schedule_timeline_control_animator(
                    &track_key,
                    frame.value,
                    transition,
                    frame.easing_weight
                );
            } else {
                set_variable_resolved_weight(
                    &track_key,
                    frame,
                    transition
                );
            }
            self.control_initialized = true;
            self.control_last_applied_time = time;
        }
    }

    /// Schedule the given track's animator to transition to `value` using the weight `ease_weight`.
    fn schedule_timeline_control_animator(
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
            animator.reset(target_value, ease_weight);
            self.control_track_values.insert(*track, target_value);
        }

        // push to queue
        animator.queue.push_back(TimelineControlKeyframe {
            value: target_value,
            duration: transition,
            weight: ease_weight
        });

        // hmm? just in case i guess
        if !animator.active && animator.queue.len() == 1 && animator.progress >= 1.0 {
            animator.start_value = animator.current_value;
            animator.target_value = animator.current_value;
        }
    }

    /// Step the timeline's animators.
    /// 
    /// TODO: what is internal route though?
    fn step_internal_route(
        &mut self,
        state_flags: &mut PlayerStateFlags,
        timeline_control_binding: &TimelineControlBinding, 
        route_delta: f64
    ) {
        if self.flags.is_two() || route_delta <= 0.0 {
            return;
        }
        let blend_animating = self.blend_animator.step_queued(route_delta, &mut self.blend_ratio);
        if blend_animating {
            state_flags.emote_dirty = true;
        }

        for (track_key, track) in &timeline_control_binding.tracks {
            if track.is_instant_variable || track.frames.is_empty() {
                continue;
            }

            let stepped_value = if let Some(value) = self.control_track_values.get_mut(track_key) {
                value
            } else {
                // NOTE: this wasn't in the original code.
                // instead it just indexes into a vector and assumes the value is always there.
                // i dont quite understand things yet, so ill add this insert first
                self.control_track_values.insert(track_key, 0.0);
                self.control_track_values.get_mut(track_key).unwrap()
            };
            let animator = self.control_track_animators
                .get_mut(track_key)
                .expect("track's animator should always be present");
            let track_animating = animator
                .step_queued(route_delta, stepped_value);
            if track_animating {
                state_flags.emote_dirty = true;
            } 
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

pub struct TimelineFlags(isize);

impl TimelineFlags {
    pub fn and_four_isnt_zero(&self) -> bool {
        self.0 & 4 != 0
    }

    pub fn is_two(&self) -> bool {
        self.0 == 2
    }

    pub fn and_two_isnt_zero(&self) -> bool {
        self.0 & 2 != 0
    }
}