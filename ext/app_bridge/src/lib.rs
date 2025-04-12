use magnus::{function, method, prelude::*, Error, Ruby};
mod app_state;
mod component;
mod request_builder;
mod error_mapping;

mod wrappers;
use wrappers::account::RAccount;
use wrappers::trigger_context::RTriggerContext;
use wrappers::trigger_event::RTriggerEvent;
use wrappers::trigger_response::RTriggerResponse;
use wrappers::app::MutRApp;

#[magnus::init]
fn init(ruby: &Ruby) -> Result<(), Error> {
    let module = ruby.define_module("AppBridge")?;

    let error = module.define_error("Error", ruby.exception_standard_error())?;
    module.define_error("UnauthenticatedError", error)?;
    module.define_error("ForbiddenError", error)?;
    module.define_error("MisconfiguredError", error)?;
    module.define_error("UnsupportedError", error)?;
    module.define_error("RateLimitError", error)?;
    module.define_error("TimeoutError", error)?;
    module.define_error("UnavailableError", error)?;
    module.define_error("InternalError", error)?;
    module.define_error("MalformedResponseError", error)?;
    module.define_error("OtherError", error)?;

    // Define the Accout class
    let account_class = module.define_class("Account", ruby.class_object())?;
    account_class.define_singleton_method("new", function!(RAccount::new, 3))?;
    account_class.define_method("id", method!(RAccount::id, 0))?;
    account_class.define_method("name", method!(RAccount::name, 0))?;
    account_class.define_method("serialized_data", method!(RAccount::serialized_data, 0))?;

    let trigger_event_class = module.define_class("TriggerEvent", ruby.class_object())?;
    trigger_event_class.define_singleton_method("new", function!(RTriggerEvent::new, 2))?;
    trigger_event_class.define_method("id", method!(RTriggerEvent::id, 0))?;
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
    app_class.define_method("trigger_ids", method!(MutRApp::trigger_ids, 0))?;
    app_class.define_private_method("_rust_fetch_events", method!(MutRApp::rb_fetch_events, 1))?;

    Ok(())
}
