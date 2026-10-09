mod state;
mod runtime;use slotmap::{SecondaryMap, SlotMap, new_key_type};
use crate::{motion::{MotionEvent, MotionSnapshot}, player::{runtime::PlayerRuntime, state::{PlayerState, PlayerStateFlags}}, timeline::{TimelineControlAnimatorState, TimelineControlBinding, TimelineKey, TimelineState}};

const MOTION_FRAMES_PER_MS: f64 = 60.0 / 1000.0;

pub mod progress;

pub struct Player {
    runtime: PlayerRuntime,
    state: PlayerState
}

impl Player {
    pub fn progress(&mut self, mut delta_ms: f64) {
        /*
         * NOTE: skipped progressEmoteLike_sdl3 etc
         */

        if delta_ms < 0.0 || delta_ms > 60000.0 {
            delta_ms = 0.0;
        }
        
        self.runtime.clear_pending_events();

        self.progress_frame(delta_ms * MOTION_FRAMES_PER_MS);

        self.update_layers();

        self.calc_bounds();

        self.runtime.clear_pending_events();
    }

    fn progress_frame(&mut self, dt: f64) {
        let actual_delta = dt;

        self.state.values.frame_last_time = dt;
        self.state.frame_eval_results.clear();
        
        if self.state.flags.queueing {
            self.state.flags.all_playing = !self.runtime.playing_timelines.is_empty();
            self.state.flags.sync_active = self.state.flags.sync_waiting && self.state.flags.all_playing;
            return;
        }

        self.state.values.frame_loop_time += actual_delta;
        self.state.values.loop_time += actual_delta;
        self.state.values.frame_tick_count += actual_delta;

        let mut previous_times: SecondaryMap<TimelineKey, f64> = SecondaryMap::new();
        self.preprogress_playing_timelines(actual_delta, &mut previous_times);

        // NOTE: put this in its own fn
        self.state.animators.step(actual_delta);

        self.apply_eval_result_post_process();

        if let Some(active_clip) = self.runtime.select_active_clip() {
            self.state.values.clamped_eval_time = active_clip_time(&self.runtime, active_clip);
        }

        // Scan PSB layers for action/sync events crossed this frame
        if actual_delta > 0.0 {
            for (timeline_key, previous_time) in previous_times {
                if let Some(timeline) = self.runtime.timelines.get(timeline_key) {
                    if timeline.current_time > previous_time {
                        todo!("in c++, below fn accepts `new_time`, but when called, there is no `new_time`, WTF?")
                        //self.runtime.active_motion.scan_layer_actions(previous_time, new_time, events);
                    }
                }
            }
        }

        self.state.flags.all_playing = !self.runtime.playing_timelines.is_empty();
        self.state.flags.sync_active = self.state.flags.sync_waiting && self.state.flags.all_playing;
    }

    fn update_layers(&mut self) {
        todo!()
    }

    fn calc_bounds(&mut self) {
        todo!()
    }

    fn preprogress_playing_timelines(&mut self, delta: f64, previous_times: &mut SecondaryMap<TimelineKey, f64>) {
        if delta <= 0.0 { 
            return;
        }

        let Self { 
            runtime, 
            state 
        } = self;

        let mut preprogress_motion_events = Vec::with_capacity(runtime.playing_timelines.len());

        for timeline_key in &runtime.playing_timelines {
            let timeline = if let Some(timeline) = runtime.timelines.get_mut(*timeline_key) {
                timeline
            } else {
                continue;
            };

            previous_times.insert(*timeline_key, timeline.current_time);
            if !timeline.playing { 
                continue;
            }

            timeline.was_playing = true;
            let keep_playing;

            match runtime.timeline_control_bindings.get(*timeline_key) {
                None => {
                    keep_playing = timeline.progress_basic(delta);
                },
                Some(timeline_control_binding) => {
                    keep_playing = Self::update_timeline_with_control_binding(
                        timeline, 
                        timeline_control_binding, 
                        delta
                    );
                }
            }

            preprogress_motion_events.push(MotionEvent {
                // TODO FILL THIS OUT
            });

            if timeline.playing && !keep_playing {
                timeline.was_playing = false;
            }
        }

        runtime.push_motion_events(&mut preprogress_motion_events);
    }

    fn apply_eval_result_post_process(&mut self) {
        todo!()
    }

    fn update_timeline_with_control_binding(
        timeline: &mut TimelineState, 
        binding: &TimelineControlBinding, 
        delta: f64
    ) -> bool {
        let keep_playing;

        let loop_begin = binding.loop_begin;
        let loop_end = binding.loop_end;
        let last_time = if binding.last_time >= 0.0 { binding.last_time } else { timeline.total_frames };

        if !timeline.control_initialized 
        || timeline.control_frame_cursor.len() != binding.tracks.len() {
            reset_timeline_control_state(
                timeline, 
                binding,
                f64::max(timeline.current_time, 0.0)
            );
        }
        if loop_begin < 0.0 {
            apply_timeline_control_window(
                timeline, 
                binding, 
                timeline.current_time + delta, 
                true
            );
            keep_playing = timeline.maybe_stop_playing(Some(last_time));
        }
        else if loop_end > loop_begin {
            let mut remainder = delta;
            while remainder > 0.0 && timeline.current_time + remainder >= loop_end {
                let current_time = timeline.current_time;
                let segment = f64::max(loop_end - current_time, 0.0);
                
                // resetTimelineControlState if we've finished/passed the end of the loop
                if segment <= 0.0 {
                    timeline.current_time = loop_begin;
                    reset_timeline_control_state(
                        timeline, 
                        binding, 
                        loop_begin
                    );
                    remainder -= f64::min(remainder, 1.0);
                    continue;
                }
                apply_timeline_control_window(
                    timeline, 
                    binding, 
                    loop_end, 
                    false
                );
                remainder -= segment;
                timeline.current_time = loop_begin;
                reset_timeline_control_state(
                    timeline, 
                    binding, 
                    loop_begin
                );
            }

            // just in case we overran remainder?
            apply_timeline_control_window(
                timeline, 
                binding, 
                timeline.current_time + remainder, 
                true
            );
            keep_playing = timeline.maybe_stop_playing(None);
        }
        // loopBegin >= 0 and loopBegin <= loopEnd:
        // apply timeline control window,
        // then stop playing if end of loop or blend says to stop
        else {
            apply_timeline_control_window(
                timeline, 
                binding, 
                timeline.current_time + delta, 
                true
            );
            keep_playing = timeline.maybe_stop_playing(Some(last_time));
        }

        return keep_playing;
    }
}

fn active_clip_time(runtime: &PlayerRuntime, clip: &MotionClip) -> f64 {
    todo!()
}



pub struct MotionClip {

}