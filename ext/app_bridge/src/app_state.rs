use crate::component::{v3, v4, v4_1};
use crate::component::v4::standout::app::http::Request;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wasmtime::component::ResourceTable;
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};
use wasmtime_wasi_io::IoView;

pub struct AppState {
    ctx: WasiCtx,
    table: ResourceTable,
    pub client: Arc<Mutex<Client>>,
    pub request_list: HashMap<u32, Request>,
    pub request_body_bytes: HashMap<u32, Vec<u8>>,
    pub next_request_id: u32,
    pub environment_variables: HashMap<String, String>,
}

impl AppState {
    pub fn new(ctx: WasiCtx, env_vars: Option<HashMap<String, String>>) -> Self {
        Self {
            ctx,
            table: ResourceTable::new(),
            client: Arc::new(Mutex::new(Client::new())),
            request_list: HashMap::new(),
            request_body_bytes: HashMap::new(),
            next_request_id: 0,
            environment_variables: env_vars.unwrap_or_default(),
        }
    }
}

// ============================================================================
// Macro to implement identical Host traits for multiple WIT versions
// ============================================================================

/// Implements http::Host and environment::Host for a given WIT version module.
/// These implementations are identical across versions.
macro_rules! impl_host_for_version {
    ($version:ident) => {
        impl $version::standout::app::http::Host for AppState {}

        impl $version::standout::app::environment::Host for AppState {
            fn env_vars(&mut self) -> Vec<(String, String)> {
                self.environment_variables.clone().into_iter().collect()
            }

            fn env_var(&mut self, name: String) -> Option<String> {
                self.environment_variables.get(&name).cloned()
            }
        }
    };
}

// Apply to both versions
impl_host_for_version!(v3);
impl_host_for_version!(v4);
impl_host_for_version!(v4_1);

// ============================================================================
// WASI implementations
// ============================================================================

impl WasiView for AppState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}

impl IoView for AppState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(WasiCtxBuilder::new().build(), None)
    }
}
