use std::result::Result::Ok;
use wasmtime::component::{bindgen, Component, Linker};
use wasmtime::{Engine, Result, Store};
use wasmtime_wasi::WasiCtxBuilder;

bindgen!();

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
    let instance = Bridge::instantiate(store, &component, &linker)?;

    Ok(instance)
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
