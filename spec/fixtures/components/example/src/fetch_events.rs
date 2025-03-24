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
  // Parse the offset from context.store, default to 0 if invalid
  let offset: usize = context.store.parse().unwrap_or(0);

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

      // Apply offset and limit to get the first 10 events
      let paginated_events = events.into_iter().skip(offset).take(10).collect();

      TriggerResponse {
        store: (offset + 10).to_string(),
        events: paginated_events,
      }
    },
    Err(err) => TriggerResponse {
      store: err.to_string(),
      events: vec![],
    },
  }
}
