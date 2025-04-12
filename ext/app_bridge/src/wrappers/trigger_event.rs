use magnus::{prelude::*, Error, TryConvert, Value};
use crate::component::standout::app::types::TriggerEvent;

#[magnus::wrap(class = "AppBridge::TriggerEvent")]
pub struct RTriggerEvent {
    inner: TriggerEvent,
}

impl RTriggerEvent {
    pub fn new(id: String, serialized_data: String) -> Self {
        let inner: TriggerEvent = TriggerEvent {
            id: id,
            serialized_data: serialized_data,
        };
        Self { inner }
    }

    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    pub fn serialized_data(&self) -> String {
        self.inner.serialized_data.clone()
    }
}

impl TryConvert for RTriggerEvent {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let id: String = val.funcall("id", ())?;
        let serialized_data: String = val.funcall("serialized_data", ())?;

        let inner = TriggerEvent {
            id,
            serialized_data,
        };

        Ok(Self { inner })
    }
}

impl From<RTriggerEvent> for TriggerEvent {
    fn from(rtrigger_event: RTriggerEvent) -> Self {
        rtrigger_event.inner
    }
}

impl From<TriggerEvent> for RTriggerEvent {
    fn from(trigger_event: TriggerEvent) -> Self {
        Self { inner: trigger_event }
    }
}

impl From<&RTriggerEvent> for TriggerEvent {
    fn from(rtrigger_event: &RTriggerEvent) -> Self {
        rtrigger_event.inner.clone()
    }
}

impl From<&TriggerEvent> for RTriggerEvent {
    fn from(trigger_event: &TriggerEvent) -> Self {
        Self { inner: trigger_event.clone() }
    }
}
