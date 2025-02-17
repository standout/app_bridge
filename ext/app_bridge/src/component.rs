use reqwest::Method;
// use anyhow::Ok;
use wasmtime::component::{bindgen, Component, Linker, ResourceTable};
use wasmtime::{Engine, Result, Store};
use wasmtime_wasi::{WasiCtx, WasiView, WasiCtxBuilder};

use reqwest::blocking::Client;
use std::sync::{Arc, Mutex};
use std::result::Result::Ok;

bindgen!();

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
  let builder = WasiCtxBuilder::new()
    .build();

  // ... configure `builder` more to add env vars, args, etc ...

  let store = Store::new(
      &engine,
      AppState {
          ctx: builder,
          table: ResourceTable::new(),
          client: Arc::new(Mutex::new(Client::new())),
      },
  );

  store
}

pub fn app(file_path: String, engine: Engine, store: &mut Store<AppState>, linker: Linker<AppState>) -> Result<Bridge> {
  // Load the application component from the file system.
  let component = Component::from_file(&engine, file_path)?;
  let instance = Bridge::instantiate(store, &component, &linker)?;

  Ok(instance)
}

pub struct AppState {
  ctx: WasiCtx,
  table: ResourceTable,
  client: Arc<Mutex<Client>>,
}

impl standout::app::http::Method {
  fn as_reqewst_method(&self) -> Method {
    match self {
        standout::app::http::Method::Get => Method::GET,
        standout::app::http::Method::Post => Method::POST,
        standout::app::http::Method::Put => Method::PUT,
        standout::app::http::Method::Delete => Method::DELETE,
        standout::app::http::Method::Patch => Method::PATCH,
        standout::app::http::Method::Head => Method::HEAD,
        standout::app::http::Method::Options => Method::OPTIONS,
    }
  }
}

impl standout::app::http::Host for AppState {
  fn request(
    &mut self,
    method: standout::app::http::Method,
    url: String,
    headers: standout::app::http::OptionalHeaders
  ) -> std::result::Result<std::string::String, std::string::String> {
    let client = self.client.lock().unwrap();
    let mut request_builder = client.request(method.as_reqewst_method(), &url);

    if let standout::app::http::OptionalHeaders::Some(headers) = headers {
        for (key, value) in headers {
            request_builder = request_builder.header(key, value);
        }
    }

    match request_builder.send() {
        Ok(resp) => match resp.text() {
            Ok(body) => Ok(body),
            Err(_) => Err("Failed to read response body".to_string()),
        },
        Err(_) => Err("Failed to fetch URL".to_string()),
    }
  }
}

impl WasiView for AppState {
  fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
  fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}

impl Default for AppState {
  fn default() -> Self {
      Self {
          ctx: WasiCtxBuilder::new().build(),
          table: ResourceTable::new(),
          client: Arc::new(Mutex::new(Client::new())),
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
