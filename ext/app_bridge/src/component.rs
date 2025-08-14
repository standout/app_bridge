use std::result::Result::Ok;
use wasmtime::component::{bindgen, Component, Linker};
use wasmtime::{Engine, Result, Store};
use wasmtime_wasi::WasiCtxBuilder;

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
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;
    standout::app::http::add_to_linker(&mut linker, |s| s)?;

    Ok(linker)
}

pub fn build_store(engine: &Engine) -> Store<AppState> {
    let builder = WasiCtxBuilder::new().build();

    // ... configure `builder` more to add env vars, args, etc ...

    let store = Store::new(&engine, AppState::new(builder));

    store
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
        let mut store = build_store(&engine);
        let linker = build_linker(&engine).unwrap();
        let instant = app(file_path.to_string(), engine, &mut store, linker);

        instant.unwrap()
    }
}
