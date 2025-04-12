use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::standout::app::types::{AppError, ErrorCode, TriggerContext, TriggerResponse};

pub type TriggerFn = fn(TriggerContext) -> Result<TriggerResponse, AppError>;

pub static TRIGGER_REGISTRY: Lazy<Mutex<HashMap<&'static str, TriggerFn>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub fn register_trigger(name: &'static str, func: TriggerFn) {
    let _ = TRIGGER_REGISTRY.lock().unwrap().insert(name, func);
}

pub fn trigger_ids() -> Result<Vec<String>, AppError> {
    let registry = TRIGGER_REGISTRY.lock().unwrap();
    Ok(registry.keys().map(|&s| s.to_string()).collect())
}

pub fn call_trigger(ctx: TriggerContext) -> Result<TriggerResponse, AppError> {
  let registry = TRIGGER_REGISTRY.lock().unwrap();
  if let Some(trigger_fn) = registry.get(ctx.trigger_id.as_str()) {
      trigger_fn(ctx)
  } else {
    Err(AppError {
      code: ErrorCode::Other,
      message: format!("Trigger '{}' not found", ctx.trigger_id),
    })
  }
}
