use slotmap::SlotMap;

use crate::{motion::{MotionEvent, MotionSnapshot}, player::{MotionClip, TimelineControlBinding}, timeline::{TimelineKey, TimelineState}};

pub struct PlayerRuntime {
    pub active_motion: MotionSnapshot,
    pub playing_timelines: Vec<TimelineKey>,
    pub timelines: SlotMap<TimelineKey, TimelineState>,
    pub timeline_control_bindings: SlotMap<TimelineKey, TimelineControlBinding>
}

impl PlayerRuntime {
    pub fn push_motion_events(&mut self, events: &mut Vec<MotionEvent>) {
        todo!()
    }

    pub fn clear_pending_events(&mut self) {
        todo!()    
    }

    pub fn select_active_clip(&self) -> Option<&MotionClip> {
        todo!()
    }
}