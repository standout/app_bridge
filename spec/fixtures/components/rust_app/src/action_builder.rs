use crate::standout::app::{
    http::{
      Method,
      RequestBuilder,
    },
    types::{
      ErrorCode, ActionContext, ActionResponse, AppError
    }
};
use serde_json::Value;

pub fn http_action(action_type: &str, context: ActionContext) -> Result<ActionResponse, AppError> {
    // Parse the input to get the URL and body
    let input: Value = serde_json::from_str(&context.serialized_input)
        .map_err(|_| AppError {
            code: ErrorCode::MalformedResponse,
            message: "Invalid JSON input".to_string(),
        })?;

    let url = input["url"].as_str()
        .ok_or_else(|| AppError {
            code: ErrorCode::Misconfigured,
            message: "Missing 'url' in input".to_string(),
        })?;

    // Create the request builder with the URL
    let mut request_builder = RequestBuilder::new().url(url);

    // Configure the request based on action type
    match action_type {
        "http-get" => {
            request_builder = request_builder.method(Method::Get);
        },
        "http-post" => {
            let body = input["body"].as_str().unwrap_or("");
            request_builder = request_builder.method(Method::Post).body(body);
        },
        _ => {
            return Err(AppError {
                code: ErrorCode::Misconfigured,
                message: format!("Unsupported action type: {}", action_type),
            });
        }
    }

    // Make the HTTP request
    match request_builder.send() {
        Ok(response) => {
            // Return the response body as serialized output
            Ok(ActionResponse {
                serialized_output: response.body,
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
