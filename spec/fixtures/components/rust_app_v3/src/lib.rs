// A simple v3 connector for backward compatibility testing
// This uses WIT v3 (without file interface)

wit_bindgen::generate!({
    path: "../../../../ext/app_bridge/wit/v3",
    world: "bridge",
    with: {},
});

use std::sync::OnceLock;
use standout::app::types::{
    ActionContext, ActionResponse, AppError, ErrorCode, TriggerContext, TriggerEvent,
    TriggerResponse,
};
use standout::app::http::RequestBuilder;
use serde_json::json;

struct MyApp;

// Thread-safe static storage using OnceLock
static ACTIONS: OnceLock<Vec<RegisteredAction>> = OnceLock::new();
static TRIGGERS: OnceLock<Vec<RegisteredTrigger>> = OnceLock::new();

struct RegisteredAction {
    id: &'static str,
    handler: fn(ActionContext) -> Result<ActionResponse, AppError>,
    input_schema: &'static str,
    output_schema: &'static str,
}

struct RegisteredTrigger {
    id: &'static str,
    handler: fn(TriggerContext) -> Result<TriggerResponse, AppError>,
    input_schema: &'static str,
    output_schema: &'static str,
}

fn get_actions() -> &'static Vec<RegisteredAction> {
    ACTIONS.get_or_init(|| {
        let http_input_schema = r#"{
            "type": "object",
            "properties": {
                "url": { "type": "string" }
            }
        }"#;

        let http_output_schema = r#"{
            "type": "object",
            "properties": {
                "status": { "type": "integer" },
                "body": { "type": "string" }
            }
        }"#;

        vec![RegisteredAction {
            id: "http-get",
            handler: http_action,
            input_schema: http_input_schema,
            output_schema: http_output_schema,
        }]
    })
}

fn get_triggers() -> &'static Vec<RegisteredTrigger> {
    TRIGGERS.get_or_init(|| {
        let trigger_input_schema = r#"{ "type": "object" }"#;
        let trigger_output_schema =
            r#"{ "type": "object", "properties": { "message": { "type": "string" } } }"#;

        vec![RegisteredTrigger {
            id: "simple-trigger",
            handler: simple_trigger,
            input_schema: trigger_input_schema,
            output_schema: trigger_output_schema,
        }]
    })
}

// Simple HTTP action
fn http_action(context: ActionContext) -> Result<ActionResponse, AppError> {
    let input: serde_json::Value = serde_json::from_str(&context.serialized_input).map_err(|_| {
        AppError {
            code: ErrorCode::MalformedResponse,
            message: "Invalid JSON input".to_string(),
        }
    })?;

    let url = input
        .get("url")
        .and_then(|v| v.as_str())
        .unwrap_or("https://httpbin.org/get");

    let response = RequestBuilder::new()
        .method(standout::app::http::Method::Get)
        .url(url)
        .send()
        .map_err(|e| AppError {
            code: ErrorCode::Other,
            message: format!("HTTP request failed: {:?}", e),
        })?;

    let output = json!({
        "status": response.status,
        "body": response.body,
    });

    Ok(ActionResponse {
        serialized_output: output.to_string(),
    })
}

// Simple trigger
fn simple_trigger(context: TriggerContext) -> Result<TriggerResponse, AppError> {
    let events = vec![TriggerEvent {
        id: "event-1".to_string(),
        serialized_data: json!({ "message": "Hello from v3 connector" }).to_string(),
    }];

    Ok(TriggerResponse {
        store: context.store,
        events,
    })
}

export!(MyApp);

// Actions implementation
impl exports::standout::app::actions::Guest for MyApp {
    fn action_ids() -> Result<Vec<String>, AppError> {
        Ok(get_actions().iter().map(|a| a.id.to_string()).collect())
    }

    fn input_schema(context: ActionContext) -> Result<String, AppError> {
        get_actions()
            .iter()
            .find(|a| a.id == context.action_id)
            .map(|a| a.input_schema.to_string())
            .ok_or_else(|| AppError {
                code: ErrorCode::Misconfigured,
                message: format!("Action '{}' not found", context.action_id),
            })
    }

    fn output_schema(context: ActionContext) -> Result<String, AppError> {
        get_actions()
            .iter()
            .find(|a| a.id == context.action_id)
            .map(|a| a.output_schema.to_string())
            .ok_or_else(|| AppError {
                code: ErrorCode::Misconfigured,
                message: format!("Action '{}' not found", context.action_id),
            })
    }

    fn execute(context: ActionContext) -> Result<ActionResponse, AppError> {
        get_actions()
            .iter()
            .find(|a| a.id == context.action_id)
            .map(|a| (a.handler)(context.clone()))
            .unwrap_or_else(|| {
                Err(AppError {
                    code: ErrorCode::Misconfigured,
                    message: format!("Action '{}' not found", context.action_id),
                })
            })
    }
}

// Triggers implementation
impl exports::standout::app::triggers::Guest for MyApp {
    fn trigger_ids() -> Result<Vec<String>, AppError> {
        Ok(get_triggers().iter().map(|t| t.id.to_string()).collect())
    }

    fn input_schema(context: TriggerContext) -> Result<String, AppError> {
        get_triggers()
            .iter()
            .find(|t| t.id == context.trigger_id)
            .map(|t| t.input_schema.to_string())
            .ok_or_else(|| AppError {
                code: ErrorCode::Misconfigured,
                message: format!("Trigger '{}' not found", context.trigger_id),
            })
    }

    fn output_schema(context: TriggerContext) -> Result<String, AppError> {
        get_triggers()
            .iter()
            .find(|t| t.id == context.trigger_id)
            .map(|t| t.output_schema.to_string())
            .ok_or_else(|| AppError {
                code: ErrorCode::Misconfigured,
                message: format!("Trigger '{}' not found", context.trigger_id),
            })
    }

    fn fetch_events(context: TriggerContext) -> Result<TriggerResponse, AppError> {
        get_triggers()
            .iter()
            .find(|t| t.id == context.trigger_id)
            .map(|t| (t.handler)(context.clone()))
            .unwrap_or_else(|| {
                Err(AppError {
                    code: ErrorCode::Misconfigured,
                    message: format!("Trigger '{}' not found", context.trigger_id),
                })
            })
    }
}
