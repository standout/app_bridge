use crate::{
  standout::app::{http::{request, Method, OptionalHeaders}, types::{TriggerContext, TriggerResponse, TriggerEvent}},
  date_utils::timestamp
};
use serde_json::Value;

pub fn main(context: TriggerContext) -> TriggerResponse {
  match request(Method::Get, "https://jsonplaceholder.typicode.com/todos", &OptionalHeaders::None) {
    Ok(body) => {
      let json_array: Vec<Value> = serde_json::from_str(&body).unwrap_or_else(|_| vec![]);
      let events: Vec<TriggerEvent> = json_array.into_iter().map(|json_obj| {
        let id = json_obj["id"].to_string();
        let serialized_data = json_obj.to_string();

        TriggerEvent {
          id,
          timestamp: timestamp(),
          serialized_data,
        }
      }).collect();

      TriggerResponse {
        store: context.store,
        events,
      }
    },
    Err(err) => TriggerResponse {
      store: err.to_string(),
      events: vec![],
    },
  }
}
