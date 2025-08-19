use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::standout::app::types::{AppError, ErrorCode, ActionContext, ActionResponse};

pub type ActionFn = fn(ActionContext) -> Result<ActionResponse, AppError>;

pub struct ActionDefinition {
    pub func: ActionFn,
    pub input_schema: String,
    pub output_schema: String,
}

pub static ACTION_REGISTRY: Lazy<Mutex<HashMap<&'static str, ActionDefinition>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub fn register_action(name: &'static str, func: ActionFn, input_schema: &str, output_schema: &str) {
    let _ = ACTION_REGISTRY.lock().unwrap().insert(name, ActionDefinition {
        func,
        input_schema: input_schema.to_string(),
        output_schema: output_schema.to_string(),
    });
}

pub fn action_ids() -> Result<Vec<String>, AppError> {
    let registry = ACTION_REGISTRY.lock().unwrap();
    Ok(registry.keys().map(|&s| s.to_string()).collect())
}

pub fn input_schema(action_id: &str) -> Result<String, AppError> {
    let registry = ACTION_REGISTRY.lock().unwrap();
    if let Some(action_def) = registry.get(action_id) {
        Ok(action_def.input_schema.clone())
    } else {
        Err(AppError {
            code: ErrorCode::Other,
            message: format!("Action '{}' not found", action_id),
        })
    }
}

pub fn output_schema(action_id: &str) -> Result<String, AppError> {
    let registry = ACTION_REGISTRY.lock().unwrap();
    if let Some(action_def) = registry.get(action_id) {
        Ok(action_def.output_schema.clone())
    } else {
        Err(AppError {
            code: ErrorCode::Other,
            message: format!("Action '{}' not found", action_id),
        })
    }
}

pub fn call_action(ctx: ActionContext) -> Result<ActionResponse, AppError> {
  let registry = ACTION_REGISTRY.lock().unwrap();
  if let Some(action_def) = registry.get(ctx.action_id.as_str()) {
      (action_def.func)(ctx)
  } else {
    Err(AppError {
      code: ErrorCode::Other,
      message: format!("Action '{}' not found", ctx.action_id),
    })
  }
}

