use crate::{player::state::PlayerStateFlags, timeline::{timeline::TimelineState, timeline_control::{TimelineControlAnimatorState, TimelineControlBinding, TimelineControlFrame, TimelineControlTrackKey}}};

pub fn apply_timeline_control_window(
    timeline: &mut TimelineState,
    binding: &TimelineControlBinding,
    target_time: f64,
    inclusive_end: bool
) {
    timeline.control_frame_cursor.clear();
    timeline.control_track_values.clear();
    timeline.control_track_animators.clear();

    for (track_key, track) in &binding.tracks {
        if track.frames.is_empty() {
            continue;
        }
        if timeline.flags.and_four_isnt_zero() && track.is_instant_variable {
            continue;
        }

        let internal_route = timeline.flags.and_two_isnt_zero() && !track.is_instant_variable;
        for (cursor_track_key, cursor) in &timeline.control_frame_cursor {
            // TODO!!!
        }
    }
}

pub fn set_variable_resolved_weight(
    track: &TimelineControlTrackKey,
    frame: &TimelineControlFrame,
    transition: f64
) {
    todo!()
}

pub fn apply_eval_result_post_process() {
    todo!()
}