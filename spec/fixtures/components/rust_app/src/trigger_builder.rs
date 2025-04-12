use crate::standout::app::{
    http::{
      Method,
      RequestBuilder,
    },
    types::{
      ErrorCode, TriggerContext, TriggerEvent, TriggerResponse, AppError
    }
};
use serde_json::Value;

pub fn get_jsonplaceholder(resource: &str, context: TriggerContext) -> Result<TriggerResponse, AppError> {
    // Parse the offset from context.store, default to 0 if invalid
    let offset: usize = context.store.parse().unwrap_or(0);

    let response = RequestBuilder::new()
      .method(Method::Get)
      .url(&format!("https://jsonplaceholder.typicode.com/{}", resource));

    match response.send() {
      Ok(response) => {
        let json_array: Vec<Value> = serde_json::from_str(&response.body).unwrap_or_else(|_| vec![]);
        let events: Vec<TriggerEvent> = json_array.into_iter().map(|json_obj| {
          let id = json_obj["id"].to_string();
          let serialized_data = json_obj.to_string();

          TriggerEvent {
            id,
            serialized_data,
          }
        }).collect();

        // Apply offset and limit to get the first 10 events
        let paginated_events = events.into_iter().skip(offset).take(10).collect();

        Ok(TriggerResponse {
          store: (offset + 10).to_string(),
          events: paginated_events,
        })
      },
      Err(err) => {
        let error_message = format!("Request failed: {}", err);
        Err(AppError {
          code: ErrorCode::Other,
          message: error_message,
        })
      },
    }
}
