// A simple v3 connector for backward compatibility testing
// This uses WIT v3 (without file interface)

wit_bindgen::generate!({
    path: "../../../../ext/app_bridge/wit/v3",
    world: "bridge",
    with: {},
});

use standout::app::types::{
    ActionContext, ActionResponse, AppError, ErrorCode, TriggerContext, TriggerEvent,
    TriggerResponse,
};
use standout::app::http::RequestBuilder;
use serde_json::json;

struct MyApp;

// Static storage for registered actions and triggers
static mut ACTIONS: Vec<RegisteredAction> = Vec::new();
static mut TRIGGERS: Vec<RegisteredTrigger> = Vec::new();

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

fn register_action(
    id: &'static str,
    handler: fn(ActionContext) -> Result<ActionResponse, AppError>,
    input_schema: &'static str,
    output_schema: &'static str,
) {
    unsafe {
        ACTIONS.push(RegisteredAction {
            id,
            handler,
            input_schema,
            output_schema,
        });
    }
}

fn register_trigger(
    id: &'static str,
    handler: fn(TriggerContext) -> Result<TriggerResponse, AppError>,
    input_schema: &'static str,
    output_schema: &'static str,
) {
    unsafe {
        TRIGGERS.push(RegisteredTrigger {
            id,
            handler,
            input_schema,
            output_schema,
        });
    }
}

// Simple HTTP action
fn http_action(context: ActionContext) -> Result<ActionResponse, AppError> {
    let input: serde_json::Value = serde_json::from_str(&context.serialized_input)
        .map_err(|_| AppError {
            code: ErrorCode::MalformedResponse,
            message: "Invalid JSON input".to_string(),
        })?;

    let url = input.get("url").and_then(|v| v.as_str()).unwrap_or("https://httpbin.org/get");
    
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
    let events = vec![
        TriggerEvent {
            id: "event-1".to_string(),
            serialized_data: json!({ "message": "Hello from v3 connector" }).to_string(),
        },
    ];

    Ok(TriggerResponse {
        store: context.store,
        events,
    })
}

fn register_all() {
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

    register_action("http-get", http_action, http_input_schema, http_output_schema);

    let trigger_input_schema = r#"{ "type": "object" }"#;
    let trigger_output_schema = r#"{ "type": "object", "properties": { "message": { "type": "string" } } }"#;
    
    register_trigger("simple-trigger", simple_trigger, trigger_input_schema, trigger_output_schema);
}

export!(MyApp);

// Actions implementation
impl exports::standout::app::actions::Guest for MyApp {
    fn action_ids() -> Result<Vec<String>, AppError> {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| register_all());
        
        unsafe { Ok(ACTIONS.iter().map(|a| a.id.to_string()).collect()) }
    }

    fn input_schema(context: ActionContext) -> Result<String, AppError> {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| register_all());
        
        unsafe {
            ACTIONS
                .iter()
                .find(|a| a.id == context.action_id)
                .map(|a| a.input_schema.to_string())
                .ok_or_else(|| AppError {
                    code: ErrorCode::Misconfigured,
                    message: format!("Action '{}' not found", context.action_id),
                })
        }
    }

    fn output_schema(context: ActionContext) -> Result<String, AppError> {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| register_all());
        
        unsafe {
            ACTIONS
                .iter()
                .find(|a| a.id == context.action_id)
                .map(|a| a.output_schema.to_string())
                .ok_or_else(|| AppError {
                    code: ErrorCode::Misconfigured,
                    message: format!("Action '{}' not found", context.action_id),
                })
        }
    }

    fn execute(context: ActionContext) -> Result<ActionResponse, AppError> {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| register_all());
        
        unsafe {
            ACTIONS
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
}

// Triggers implementation
impl exports::standout::app::triggers::Guest for MyApp {
    fn trigger_ids() -> Result<Vec<String>, AppError> {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| register_all());
        
        unsafe { Ok(TRIGGERS.iter().map(|t| t.id.to_string()).collect()) }
    }

    fn input_schema(context: TriggerContext) -> Result<String, AppError> {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| register_all());
        
        unsafe {
            TRIGGERS
                .iter()
                .find(|t| t.id == context.trigger_id)
                .map(|t| t.input_schema.to_string())
                .ok_or_else(|| AppError {
                    code: ErrorCode::Misconfigured,
                    message: format!("Trigger '{}' not found", context.trigger_id),
                })
        }
    }

    fn output_schema(context: TriggerContext) -> Result<String, AppError> {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| register_all());
        
        unsafe {
            TRIGGERS
                .iter()
                .find(|t| t.id == context.trigger_id)
                .map(|t| t.output_schema.to_string())
                .ok_or_else(|| AppError {
                    code: ErrorCode::Misconfigured,
                    message: format!("Trigger '{}' not found", context.trigger_id),
                })
        }
    }

    fn fetch_events(context: TriggerContext) -> Result<TriggerResponse, AppError> {
        static INIT: std::sync::Once = std::sync::Once::new();
        INIT.call_once(|| register_all());
        
        unsafe {
            TRIGGERS
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
}
