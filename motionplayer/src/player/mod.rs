mod state;
mod runtime;use slotmap::{SecondaryMap, SlotMap, new_key_type};
use crate::{motion::MotionSnapshot, player::{runtime::PlayerRuntime, state::PlayerState}, timeline::{TimelineKey, TimelineState}};

const MOTION_FRAMES_PER_MS: f64 = 60.0 / 1000.0;

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

        self.state.frame_last_time = dt;
        self.state.frame_eval_results.clear();
        
        if self.state.queueing {
            self.state.all_playing = !self.runtime.playing_timelines.is_empty();
            self.state.sync_active = self.state.sync_waiting && self.state.all_playing;
            return;
        }

        self.state.frame_loop_time += actual_delta;
        self.state.loop_time += actual_delta;
        self.state.frame_tick_count += actual_delta;

        let mut previous_times: SecondaryMap<TimelineKey, f64> = SecondaryMap::new();
        self.preprogress_playing_timelines(actual_delta, &mut previous_times);

        // NOTE: put this in its own fn
        self.state.animators.step(actual_delta);

        self.apply_eval_result_post_process();

        if let Some(active_clip) = self.runtime.select_active_clip() {
            self.state.clamped_eval_time = Self::active_clip_time(&self.runtime, active_clip);
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

        self.state.all_playing = !self.runtime.playing_timelines.is_empty();
        self.state.sync_active = self.state.sync_waiting && self.state.all_playing;
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

        for timeline_key in &self.runtime.playing_timelines {
            let timeline = if let Some(timeline) = self.runtime.timelines.get_mut(*timeline_key) {
                timeline
            } else {
                continue;
            };

            previous_times.insert(*timeline_key, timeline.current_time);

            if !timeline.playing { 
                continue;
            }

            timeline.was_playing = true;
            let mut keep_playing = true;

            match self.runtime.timeline_control_bindings.get(*timeline_key) {
                None => {
                    keep_playing = timeline.no_control_binding_update(delta);
                },
                Some(timeline_control_binding) => {

                }
            }
        }
    }

    fn apply_eval_result_post_process(&mut self) {
        todo!()
    }

    fn active_clip_time(runtime: &PlayerRuntime, clip: &MotionClip) -> f64 {
        todo!()
    }
}

pub struct MotionClip {

}

pub struct TimelineControlBinding {

}