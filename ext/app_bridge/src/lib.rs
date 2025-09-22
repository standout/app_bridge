use magnus::{function, method, prelude::*, Error, Ruby};
mod app_state;
mod component;
mod request_builder;
mod error_mapping;

mod wrappers;
use wrappers::connection::RConnection;
use wrappers::trigger_context::RTriggerContext;
use wrappers::trigger_event::RTriggerEvent;
use wrappers::trigger_response::RTriggerResponse;
use wrappers::action_context::RActionContext;
use wrappers::action_response::RActionResponse;
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

    // Define the Connection class
    let connection_class = module.define_class("Connection", ruby.class_object())?;
    connection_class.define_singleton_method("new", function!(RConnection::new, 3))?;
    connection_class.define_method("id", method!(RConnection::id, 0))?;
    connection_class.define_method("name", method!(RConnection::name, 0))?;
    connection_class.define_method("serialized_data", method!(RConnection::serialized_data, 0))?;

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
    trigger_context_class.define_singleton_method("new", function!(RTriggerContext::new, 4))?;
    trigger_context_class.define_method("trigger_id", method!(RTriggerContext::trigger_id, 0))?;
    trigger_context_class.define_method("connection", method!(RTriggerContext::connection, 0))?;
    trigger_context_class.define_method("store", method!(RTriggerContext::store, 0))?;
    trigger_context_class.define_method("serialized_input", method!(RTriggerContext::serialized_input, 0))?;

    // Define the Action classes
    let action_context_class = module.define_class("ActionContext", ruby.class_object())?;
    action_context_class.define_singleton_method("new", function!(RActionContext::new, 3))?;
    action_context_class.define_method("action_id", method!(RActionContext::action_id, 0))?;
    action_context_class.define_method("connection", method!(RActionContext::connection, 0))?;
    action_context_class.define_method("serialized_input", method!(RActionContext::serialized_input, 0))?;

    let action_response_class = module.define_class("ActionResponse", ruby.class_object())?;
    action_response_class.define_singleton_method("new", function!(RActionResponse::new, 1))?;
    action_response_class.define_method("serialized_output", method!(RActionResponse::serialized_output, 0))?;

    // Define the App class
    let app_class = module.define_class("App", ruby.class_object())?;
    app_class.define_alloc_func::<MutRApp>();
    app_class.define_method("trigger_ids", method!(MutRApp::trigger_ids, 0))?;
    app_class.define_method("action_ids", method!(MutRApp::action_ids, 0))?;
    app_class.define_method("action_input_schema", method!(MutRApp::action_input_schema, 1))?;
    app_class.define_method("action_output_schema", method!(MutRApp::action_output_schema, 1))?;
    app_class.define_method("trigger_input_schema", method!(MutRApp::trigger_input_schema, 1))?;
    app_class.define_method("trigger_output_schema", method!(MutRApp::trigger_output_schema, 1))?;
    app_class.define_private_method("_rust_initialize", method!(MutRApp::initialize, 2))?;
    app_class.define_private_method("_rust_fetch_events", method!(MutRApp::rb_fetch_events, 1))?;
    app_class.define_private_method("_rust_execute_action", method!(MutRApp::rb_execute_action, 1))?;

    Ok(())
}
