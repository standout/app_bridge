//! Canonical types used internally by the bridge.
//!
//! These are version-agnostic and converted to/from versioned WIT types at the boundary.

use std::fmt;

// ============================================================================
// Canonical Types - used throughout the codebase, not tied to any version
// ============================================================================

#[derive(Debug, Clone)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for AppError {}

#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    Unauthenticated,
    Forbidden,
    Misconfigured,
    Unsupported,
    RateLimit,
    Timeout,
    Unavailable,
    InternalError,
    MalformedResponse,
    Other,
    CompleteWorkflow,
    CompleteParent,
}

#[derive(Debug, Clone)]
pub struct Connection {
    pub id: String,
    pub name: String,
    pub serialized_data: String,
}

#[derive(Debug, Clone)]
pub struct TriggerContext {
    pub trigger_id: String,
    pub connection: Connection,
    pub store: String,
    pub serialized_input: String,
}

#[derive(Debug, Clone)]
pub struct ActionContext {
    pub action_id: String,
    pub connection: Connection,
    pub serialized_input: String,
}

#[derive(Debug, Clone)]
pub struct TriggerEvent {
    pub id: String,
    pub serialized_data: String,
}

#[derive(Debug, Clone)]
pub struct TriggerResponse {
    pub store: String,
    pub events: Vec<TriggerEvent>,
}

#[derive(Debug, Clone)]
pub struct ActionResponse {
    pub serialized_output: String,
}
