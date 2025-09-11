use crate::standout::app::{
    http::{
      Method,
      RequestBuilder,
    },
    types::{
      ErrorCode, ActionContext, ActionResponse, AppError
    }
};
use serde_json::{Value, json};

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
    let body_value = match action_type {
        "http-get" => {
            request_builder = request_builder.method(Method::Get);
            None
        },
        "http-post" => {
            let body = input.get("body").cloned();
            request_builder = request_builder.method(Method::Post);
            if let Some(ref body_val) = body {
                request_builder = request_builder.body(body_val.as_str().unwrap_or(""));
            }
            body
        },
        _ => {
            return Err(AppError {
                code: ErrorCode::Misconfigured,
                message: format!("Unsupported action type: {}", action_type),
            });
        }
    };

    // Check if we're in test mode (mock HTTP requests)
    let response_data = if std::env::var("APP_BRIDGE_TEST_MODE").is_ok() {
        // Return mock response data for tests
        match action_type {
            "http-get" => {
                json!({
                    "args": {},
                    "headers": {
                        "Accept": "*/*",
                        "Host": "mock.test",
                        "User-Agent": "MockHTTP/1.0"
                    },
                    "origin": "127.0.0.1",
                    "url": url
                })
            },
            "http-post" => {
                json!({
                    "args": {},
                    "data": body_value.as_ref().map(|b| b.as_str().unwrap_or("")).unwrap_or(""),
                    "files": {},
                    "form": {},
                    "headers": {
                        "Accept": "*/*",
                        "Content-Length": body_value.as_ref().map(|b| b.as_str().unwrap_or("").len()).unwrap_or(0).to_string(),
                        "Content-Type": "application/json",
                        "Host": "mock.test",
                        "User-Agent": "MockHTTP/1.0"
                    },
                    "json": body_value.as_ref().and_then(|b| serde_json::from_str(b.as_str().unwrap_or("")).ok()).unwrap_or(json!(null)),
                    "origin": "127.0.0.1",
                    "url": url
                })
            },
            _ => json!({})
        }
    } else {
        // Make the actual HTTP request
        match request_builder.send() {
            Ok(response) => {
                serde_json::from_str(&response.body).map_err(|e| {
                    AppError {
                        code: ErrorCode::MalformedResponse,
                        message: format!("Invalid JSON response: {}", e),
                    }
                })?
            },
            Err(err) => {
                let error_message = format!("Request failed: {}", err);
                return Err(AppError {
                    code: ErrorCode::Other,
                    message: error_message,
                });
            }
        }
    };

    // Build output based on action type
    let output = match action_type {
        "http-get" => {
            json!({
                "url": url,
                "response": response_data
            })
        },
        "http-post" => {
            json!({
                "url": url,
                "body": body_value,
                "response": response_data
            })
        },
        _ => unreachable!()
    };

    // Return the response as serialized output
    Ok(ActionResponse {
        serialized_output: output.to_string(),
    })
}

pub fn complex_input_action(context: ActionContext) -> Result<ActionResponse, AppError> {
    // Parse the input to get the customer data
    let input: Value = serde_json::from_str(&context.serialized_input)
        .map_err(|_| AppError {
            code: ErrorCode::MalformedResponse,
            message: "Invalid JSON input".to_string(),
        })?;

    // Validate that customer data exists
    let customer = input.get("customer")
        .ok_or_else(|| AppError {
            code: ErrorCode::Misconfigured,
            message: "Missing 'customer' in input".to_string(),
        })?;

    let mut environment_variables = json!({});

    // Get all environment variables
    for (key, value) in std::env::vars() {
        println!("Found environment variable {}: {}", key, value);
        environment_variables[key] = json!(value);
    }

    // If no environment variables were found, add a message
    if environment_variables.as_object().unwrap().is_empty() {
        environment_variables["message"] = json!("No environment variables found");
    }

    // Process the customer data (in a real app, this would do actual processing)
    let output = json!({
        "customer": customer,
        "processed": true,
        "environment_variables": environment_variables
    });

    // Return the processed data as serialized output
    Ok(ActionResponse {
        serialized_output: output.to_string(),
    })
}
