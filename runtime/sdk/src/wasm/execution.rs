use super::raw;
use crate::{events::Event, Bytes, TriggerEventAction};

pub fn execution_result(result: Bytes) {
    let result_length = result.len() as i32;

    unsafe {
        raw::execution_result(result.as_ptr(), result_length);
    }
}

/// Triggers an event on the host node
/// Allows you to resolve data requests, sign blocks but at a later stage
pub fn trigger_event(event: Event, delay_in_ms: Option<u32>) {
    let trigger_event_action = TriggerEventAction { event, delay_in_ms };

    let action = serde_json::to_string(&trigger_event_action).unwrap();

    unsafe { raw::trigger_event(action.as_ptr(), action.len() as u32) };
}
