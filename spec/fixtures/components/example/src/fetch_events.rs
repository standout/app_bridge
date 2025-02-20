use crate::{
  date_utils::timestamp,
  standout::app::{
    http::{
      Method,
      RequestBuilder,
    },
    types::{
      TriggerContext,
      TriggerEvent,
      TriggerResponse
    }
  }
};
use serde_json::Value;

pub fn main(context: TriggerContext) -> TriggerResponse {

  let response = RequestBuilder::new()
    .method(Method::Get)
    .url("https://jsonplaceholder.typicode.com/todos");

  match response.send() {
    Ok(response) => {
      let json_array: Vec<Value> = serde_json::from_str(&response.body).unwrap_or_else(|_| vec![]);
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
