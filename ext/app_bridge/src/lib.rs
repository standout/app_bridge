use std::cell::RefCell;
use magnus::{method, prelude::*, Error, Ruby};
use wasmtime::Store;
mod component;
use component::{AppState, Bridge, build_linker, build_store, build_engine, app};

#[derive(Default)]
pub struct RApp {
    component_path: String,
    instance: RefCell<Option<Bridge>>,
    store: RefCell<Option<Store<AppState>>>,
}

#[derive(Default)]
#[magnus::wrap(class = "AppBridge::App")]
struct MutRApp(RefCell<RApp>);

impl MutRApp {
    fn initialize(&self, component_path: String) {
        let mut this = self.0.borrow_mut();
        let engine = build_engine();
        let linker = build_linker(&engine).unwrap();
        let mut store = build_store(&engine);

        let app = app(component_path.clone(), engine, &mut store, linker).unwrap();

        this.component_path = component_path.to_string();
        *this.instance.borrow_mut() = Some(app);
        *this.store.borrow_mut() = Some(store);
    }

    fn triggers(&self) -> Vec<String> {
        let binding = self.0.borrow();

        let mut instance = binding.instance.borrow_mut();
        let mut store = binding.store.borrow_mut();

        if let (Some(instance), Some(store)) = (&mut *instance, &mut *store) {
            instance.standout_app_triggers().call_get_triggers(store).unwrap()
        } else {
            vec![]
        }
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("AppBridge")?;
    let app_class = module.define_class("App", ruby.class_object())?;
    app_class.define_alloc_func::<MutRApp>();
    app_class.define_method("initialize", method!(MutRApp::initialize, 1))?;
    app_class.define_method("triggers", method!(MutRApp::triggers, 0))?;

    Ok(())
}
