use crate::{player::state::PlayerStateFlags, timeline::{timeline::TimelineState, timeline_control::{TimelineControlAnimatorState, TimelineControlBinding, TimelineControlFrame, TimelineControlTrackKey}}};

fn apply_timeline_control_window(
    timeline: &mut TimelineState,
    timeline_control_binding: &TimelineControlBinding,
    target_time: f64,
    inclusive_end: bool
) {
    todo!()
}

fn reset_timeline_control_state(
    timeline: &mut TimelineState,
    binding: &TimelineControlBinding,
    time: f64
) {
    timeline.control_frame_cursor.clear();
    timeline.control_track_values.clear();
    timeline.control_track_animators.clear();

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
        timeline.control_frame_cursor.insert(track_key, cursor);

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
        if (timeline.flags & 2) != 0 && !track.is_instant_variable {
            timeline.schedule_timeline_control_animator(
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
        timeline.control_initialized = true;
        timeline.control_last_applied_time = time;
    }
}

fn set_variable_resolved_weight(
    track: &TimelineControlTrackKey,
    frame: &TimelineControlFrame,
    transition: f64
) {
    todo!()
}

fn step_internal_route(
    state_flags: &mut PlayerStateFlags,
    timeline: &mut TimelineState, 
    timeline_control_binding: &TimelineControlBinding, 
    route_delta: f64
) {
    // what is this flags thing
    if timeline.flags == 2 || route_delta <= 0.0 {
        return;
    }
    let blend_animating = timeline.blend_animator.step_queued(route_delta, &mut timeline.blend_ratio);
    if blend_animating {
        state_flags.emote_dirty = true;
    }

    for (track_key, track) in &timeline_control_binding.tracks {
        if track.is_instant_variable || track.frames.is_empty() {
            continue;
        }

        let stepped_value = if let Some(value) = timeline.control_track_values.get_mut(track_key) {
            value
        } else {
            // NOTE: this wasn't in the original code.
            // instead it just indexes into a vector and assumes the value is always there.
            // i dont quite understand things yet, so ill add this insert first
            timeline.control_track_values.insert(track_key, 0.0);
            timeline.control_track_values.get_mut(track_key).unwrap()
        };
        let animator = timeline
            .control_track_animators
            .get_mut(track_key)
            .expect("track's animator should always be present");
        let track_animating = animator
            .step_queued(route_delta, stepped_value);
        if track_animating {
            state_flags.emote_dirty = true;
        } 
    }
}