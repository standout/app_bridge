use std::collections::HashMap;
use std::result::Result::Ok;
use wasmtime::component::{Component, Linker};
use wasmtime::{Engine, Result, Store};
use wasmtime_wasi::p2::WasiCtxBuilder;

use crate::app_state::AppState;
use crate::types::{
    ActionContext, ActionResponse, AppError, Connection, ErrorCode, TriggerContext,
    TriggerEvent, TriggerResponse,
};

// ============================================================================
// WIT version modules
//
// To add a new version:
// 1. Add a new pub mod vN { bindgen!(...) }
// 2. Add impl_conversions!(vN) below
// 3. Add variant to BridgeWrapper enum
// 4. Add to build_linker() and app() functions
// 5. Add arm to each bridge_method! in BridgeWrapper impl
// ============================================================================

pub mod v3 {
    wasmtime::component::bindgen!({
        path: "./wit/v3",
        world: "bridge",
    });
}

pub mod v4 {
    wasmtime::component::bindgen!({
        path: "./wit/v4",
        world: "bridge",
    });
}

// ============================================================================
// Version conversion macro - generates From impls for a version module
// ============================================================================

macro_rules! impl_conversions {
    ($v:ident) => {
        // ErrorCode: version → canonical
        impl From<$v::standout::app::types::ErrorCode> for ErrorCode {
            fn from(c: $v::standout::app::types::ErrorCode) -> Self {
                use $v::standout::app::types::ErrorCode as V;
                match c {
                    V::Unauthenticated => Self::Unauthenticated,
                    V::Forbidden => Self::Forbidden,
                    V::Misconfigured => Self::Misconfigured,
                    V::Unsupported => Self::Unsupported,
                    V::RateLimit => Self::RateLimit,
                    V::Timeout => Self::Timeout,
                    V::Unavailable => Self::Unavailable,
                    V::InternalError => Self::InternalError,
                    V::MalformedResponse => Self::MalformedResponse,
                    V::Other => Self::Other,
                    V::CompleteWorkflow => Self::CompleteWorkflow,
                    V::CompleteParent => Self::CompleteParent,
                }
            }
        }

        // AppError: version → canonical
        impl From<$v::standout::app::types::AppError> for AppError {
            fn from(e: $v::standout::app::types::AppError) -> Self {
                Self { code: e.code.into(), message: e.message }
            }
        }

        // TriggerEvent: version → canonical
        impl From<$v::standout::app::types::TriggerEvent> for TriggerEvent {
            fn from(e: $v::standout::app::types::TriggerEvent) -> Self {
                Self { id: e.id, serialized_data: e.serialized_data }
            }
        }

        // TriggerResponse: version → canonical
        impl From<$v::standout::app::types::TriggerResponse> for TriggerResponse {
            fn from(r: $v::standout::app::types::TriggerResponse) -> Self {
                Self {
                    store: r.store,
                    events: r.events.into_iter().map(Into::into).collect(),
                }
            }
        }

        // ActionResponse: version → canonical
        impl From<$v::standout::app::types::ActionResponse> for ActionResponse {
            fn from(r: $v::standout::app::types::ActionResponse) -> Self {
                Self { serialized_output: r.serialized_output }
            }
        }

        // Connection: canonical → version (for passing to components)
        impl From<&Connection> for $v::standout::app::types::Connection {
            fn from(c: &Connection) -> Self {
                Self {
                    id: c.id.clone(),
                    name: c.name.clone(),
                    serialized_data: c.serialized_data.clone(),
                }
            }
        }

        // TriggerContext: canonical → version
        impl From<&TriggerContext> for $v::standout::app::types::TriggerContext {
            fn from(c: &TriggerContext) -> Self {
                Self {
                    trigger_id: c.trigger_id.clone(),
                    connection: (&c.connection).into(),
                    store: c.store.clone(),
                    serialized_input: c.serialized_input.clone(),
                }
            }
        }

        // ActionContext: canonical → version
        impl From<&ActionContext> for $v::standout::app::types::ActionContext {
            fn from(c: &ActionContext) -> Self {
                Self {
                    action_id: c.action_id.clone(),
                    connection: (&c.connection).into(),
                    serialized_input: c.serialized_input.clone(),
                }
            }
        }
    };
}

// Generate conversions for all supported versions
impl_conversions!(v3);
impl_conversions!(v4);

// ============================================================================
// BridgeWrapper - unified interface for all component versions
//
// To add vN: add variant BridgeWrapper::VN(vN::Bridge)
// ============================================================================

pub enum BridgeWrapper {
    V3(v3::Bridge),
    V4(v4::Bridge),
}

/// Macro to implement a bridge method that works across all versions.
/// Each version's result is converted to canonical types.
macro_rules! bridge_method {
    // Simple no-arg method (e.g., trigger_ids, action_ids)
    (fn $name:ident() -> Result<$ok_type:ty> via $interface:ident . $method:ident) => {
        pub fn $name(&self, store: &mut Store<AppState>) -> Result<std::result::Result<$ok_type, AppError>> {
            match self {
                BridgeWrapper::V3(b) => {
                    let r = b.$interface().$method(store)?;
                    Ok(r.map_err(Into::into))
                }
                BridgeWrapper::V4(b) => {
                    let r = b.$interface().$method(store)?;
                    Ok(r.map_err(Into::into))
                }
            }
        }
    };
    // Method with TriggerContext
    (fn $name:ident(&TriggerContext) -> Result<$ok_type:ty> via $interface:ident . $method:ident) => {
        pub fn $name(&self, store: &mut Store<AppState>, ctx: &TriggerContext) -> Result<std::result::Result<$ok_type, AppError>> {
            match self {
                BridgeWrapper::V3(b) => {
                    let r = b.$interface().$method(store, &ctx.into())?;
                    Ok(r.map(Into::into).map_err(Into::into))
                }
                BridgeWrapper::V4(b) => {
                    let r = b.$interface().$method(store, &ctx.into())?;
                    Ok(r.map(Into::into).map_err(Into::into))
                }
            }
        }
    };
    // Method with ActionContext
    (fn $name:ident(&ActionContext) -> Result<$ok_type:ty> via $interface:ident . $method:ident) => {
        pub fn $name(&self, store: &mut Store<AppState>, ctx: &ActionContext) -> Result<std::result::Result<$ok_type, AppError>> {
            match self {
                BridgeWrapper::V3(b) => {
                    let r = b.$interface().$method(store, &ctx.into())?;
                    Ok(r.map(Into::into).map_err(Into::into))
                }
                BridgeWrapper::V4(b) => {
                    let r = b.$interface().$method(store, &ctx.into())?;
                    Ok(r.map(Into::into).map_err(Into::into))
                }
            }
        }
    };
}

impl BridgeWrapper {
    // Trigger methods
    bridge_method!(fn call_trigger_ids() -> Result<Vec<String>> via standout_app_triggers . call_trigger_ids);
    bridge_method!(fn call_trigger_input_schema(&TriggerContext) -> Result<String> via standout_app_triggers . call_input_schema);
    bridge_method!(fn call_trigger_output_schema(&TriggerContext) -> Result<String> via standout_app_triggers . call_output_schema);
    bridge_method!(fn call_fetch_events(&TriggerContext) -> Result<TriggerResponse> via standout_app_triggers . call_fetch_events);

    // Action methods
    bridge_method!(fn call_action_ids() -> Result<Vec<String>> via standout_app_actions . call_action_ids);
    bridge_method!(fn call_action_input_schema(&ActionContext) -> Result<String> via standout_app_actions . call_input_schema);
    bridge_method!(fn call_action_output_schema(&ActionContext) -> Result<String> via standout_app_actions . call_output_schema);
    bridge_method!(fn call_execute(&ActionContext) -> Result<ActionResponse> via standout_app_actions . call_execute);
}

// ============================================================================
// Builder functions
// ============================================================================

pub fn build_engine() -> Engine {
    Engine::default()
}

pub fn build_linker(engine: &Engine) -> Result<Linker<AppState>> {
    let mut linker = Linker::<AppState>::new(engine);

    // WASI support (shared by all versions)
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;

    // ---- Version-specific interfaces ----
    // v3: http + environment
    v3::standout::app::http::add_to_linker(&mut linker, |s| s)?;
    v3::standout::app::environment::add_to_linker(&mut linker, |s| s)?;

    // v4: http + environment + file
    v4::standout::app::http::add_to_linker(&mut linker, |s| s)?;
    v4::standout::app::environment::add_to_linker(&mut linker, |s| s)?;
    v4::standout::app::file::add_to_linker(&mut linker, |s| s)?;

    // Add new versions here:
    // v5::standout::app::http::add_to_linker(&mut linker, |s| s)?;
    // v5::standout::app::environment::add_to_linker(&mut linker, |s| s)?;
    // v5::standout::app::file::add_to_linker(&mut linker, |s| s)?;
    // v5::standout::app::new_feature::add_to_linker(&mut linker, |s| s)?;

    Ok(linker)
}

pub fn build_store(engine: &Engine, env_vars: Option<HashMap<String, String>>) -> Store<AppState> {
    let mut builder = WasiCtxBuilder::new();

    if let Some(env_vars) = &env_vars {
        for (key, value) in env_vars {
            builder.env(key, value);
        }
    }

    Store::new(engine, AppState::new(builder.build(), env_vars))
}

/// Try to instantiate a WASM component.
/// Attempts versions from newest to oldest until one succeeds.
pub fn app(
    file_path: String,
    engine: Engine,
    store: &mut Store<AppState>,
    linker: Linker<AppState>,
) -> Result<BridgeWrapper> {
    let component = Component::from_file(&engine, &file_path)?;

    // Try versions newest-first. When adding vN, insert at the top.
    // v4 (current - has file interface)
    if let Ok(instance) = v4::Bridge::instantiate(&mut *store, &component, &linker) {
        return Ok(BridgeWrapper::V4(instance));
    }

    // v3 (legacy - no file interface)
    if let Ok(instance) = v3::Bridge::instantiate(&mut *store, &component, &linker) {
        return Ok(BridgeWrapper::V3(instance));
    }

    Err(wasmtime::Error::msg(
        "Failed to instantiate component: no compatible WIT version found (tried v4, v3)",
    ))
}
