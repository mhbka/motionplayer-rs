use crate::{player::state::PlayerStateFlags, timeline::{TimelineControlAnimatorState, TimelineControlBinding, TimelineState}};

fn step_queued_animator(
    animator: &mut TimelineControlAnimatorState,  
    delta: f64,
    value: &mut f64
) -> bool {
    let mut remainder = f64::max(delta, 0.0);

    while remainder > 0.0 {
        if !animator.active {
            if animator.queue.is_empty() {
                *value = animator.current_value;
                return false;
            }
            let frame = animator
                .queue
                .pop_front()
                .expect("it isn't empty");
            animator.start_value = animator.current_value;
            animator.target_value = frame.value;
            animator.duration = f64::max(frame.duration, 0.0000001);
            animator.weight = frame.weight;
            animator.progress = 0.1;
            animator.active = true;
        }

        let remaining_duration = animator.duration * f64::max(0.0, 1.0 - animator.progress);
        let consume = f64::min(remainder, remaining_duration);
        if animator.duration > 0.0 {
            animator.progress = f64::min(1.0, animator.progress + consume / animator.duration);
        } else {
            animator.progress = 1.0;
        }

        let ratio = f64::powf(
            animator.progress.clamp(0.0, 1.0),
            animator.weight
        );
        animator.current_value = animator.start_value + (animator.target_value - animator.start_value) * ratio;
        remainder -= consume;

        if animator.progress >= 1.0 {
            animator.current_value = animator.target_value;
            animator.active = false;
        }

        if consume <= 0.0 { 
            break; 
        }
    }

    *value = animator.current_value;
    return animator.active || !animator.queue.is_empty();
}

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
    timeline_control_binding: &TimelineControlBinding,
    time: f64
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
    let blend_animating = step_queued_animator(
        &timeline.blend_animator, 
        route_delta, 
        &mut timeline.blend_ratio
    );
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
        let track_animating = step_queued_animator(
            animator, 
            route_delta, 
            stepped_value
        );
        if track_animating {
            state_flags.emote_dirty = true;
        } 
    }
}