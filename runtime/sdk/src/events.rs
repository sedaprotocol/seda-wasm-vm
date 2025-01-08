use serde::{Deserialize, Serialize};

use crate::promises::VmCallData;

pub type EventId = String;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub enum EventData {
    Vm(VmCallData),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub struct Event {
    pub id:               EventId,
    pub data:             EventData,
    /// Allows the system to check if this event is in queue and does not add it to the queue when true
    /// When false, the event will always be added to the queue regardless if it's already running or not
    pub check_duplicates: bool,
}

impl Event {
    pub fn new<T: ToString>(id: T, data: EventData) -> Self {
        Self {
            id: id.to_string(),
            data,
            check_duplicates: true,
        }
    }

    pub fn set_check_duplicates(&mut self, check_duplicates: bool) {
        self.check_duplicates = check_duplicates;
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_event() {
        let mut event = Event::new("test", EventData::Vm(VmCallData::default()));
        assert_eq!(event.id, "test");
        assert_eq!(event.data, EventData::Vm(VmCallData::default()));
        assert!(event.check_duplicates);
        event.set_check_duplicates(false);
        assert!(!event.check_duplicates);
    }
}
