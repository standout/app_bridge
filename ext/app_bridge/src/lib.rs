use magnus::{function, method, prelude::*, Error, RArray, Ruby, TryConvert, Value};
use std::cell::RefCell;
use wasmtime::Store;
mod app_state;
mod component;
mod request_builder;

use app_state::AppState;
use component::standout::app::types::{Account, TriggerContext, TriggerEvent, TriggerResponse};
use component::{app, build_engine, build_linker, build_store, Bridge};

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
            instance
                .standout_app_triggers()
                .call_get_triggers(store)
                .unwrap()
        } else {
            vec![]
        }
    }

    fn rb_fetch_events(&self, context: Value) -> RTriggerResponse {
        let context: RTriggerContext = TryConvert::try_convert(context).unwrap();
        let response = self.fetch_events(context.inner);

        RTriggerResponse::from_trigger_response(response)
    }

    fn fetch_events(&self, context: TriggerContext) -> TriggerResponse {
        let binding = self.0.borrow();

        let mut instance = binding.instance.borrow_mut();
        let mut store = binding.store.borrow_mut();

        if let (Some(instance), Some(store)) = (&mut *instance, &mut *store) {
            instance
                .standout_app_triggers()
                .call_fetch_events(store, &context)
                .unwrap()
        } else {
            TriggerResponse {
                store: context.store,
                events: vec![],
            }
        }
    }
}

#[magnus::wrap(class = "AppBridge::Account")]
struct RAccount {
    inner: Account,
}

impl RAccount {
    fn new(id: String, name: String, serialized_data: String) -> Self {
        let inner = Account {
            id: id,
            name: name,
            serialized_data: serialized_data,
        };
        Self { inner }
    }

    fn id(&self) -> String {
        self.inner.id.clone()
    }

    fn name(&self) -> String {
        self.inner.name.clone()
    }

    fn serialized_data(&self) -> String {
        self.inner.serialized_data.clone()
    }
}

impl Clone for RAccount {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl TryConvert for RAccount {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let id: String = val.funcall("id", ())?;
        let name: String = val.funcall("name", ())?;
        let serialized_data: String = val.funcall("serialized_data", ())?;

        let inner = Account {
            id,
            name,
            serialized_data,
        };

        Ok(Self { inner })
    }
}

#[magnus::wrap(class = "AppBridge::TriggerContext")]
struct RTriggerContext {
    inner: TriggerContext,
    wrapped_account: RAccount,
}

impl RTriggerContext {
    fn new(trigger_id: String, account: Value, store: String) -> Self {
        let account: RAccount = TryConvert::try_convert(account).unwrap();

        let inner = TriggerContext {
            trigger_id: trigger_id,
            account: account.clone().inner,
            store: store,
        };
        Self {
            inner,
            wrapped_account: account.clone(),
        }
    }

    fn trigger_id(&self) -> String {
        self.inner.trigger_id.clone()
    }

    fn account(&self) -> RAccount {
        self.wrapped_account.clone()
    }

    fn store(&self) -> String {
        self.inner.store.clone()
    }
}

impl TryConvert for RTriggerContext {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let account_val: Value = val.funcall("account", ())?;
        let store: String = val.funcall("store", ())?;
        let trigger_id: String = val.funcall("trigger_id", ())?;

        let account: RAccount = TryConvert::try_convert(account_val).unwrap();

        let inner = TriggerContext {
            trigger_id: trigger_id,
            account: account.clone().inner,
            store: store,
        };

        Ok(Self {
            inner,
            wrapped_account: account.clone(),
        })
    }
}

#[magnus::wrap(class = "AppBridge::TriggerResponse")]
struct RTriggerResponse {
    inner: TriggerResponse,
}

impl RTriggerResponse {
    fn new(store: String, events: RArray) -> Self {
        let iter = events.into_iter();
        let res: Vec<RTriggerEvent> = iter
            .map(&TryConvert::try_convert)
            .collect::<Result<Vec<RTriggerEvent>, Error>>()
            .unwrap();

        let inner = TriggerResponse {
            store: store,
            events: res.iter().map(|e| e.inner.clone()).collect(),
        };
        Self { inner }
    }

    fn from_trigger_response(inner: TriggerResponse) -> Self {
        Self { inner }
    }

    fn store(&self) -> String {
        self.inner.store.clone()
    }

    fn events(&self) -> RArray {
        self.inner
            .events
            .iter()
            .map(|e| RTriggerEvent::new(e.id.clone(), e.timestamp, e.serialized_data.clone()))
            .collect()
    }
}

#[magnus::wrap(class = "AppBridge::TriggerEvent")]
struct RTriggerEvent {
    inner: TriggerEvent,
}

impl RTriggerEvent {
    fn new(id: String, timestamp: u64, serialized_data: String) -> Self {
        let inner = TriggerEvent {
            id: id,
            timestamp: timestamp,
            serialized_data: serialized_data,
        };
        Self { inner }
    }

    fn id(&self) -> String {
        self.inner.id.clone()
    }

    fn timestamp(&self) -> u64 {
        self.inner.timestamp
    }

    fn serialized_data(&self) -> String {
        self.inner.serialized_data.clone()
    }
}

impl TryConvert for RTriggerEvent {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let id: String = val.funcall("id", ())?;
        let timestamp: u64 = val.funcall("timestamp", ())?;
        let serialized_data: String = val.funcall("serialized_data", ())?;

        let inner = TriggerEvent {
            id,
            timestamp,
            serialized_data,
        };

        Ok(Self { inner })
    }
}

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("AppBridge")?;

    // Define the Accout class
    let account_class = module.define_class("Account", ruby.class_object())?;
    account_class.define_singleton_method("new", function!(RAccount::new, 3))?;
    account_class.define_method("id", method!(RAccount::id, 0))?;
    account_class.define_method("name", method!(RAccount::name, 0))?;
    account_class.define_method("serialized_data", method!(RAccount::serialized_data, 0))?;

    let trigger_event_class = module.define_class("TriggerEvent", ruby.class_object())?;
    trigger_event_class.define_singleton_method("new", function!(RTriggerEvent::new, 3))?;
    trigger_event_class.define_method("id", method!(RTriggerEvent::id, 0))?;
    trigger_event_class.define_method("timestamp", method!(RTriggerEvent::timestamp, 0))?;
    trigger_event_class.define_method(
        "serialized_data",
        method!(RTriggerEvent::serialized_data, 0),
    )?;

    let trigger_response_class = module.define_class("TriggerResponse", ruby.class_object())?;
    trigger_response_class.define_singleton_method("new", function!(RTriggerResponse::new, 2))?;
    trigger_response_class.define_method("store", method!(RTriggerResponse::store, 0))?;
    trigger_response_class.define_method("events", method!(RTriggerResponse::events, 0))?;

    let trigger_context_class = module.define_class("TriggerContext", ruby.class_object())?;
    trigger_context_class.define_singleton_method("new", function!(RTriggerContext::new, 3))?;
    trigger_context_class.define_method("trigger_id", method!(RTriggerContext::trigger_id, 0))?;
    trigger_context_class.define_method("account", method!(RTriggerContext::account, 0))?;
    trigger_context_class.define_method("store", method!(RTriggerContext::store, 0))?;

    // Define the App class
    let app_class = module.define_class("App", ruby.class_object())?;
    app_class.define_alloc_func::<MutRApp>();
    app_class.define_method("initialize", method!(MutRApp::initialize, 1))?;
    app_class.define_method("triggers", method!(MutRApp::triggers, 0))?;
    app_class.define_private_method("_rust_fetch_events", method!(MutRApp::rb_fetch_events, 1))?;


    Ok(())
}
