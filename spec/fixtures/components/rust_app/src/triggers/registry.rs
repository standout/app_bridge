use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::standout::app::types::{AppError, ErrorCode, TriggerContext, TriggerResponse};

pub type TriggerFn = fn(TriggerContext) -> Result<TriggerResponse, AppError>;

pub struct TriggerDefinition {
    pub func: TriggerFn,
    pub input_schema: String,
    pub output_schema: String,
}

pub static TRIGGER_REGISTRY: Lazy<Mutex<HashMap<&'static str, TriggerDefinition>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub fn register_trigger(name: &'static str, func: TriggerFn, input_schema: &str, output_schema: &str) {
    let _ = TRIGGER_REGISTRY.lock().unwrap().insert(name, TriggerDefinition {
        func,
        input_schema: input_schema.to_string(),
        output_schema: output_schema.to_string(),
    });
}

pub fn trigger_ids() -> Result<Vec<String>, AppError> {
    let registry = TRIGGER_REGISTRY.lock().unwrap();
    Ok(registry.keys().map(|&s| s.to_string()).collect())
}

pub fn input_schema(trigger_id: &str) -> Result<String, AppError> {
    let registry = TRIGGER_REGISTRY.lock().unwrap();
    if let Some(def) = registry.get(trigger_id) {
        Ok(def.input_schema.clone())
    } else {
        Err(AppError {
            code: ErrorCode::Other,
            message: format!("Trigger '{}' not found", trigger_id),
        })
    }
}

pub fn output_schema(trigger_id: &str) -> Result<String, AppError> {
    let registry = TRIGGER_REGISTRY.lock().unwrap();
    if let Some(def) = registry.get(trigger_id) {
        Ok(def.output_schema.clone())
    } else {
        Err(AppError {
            code: ErrorCode::Other,
            message: format!("Trigger '{}' not found", trigger_id),
        })
    }
}

pub fn call_trigger(ctx: TriggerContext) -> Result<TriggerResponse, AppError> {
  let registry = TRIGGER_REGISTRY.lock().unwrap();
  if let Some(def) = registry.get(ctx.trigger_id.as_str()) {
      (def.func)(ctx)
  } else {
    Err(AppError {
      code: ErrorCode::Other,
      message: format!("Trigger '{}' not found", ctx.trigger_id),
    })
  }
}
