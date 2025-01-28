use magnus::{function, method, prelude::*, Error, Ruby};
use wasmtime::*;
use wasmtime::component::{Component, Linker};
use anyhow::Result;

fn hello(subject: String) -> String {
    format!("Hello from Rust, {subject}!")
}

fn load_wasm_component(component_path: String) -> wasmtime::Result<(wasmtime::component::Instance, wasmtime::Store<()>)> {
    // Instantiate the engine and store
    let engine = wasmtime::Engine::default();
    let mut store = wasmtime::Store::new(&engine, ());

    // Load the component from disk
    let bytes = std::fs::read(component_path)?;
    let component = wasmtime::component::Component::new(&engine, bytes)?;

    // Configure the linker
    let mut linker = wasmtime::component::Linker::new(&engine);
    // The component expects one import `name` that
    // takes no params and returns a string
    // linker
    //     .root()
    //     .func_wrap("name", |_store, _params: ()| {
    //         Ok((String::from("Alice"),))
    //     })?;

    // Instantiate the component
    let instance = linker.instantiate(&mut store, &component)?;

    // Call the `greet` function
    // let func = instance.get_func(&mut store, "greet").expect("greet export not found");
    // let mut result = [wasmtime::component::Val::String("".into())];
    // func.call(&mut store, &[], &mut result)?;

    // This should print out `Greeting: [String("Hello, Alice!")]`
    // println!("Greeting: {:?}", result);

    Ok((instance, store))
}

#[magnus::wrap(class = "AppBridge::App")]
pub struct RApp {
    instance: wasmtime::component::Instance,
    store: Store<()>,
}

impl RApp {
    fn new(component_path: String) -> Self {
        let (instance, store) = load_wasm_component(component_path).unwrap();

        Self { instance, store }
    }

    fn get_triggers(&self) -> Vec<String> {
        // let get_triggers = self.instance.get_typed_func::<(), Vec<String>>(&mut self.store, "get_triggers").unwrap();
        // get_triggers.call(&mut self.store, ()).unwrap()

        vec!["trigger1".to_string(), "trigger2".to_string()]
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("AppBridge")?;
    module.define_singleton_method("hello", function!(hello, 1))?;

    let app_class = module.define_class("App", ruby.class_object())?;
    app_class.define_singleton_method("new", function!(RApp::new, 1))?;
    app_class.define_method("get_triggers", method!(RApp::get_triggers, 0))?;

    Ok(())
}
