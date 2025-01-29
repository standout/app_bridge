use std::vec;

use magnus::{function, method, prelude::*, Error, Ruby};
// use wasmtime::*;
use wasmtime::component::{bindgen, Component, Linker, ResourceTable};
// use anyhow::Result;
use wasmtime_wasi::bindings::exports::wasi;
// use wasmtime_wasi::WasiCtx;
// use wasmtime_wasi::WasiCtxBuilder;


bindgen!();



struct ThirdPartyApp {
    triggers: Vec<String>,
}




// impl BridgeImports for ThirdPartyApp {
    // TODO: Add functions that the Web Assembly component can call. The one
    // that is marked as imports in the wit file.
    //
    // fn get_triggers(&self) -> Vec<String> {
    //     self.triggers.clone()
    // }
// }



use wasmtime::{Engine, Result, Store, Config};
use wasmtime_wasi::{WasiCtx, WasiView, WasiCtxBuilder};

fn wasm_component(component_path: String) -> Result<(Bridge, Store<MyState>)> {
    let engine = Engine::default();

    let mut linker = Linker::<MyState>::new(&engine);
    wasmtime_wasi::add_to_linker_sync(&mut linker)?;
    // ... add any further functionality to `linker` if desired ...

    let mut builder = WasiCtxBuilder::new();

    // ... configure `builder` more to add env vars, args, etc ...

    let mut store = Store::new(
        &engine,
        MyState {
            ctx: builder.build(),
            table: ResourceTable::new(),
        },
    );

    // ... use `linker` to instantiate within `store` ...


    // Load the application component from the file system.
    let component = Component::from_file(&engine, component_path)?;
    let instance = Bridge::instantiate(&mut store, &component, &linker)?;


    Ok((instance, store))
}

struct MyState {
    ctx: WasiCtx,
    table: ResourceTable,
}

impl WasiView for MyState {
    fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
    fn table(&mut self) -> &mut ResourceTable { &mut self.table }
}

fn hello(subject: String) -> String {
    format!("Hello from Rust, {subject}!")
}

fn load_wasm_component(component_path: String) -> wasmtime::Result<(Bridge, wasmtime::Store<WasiCtx>)> {
    let engine = Engine::default();
    let mut store = Store::new(&engine, WasiCtxBuilder::new().inherit_stdio().inherit_env().build());



    // Skapa WASI och bind den till en komponent
    // let wasi = wasmtime_wasi::Wasi::new(&mut store, store.data().clone());
    // let mut linker = Linker::new(&engine);
    // wasi.add_to_linker(&mut linker)?;

    // Instantiation of bindings always happens through a `Linker`.
    // Configuration of the linker is done through a generated `add_to_linker`
    // method on the bindings structure.
    //
    // Note that the closure provided here is a projection from `T` in
    // `Store<T>` to `&mut U` where `U` implements the `HelloWorldImports`
    // trait. In this case the `T`, `ThirdPartyApp`, is stored directly in the
    // structure so no projection is necessary here.
    // let mut linker = Linker::new(&engine);
    // Bridge::add_to_linker(&mut linker, |state: &mut ThirdPartyApp| state)?;

    // let wasi_ctx = wasmtime_wasi::WasiCtxBuilder::new()
    //     .inherit_stdio()
    //     .inherit_env()
    //     .build();

    let wasi_ctx = WasiCtxBuilder::new()
        .inherit_stdio() // Använder terminalens stdin, stdout, stderr
        .inherit_env()   // Exponerar alla miljövariabler till WASM
        .build();

    let mut store = Store::new(&engine, wasi_ctx);


    let component = Component::from_file(&engine, component_path)?;

    let mut linker = Linker::new(&engine);

    // wasmtime_wasi::bindings::cli::environment::add_to_linker_get_host(linker, host_getter)
    // wasmtime_wasi::bindings::cli::environment::add_to_linker(&mut linker, |ctx| ctx)?;


    // As with the core wasm API of Wasmtime instantiation occurs within a
    // `Store`. The bindings structure contains an `instantiate` method which
    // takes the store, component, and linker. This returns the `bindings`
    // structure which is an instance of `Bridge` and supports typed access
    // to the exports of the component.
    // let mut store = Store::new(
    //     &engine,
    //     ThirdPartyApp {
    //         triggers: vec!["trigger1".to_string(), "trigger2".to_string()],
    //     },
    // );

    // let mut store = Store::new(
    //     &engine,
    //     wasi_ctx
    // );
    let instance = Bridge::instantiate(&mut store, &component, &linker)?;

    // let instance = linker.instantiate(&mut store, &component)?;

    // Here our `greet` function doesn't take any parameters for the component,
    // but in the Wasmtime embedding API the first argument is always a `Store`.
    // bindings.call_greet(&mut store)?;

    Ok((instance, store))
}

#[magnus::wrap(class = "AppBridge::App")]
pub struct RApp {
    instance: Bridge,
    store: Store<MyState>,
}

impl RApp {
    fn new(component_path: String) -> Self {
        // let (instance, store) = load_wasm_component(component_path).unwrap();

        let (instance, store) = wasm_component(component_path).unwrap();

        Self { instance, store }
    }

    fn get_triggers(&mut self, _ruby: &Ruby) -> Vec<String> {
        // let get_triggers = self.instance.get_typed_func::<(), Vec<String>>(&mut self.store, "get_triggers").unwrap();
        // get_triggers.call(&mut self.store, ()).unwrap()
        self.instance.standout_app_triggers().call_get_triggers(&mut self.store).unwrap()

        // vec!["trigger1".to_string(), "trigger2".to_string()]
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("AppBridge")?;
    module.define_singleton_method("hello", function!(hello, 1))?;

    let app_class = module.define_class("App", ruby.class_object())?;
    app_class.define_singleton_method("new", function!(RApp::new, 1))?;
    app_class.define_method("get_triggers", method!(RApp::get_triggers, 1))?;

    Ok(())
}
