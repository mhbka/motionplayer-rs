use slotmap::{SecondaryMap, SlotMap};

use crate::timeline::TimelineKey;

pub struct PlayerState {
    pub flags: PlayerStateFlags,
    pub values: PlayerStateValues,
    pub frame_eval_results: SlotMap<TimelineKey, f64>,
    pub animators: AnimatorStates
}

pub struct PlayerStateFlags {
    pub emote_dirty: bool,
    pub all_playing: bool,
    pub sync_active: bool,
    pub sync_waiting: bool,
    pub queueing: bool,
}

pub struct PlayerStateValues {
    pub clamped_eval_time: f64,
    pub loop_time: f64,
    pub frame_tick_count: f64,
    pub frame_loop_time: f64,
    pub frame_last_time: f64,
}

pub struct AnimatorStates {
    variable_animators: SecondaryMap<TimelineKey, VariableAnimatorState>,
    type4_animators: SecondaryMap<TimelineKey, VariableAnimatorState>,
    type5_animators: SecondaryMap<TimelineKey, VariableAnimatorState>,
    type6_animators: SecondaryMap<TimelineKey, VariableAnimatorState>,
    type7_animators: SecondaryMap<TimelineKey, VariableAnimatorState>,
    type8_animators: SecondaryMap<TimelineKey, VariableAnimatorState>
}

impl AnimatorStates {
    pub fn step(&mut self, actual_delta: f64) {
        let mut remaining_controller_step = actual_delta;
        while remaining_controller_step > 0.0 {
            let controller_dt = f64::min(remaining_controller_step, 1.1);
            Self::step_controller_bucket(&mut self.type4_animators, controller_dt);
            Self::step_controller_bucket(&mut self.type5_animators, controller_dt);
            Self::step_controller_bucket(&mut self.type6_animators, controller_dt);
            Self::step_controller_bucket(&mut self.type8_animators, controller_dt);
            Self::step_controller_bucket(&mut self.type7_animators, controller_dt);

            self.refresh_fixed_controller_eval_outputs();

            remaining_controller_step -= controller_dt;
        }
    }

    fn refresh_fixed_controller_eval_outputs(&mut self) {
        // TODO: this probably needs to be at 'Player' level as it seems to need some state 
        // check later
    }
 
    fn step_controller_bucket(animators: &mut SecondaryMap<TimelineKey, VariableAnimatorState>, dt: f64) {

    }
}

