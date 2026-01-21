use crate::standout::app::{
    http::{
      Method,
      RequestBuilder,
    },
    file::normalize as file_normalize,
    types::{
      ErrorCode, ActionContext, ActionResponse, AppError
    }
};
use serde_json::{Value, json};

// Note: file_normalize returns FileData { base64, content_type, filename }
// It automatically detects input type (URL, data URI, base64)

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
                "body": {
                    "content": body_value.as_ref().and_then(|v| v.as_str()).unwrap_or(""),
                    "content_type": "application/json"
                },
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

/// File normalize action - normalizes any file source to FileData
/// Automatically detects input type: URL, data URI, or base64
/// The platform will process this based on output schema and replace with blob ID
pub fn file_normalize_action(context: ActionContext) -> Result<ActionResponse, AppError> {
    let input: Value = serde_json::from_str(&context.serialized_input)
        .map_err(|_| AppError {
            code: ErrorCode::MalformedResponse,
            message: "Invalid JSON input".to_string(),
        })?;

    // Get source - can be url, base64, data_uri, or the generic "source" field
    let source = input.get("source")
        .or_else(|| input.get("url"))
        .or_else(|| input.get("base64"))
        .or_else(|| input.get("data_uri"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError {
            code: ErrorCode::Misconfigured,
            message: "Missing 'source' in input".to_string(),
        })?;

    // Get optional headers as Vec<(String, String)> for URL requests
    let headers: Option<Vec<(String, String)>> = input.get("headers")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|item| {
                    let pair = item.as_array()?;
                    if pair.len() == 2 {
                        Some((
                            pair[0].as_str()?.to_string(),
                            pair[1].as_str()?.to_string(),
                        ))
                    } else {
                        None
                    }
                })
                .collect()
        });

    // Get optional filename override
    let filename = input.get("filename").and_then(|v| v.as_str()).map(|s| s.to_string());

    // Use file.normalize - automatically detects input type
    // Returns FileData { base64, content_type, filename }
    // The platform will find fields with format: "file-output" in the schema
    // and replace this data with the blob ID from file_uploader
    let file_data = file_normalize(
        source,
        headers.as_ref().map(|v| v.as_slice()),
        filename.as_deref(),
    )
    .map_err(|e| AppError {
        code: ErrorCode::Other,
        message: format!("Failed to normalize file: {:?}", e),
    })?;

    // Output the file data - schema should mark "file" with format: "file-output"
    let output = json!({
        "file": {
            "base64": file_data.base64,
            "content_type": file_data.content_type,
            "filename": file_data.filename
        }
    });

    Ok(ActionResponse {
        serialized_output: output.to_string(),
    })
}
