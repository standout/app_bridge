use std::cell::RefCell;
use component::exports::standout::app;
use magnus::{function, method, prelude::*, Error, Ruby};
use wasmtime::AsContextMut;
mod component;

// START GLOBALS
// use wasmtime::{Engine, Store};
// use wasmtime::component::Linker;
// static mut ENGINE: Option<Engine> = None;
// static mut LINKER: Option<Linker<component::MyState>> = None;
// static mut STORE: Option<Store<component::MyState>> = None;
// END GLOBALS

// #[magnus::wrap(class = "AppBridge::App")]
#[derive(Default)]
pub struct RApp {
    component_path: String,
    instance: component::Bridge,
    store: wasmtime::Store<component::MyState>,
}

// impl RApp {
//     fn new(component_path: String) -> Self {
//         // let engine = ENGINE.get_or_insert(component::build_engine());
//         // let linker = LINKER.get_or_insert(component::build_linker(engine));
//         // let store = STORE.get_or_insert(component::build_store(engine));
//         // let (instance, store) = component::load_from_file(component_path).unwrap();
//         // Self { instance, store }
//         unsafe {
//             Self {
//                 instance: component::app(component_path, ENGINE.unwrap(),STORE.unwrap(), LINKER.unwrap()).unwrap(),
//                 // store: component::load_from_file(component_path).unwrap().1,
//             }
//         }
//     }
//     fn rb_get_triggers(&self) -> Vec<String> {
//         // rb_self.instance.standout_app_triggers().call_get_triggers(rb_self.store).unwrap()
//         vec!["trigger1".to_string(), "trigger2".to_string()]
//     }
// }

#[derive(Default)]
#[magnus::wrap(class = "AppBridge::App")]
struct MutRApp(RefCell<RApp>);

impl MutRApp {
    fn initialize(&self, component_path: String) {
        let mut this = self.0.borrow_mut();
        let engine = component::build_engine();
        let linker = component::build_linker(&engine).unwrap();
        let mut store = component::build_store(&engine);

        let app = component::app(component_path.clone(), engine, &mut store, linker).unwrap();

        this.component_path = component_path.to_string();
        this.instance = app;
        this.store = store;
    }

    fn get_triggers(&self) -> Vec<String> {
        // vec!["trigger1".to_string(), "trigger2".to_string()]
        // let this = self.0.borrow();
        // let instance = this.instance.as_ref().unwrap();
        // .call_get_triggers(&this.store).unwrap()

        // let mut this = self.0.borrow_mut();
        // let instance = &this.instance;
        // let store = self.0.;
        // instance.standout_app_triggers().call_get_triggers(store).unwrap()

        let component_path = self.0.borrow().component_path.clone();
        let engine = component::build_engine();
        let linker = component::build_linker(&engine).unwrap();
        let mut store = component::build_store(&engine);

        let app = component::app(component_path, engine, &mut store, linker).unwrap();

        app.standout_app_triggers().call_get_triggers(store).unwrap()
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("AppBridge")?;
    // module.define_singleton_method("get_triggers", function!(RApp::rb_get_triggers, 0))?;

    let app_class = module.define_class("App", ruby.class_object())?;
    app_class.define_alloc_func::<MutRApp>();
    app_class.define_method("initialize", method!(MutRApp::initialize, 1))?;
    // app_class.define_singleton_method("new", function!(RApp::new, 1))?;
    app_class.define_method("triggers", method!(MutRApp::get_triggers, 0))?;

    Ok(())
}
