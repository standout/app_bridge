use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::standout::app::types::{AppError, ErrorCode, ActionContext, ActionResponse};

pub type ActionFn = fn(ActionContext) -> Result<ActionResponse, AppError>;

pub static ACTION_REGISTRY: Lazy<Mutex<HashMap<&'static str, ActionFn>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub fn register_action(name: &'static str, func: ActionFn) {
    let _ = ACTION_REGISTRY.lock().unwrap().insert(name, func);
}

pub fn action_ids() -> Result<Vec<String>, AppError> {
    let registry = ACTION_REGISTRY.lock().unwrap();
    Ok(registry.keys().map(|&s| s.to_string()).collect())
}

pub fn call_action(ctx: ActionContext) -> Result<ActionResponse, AppError> {
  let registry = ACTION_REGISTRY.lock().unwrap();
  if let Some(action_fn) = registry.get(ctx.action_id.as_str()) {
      action_fn(ctx)
  } else {
    Err(AppError {
      code: ErrorCode::Other,
      message: format!("Action '{}' not found", ctx.action_id),
    })
  }
}

