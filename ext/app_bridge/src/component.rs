use std::result::Result::Ok;
use std::collections::HashMap;
use wasmtime::component::{bindgen, Component, Linker};
use wasmtime::{Engine, Result, Store};
use wasmtime_wasi::p2::WasiCtxBuilder;

bindgen!({
    path: "./wit",
    world: "bridge",
});

use crate::app_state::AppState;

pub fn build_engine() -> Engine {
    Engine::default()
}

pub fn build_linker(engine: &Engine) -> Result<Linker<AppState>> {
    let mut linker = Linker::<AppState>::new(&engine);
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
    standout::app::http::add_to_linker(&mut linker, |s| s)?;
    standout::app::environment::add_to_linker(&mut linker, |s| s)?;

    Ok(linker)
}

pub fn build_store(engine: &Engine, env_vars: Option<HashMap<String, String>>) -> Store<AppState> {
    let mut builder = WasiCtxBuilder::new();

    // Add environment variables to WASI context if provided
    if let Some(env_vars) = &env_vars {
        for (key, value) in env_vars {
            builder.env(key, value);
        }
    }

    let ctx = builder.build();

    // Create AppState with or without environment variables
    let app_state = AppState::new(ctx, env_vars);

    Store::new(&engine, app_state)
}

pub fn app(
    file_path: String,
    engine: Engine,
    store: &mut Store<AppState>,
    linker: Linker<AppState>,
) -> Result<Bridge> {
    // Load the application component from the file system.
    let component = Component::from_file(&engine, file_path)?;

    // Try to instantiate the component - if it fails due to missing interface,
    // we'll catch that error and return a specific message
    match Bridge::instantiate(store, &component, &linker) {
        Ok(instance) => Ok(instance),
        Err(e) => {
            if e.to_string().contains("no exported instance") {
                Err(wasmtime::Error::msg(
                    "Incompatible WASM file version"
                ))
            } else {
                Err(e)
            }
        }
    }
}

impl Default for Bridge {
    fn default() -> Self {
        let file_path = "spec/fixtures/components/example_app.wasm";
        let engine = Engine::default();
        let mut store = build_store(&engine, None);
        let linker = build_linker(&engine).unwrap();
        let instant = app(file_path.to_string(), engine, &mut store, linker);

        instant.unwrap()
    }
}
