use crate::component::standout;
use crate::component::standout::app::http::Request;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wasmtime::component::ResourceTable;
use wasmtime_wasi::p2::{WasiCtx, WasiCtxBuilder, WasiView, IoView};

pub struct AppState {
    ctx: WasiCtx,
    table: ResourceTable,
    pub client: Arc<Mutex<Client>>,
    pub request_list: HashMap<u32, Request>,
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
            next_request_id: 0,
            environment_variables: env_vars.unwrap_or_default(),
        }
    }
}

impl standout::app::http::Host for AppState {
    // Impl http host methods here
}

impl standout::app::environment::Host for AppState {
    fn env_vars(&mut self) -> Vec<(String, String)> {
        self.environment_variables.clone().into_iter().collect()
    }

    fn env_var(&mut self, name: String) -> Option<String> {
        self.environment_variables.get(&name).cloned()
    }
}

impl IoView for AppState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl WasiView for AppState {
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(WasiCtxBuilder::new().build(), None)
    }
}
