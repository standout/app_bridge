use magnus::{Error, RArray, TryConvert};
use crate::component::standout::app::types::TriggerResponse;
use super::trigger_event::RTriggerEvent;

#[magnus::wrap(class = "AppBridge::TriggerResponse")]
pub struct RTriggerResponse {
  inner: TriggerResponse,
}

impl RTriggerResponse {
  pub fn new(store: String, events: RArray) -> Self {
      let iter = events.into_iter();
      let res: Vec<RTriggerEvent> = iter
          .map(&TryConvert::try_convert)
          .collect::<Result<Vec<RTriggerEvent>, Error>>()
          .unwrap();

      let inner = TriggerResponse {
          store: store,
          events: res.iter().map(|e| e.into()).collect(),
      };
      Self { inner }
  }

  pub fn store(&self) -> String {
      self.inner.store.clone()
  }

  pub fn events(&self) -> RArray {
      self.inner
          .events
          .iter()
          .map(|e| RTriggerEvent::from(e))
          .collect()
  }
}

impl From<TriggerResponse> for RTriggerResponse {
  fn from(value: TriggerResponse) -> Self {
      Self { inner: value }
  }
}
