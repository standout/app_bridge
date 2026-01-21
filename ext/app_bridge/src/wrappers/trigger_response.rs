use magnus::{Error, Ruby, TryConvert};
use crate::types::TriggerResponse;
use super::trigger_event::RTriggerEvent;

#[magnus::wrap(class = "AppBridge::TriggerResponse")]
pub struct RTriggerResponse {
    inner: TriggerResponse,
}

impl RTriggerResponse {
    pub fn new(store: String, events: magnus::RArray) -> Self {
        let iter = events.into_iter();
        let res: Vec<RTriggerEvent> = iter
            .map(&TryConvert::try_convert)
            .collect::<Result<Vec<RTriggerEvent>, Error>>()
            .unwrap();

        let inner = TriggerResponse {
            store,
            events: res.iter().map(|e| e.into()).collect(),
        };
        Self { inner }
    }

    pub fn store(&self) -> String {
        self.inner.store.clone()
    }

    pub fn events(&self) -> magnus::RArray {
        let ruby = Ruby::get().unwrap();
        let array = ruby.ary_new();
        for e in &self.inner.events {
            let _ = array.push(RTriggerEvent::from(e));
        }
        array
    }
}

impl From<TriggerResponse> for RTriggerResponse {
    fn from(value: TriggerResponse) -> Self {
        Self { inner: value }
    }
}
