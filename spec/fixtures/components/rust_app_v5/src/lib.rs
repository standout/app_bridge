// v5 connector fixture with connections interface for testing

wit_bindgen::generate!({
    path: "../../../../ext/app_bridge/wit/v5",
    world: "bridge",
    with: {},
});

use serde_json::json;
use standout::app::types::{
    ActionContext, ActionResponse, AppError, ErrorCode, TriggerContext, TriggerEvent,
    TriggerResponse,
};
use standout::app::http::RequestBuilder;

struct MyApp;

const CONNECTION_CONFIG: &str = r#"{
  "strategies": [
    {
      "id": "default",
      "type": "oauth2",
      "title": "OAuth 2.0",
      "authorization_server": {
        "issuer": "https://oauth.example.com",
        "authorization_endpoint": "https://oauth.example.com/authorize",
        "token_endpoint": "https://oauth.example.com/token",
        "response_types_supported": ["code"],
        "grant_types_supported": ["authorization_code", "refresh_token"]
      },
      "oauth_client": {
        "grant_type": "authorization_code",
        "scopes": ["read"],
        "client_id": "{{env:OAUTH_CLIENT_ID}}",
        "client_secret": "{{env:OAUTH_CLIENT_SECRET}}"
      },
      "storage": ["access_token", "refresh_token", "expires_at", "tenant"]
    }
  ],
  "default_strategy": "default",
  "connection_schema": {
    "type": "object",
    "properties": {
      "tenant": {
        "type": "string",
        "description": "Tenant subdomain chosen at connection"
      }
    },
    "required": ["tenant"]
  },
  "runtime": {
    "base_url": { "template": "https://{{tenant}}.api.example.com" },
    "headers": {
      "Authorization": { "template": "Bearer {{access_token}}" },
      "Content-Type": "application/json"
    }
  },
  "post_connect": {
    "request": {
      "method": "GET",
      "path": "/health"
    },
    "success_status": [200]
  }
}"#;

fn http_action(context: ActionContext) -> Result<ActionResponse, AppError> {
    let input: serde_json::Value = serde_json::from_str(&context.serialized_input).map_err(|e| {
        AppError {
            code: ErrorCode::Misconfigured,
            message: format!("Invalid input: {}", e),
        }
    })?;

    let url = input["url"].as_str().ok_or_else(|| AppError {
        code: ErrorCode::Misconfigured,
        message: "Missing url".to_string(),
    })?;

    let response = RequestBuilder::new()
        .method(standout::app::http::Method::Get)
        .url(url)
        .send()
        .map_err(|e| AppError {
            code: ErrorCode::Other,
            message: format!("HTTP error: {:?}", e),
        })?;

    Ok(ActionResponse {
        serialized_output: json!({
            "status": response.status,
            "body": response.body
        })
        .to_string(),
    })
}

fn simple_trigger(_context: TriggerContext) -> Result<TriggerResponse, AppError> {
    Ok(TriggerResponse {
        store: "{}".to_string(),
        events: vec![TriggerEvent {
            id: "event-1".to_string(),
            serialized_data: json!({ "message": "hello" }).to_string(),
        }],
    })
}

impl exports::standout::app::connections::Guest for MyApp {
    fn connection_config() -> String {
        CONNECTION_CONFIG.to_string()
    }
}

impl exports::standout::app::actions::Guest for MyApp {
    fn action_ids() -> Result<Vec<String>, AppError> {
        Ok(vec!["http-get".to_string()])
    }

    fn input_schema(_context: ActionContext) -> Result<String, AppError> {
        Ok(r#"{"type":"object","properties":{"url":{"type":"string"}}}"#.to_string())
    }

    fn output_schema(_context: ActionContext) -> Result<String, AppError> {
        Ok(r#"{"type":"object","properties":{"status":{"type":"integer"},"body":{"type":"string"}}}"#
            .to_string())
    }

    fn execute(context: ActionContext) -> Result<ActionResponse, AppError> {
        http_action(context)
    }
}

impl exports::standout::app::triggers::Guest for MyApp {
    fn trigger_ids() -> Result<Vec<String>, AppError> {
        Ok(vec!["simple-trigger".to_string()])
    }

    fn input_schema(_context: TriggerContext) -> Result<String, AppError> {
        Ok(r#"{"type":"object"}"#.to_string())
    }

    fn output_schema(_context: TriggerContext) -> Result<String, AppError> {
        Ok(r#"{"type":"object","properties":{"message":{"type":"string"}}}"#.to_string())
    }

    fn fetch_events(context: TriggerContext) -> Result<TriggerResponse, AppError> {
        simple_trigger(context)
    }
}

export!(MyApp);
