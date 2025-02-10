use anyhow::Ok;
use wasmtime::component::{bindgen, Component, Linker, ResourceTable};
use wasmtime::{Engine, Result, Store};
use wasmtime_wasi::{WasiCtx, WasiView, WasiCtxBuilder};

bindgen!();

pub fn build_engine() -> Engine {
  Engine::default()
}



pub fn build_linker(engine: &Engine) -> Result<Linker<MyState>> {
  let mut linker = Linker::<MyState>::new(&engine);
  wasmtime_wasi::add_to_linker_sync(&mut linker)?;
  // ... add any further functionality to `linker` if desired ...
  Ok(linker)
}

pub fn build_store(engine: &Engine) -> Store<MyState> {
  let mut builder = WasiCtxBuilder::new();

  // ... configure `builder` more to add env vars, args, etc ...

  // TODO: This context should be global for all wasm components.
  let store = Store::new(
      &engine,
      MyState {
          ctx: builder.build(),
          table: ResourceTable::new(),
      },
  );

  store
}

pub fn app(file_path: String, engine: Engine, store: &mut Store<MyState>, linker: Linker<MyState>) -> Result<Bridge> {
  // Load the application component from the file system.
  let component = Component::from_file(&engine, file_path)?;
  let instance = Bridge::instantiate(store, &component, &linker)?;

  Ok(instance)
}

pub fn load_from_file(file_path: String) -> Result<(Bridge, Store<MyState>)> {
  let engine = Engine::default();

  let mut linker = Linker::<MyState>::new(&engine);
  wasmtime_wasi::add_to_linker_sync(&mut linker)?;
  // ... add any further functionality to `linker` if desired ...

  let mut builder = WasiCtxBuilder::new();

  // ... configure `builder` more to add env vars, args, etc ...

  // TODO: This context should be global for all wasm components.
  let mut store = Store::new(
      &engine,
      MyState {
          ctx: builder.build(),
          table: ResourceTable::new(),
      },
  );

  // ... use `linker` to instantiate within `store` ...


  // Load the application component from the file system.
  let component = Component::from_file(&engine, file_path)?;
  let instance = Bridge::instantiate(&mut store, &component, &linker)?;


  Ok((instance, store))
}

pub struct MyState {
  ctx: WasiCtx,
  table: ResourceTable,
}

impl WasiView for MyState {
  fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
  fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}

impl Default for MyState {
  fn default() -> Self {
      Self {
          ctx: WasiCtxBuilder::new().build(),
          table: ResourceTable::new(),
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
