mod trigger_builder;
mod action_builder;
mod triggers;
mod actions;
use wit_bindgen::generate;

use triggers::registry::call_trigger;
use triggers::registry::trigger_ids;
use actions::registry::{call_action, action_ids, register_action, input_schema, output_schema};
use triggers::registry as trigger_registry;

generate!({
    path: "./../../../../ext/app_bridge/wit",
    world: "bridge",
});

use crate::exports::standout::app::triggers::{Guest as TriggersGuest, AppError as TriggersAppError};
use crate::exports::standout::app::actions::{Guest as ActionsGuest, AppError as ActionsAppError};
use crate::standout::app::types::{TriggerContext, TriggerResponse, ActionContext, ActionResponse};

fn register_triggers() {
    // Define simple example schemas for triggers
    let trigger_input_schema = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {
            "since": { "type": "string", "description": "Fetch events since ISO timestamp" }
        },
        "additionalProperties": false
    }"#;

    let trigger_output_schema = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {
            "events": { "type": "array" },
            "store": { "type": "string" }
        }
    }"#;

    trigger_registry::register_trigger("new-photos", { |context|
        trigger_builder::get_jsonplaceholder("photos", context)
    }, trigger_input_schema, trigger_output_schema);
    trigger_registry::register_trigger("new-posts", { |context|
        trigger_builder::get_jsonplaceholder("posts", context)
    }, trigger_input_schema, trigger_output_schema);
    trigger_registry::register_trigger("new-comments", { |context|
        trigger_builder::get_jsonplaceholder("comments", context)
    }, trigger_input_schema, trigger_output_schema);
    trigger_registry::register_trigger("new-albums", { |context|
        trigger_builder::get_jsonplaceholder("albums", context)
    }, trigger_input_schema, trigger_output_schema);
    trigger_registry::register_trigger("new-todos", { |context|
        trigger_builder::get_jsonplaceholder("todos", context)
    }, trigger_input_schema, trigger_output_schema);
    trigger_registry::register_trigger("new-users", { |context|
        trigger_builder::get_jsonplaceholder("users", context)
    }, trigger_input_schema, trigger_output_schema);
}

fn register_actions() {
    // HTTP GET action schema
    let http_get_input_schema = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {
            "url": {
                "type": "string",
                "format": "uri",
                "description": "The URL to make a GET request to"
            }
        },
        "required": ["url"],
        "additionalProperties": false
    }"#;

    let http_get_output_schema = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {
            "url": {
                "type": "string",
                "description": "The URL that was requested"
            },
            "response": {
                "type": "object",
                "description": "The parsed JSON response from the HTTP request"
            }
        },
        "required": ["url", "response"]
    }"#;

    // HTTP POST action schema
    let http_post_input_schema = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {
            "url": {
                "type": "string",
                "format": "uri",
                "description": "The URL to make a POST request to"
            },
            "body": {
                "type": "string",
                "format": "code",
                "description": "The JSON body to send with the POST request"
            }
        },
        "required": ["url"],
        "additionalProperties": false
    }"#;

    let http_post_output_schema = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {
            "url": {
                "type": "string",
                "description": "The URL that was requested"
            },
            "body": {
                "type": "object",
                "description": "The body that was sent with the request"
            },
            "response": {
                "type": "object",
                "description": "The parsed JSON response from the HTTP request"
            }
        },
        "required": ["url", "body", "response"]
    }"#;

    // Create wrapper functions that match the expected signature
    let http_get_wrapper = |context: ActionContext| action_builder::http_action("http-get", context);
    let http_post_wrapper = |context: ActionContext| action_builder::http_action("http-post", context);

    register_action("http-get", http_get_wrapper, http_get_input_schema, http_get_output_schema);
    register_action("http-post", http_post_wrapper, http_post_input_schema, http_post_output_schema);

    // Complex input action schema
    let complex_input_schema = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {
            "customer": {
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "enum": ["active", "inactive", "pending"]
                    },
                    "orders": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "items": {
                                    "type": "array",
                                    "items": {
                                        "type": "object",
                                        "properties": {
                                            "sku": { "type": "string" },
                                            "quantity": { "type": "integer" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }"#;

    let complex_input_output_schema = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {
            "customer": {
                "type": "object",
                "description": "Processed customer information"
            },
            "processed": {
                "type": "boolean",
                "description": "Whether the input was processed successfully"
            }
        },
        "required": ["customer", "processed"]
    }"#;

    // Create wrapper function for complex input action
    let complex_input_wrapper = |context: ActionContext| action_builder::complex_input_action(context);

    register_action("complex-input", complex_input_wrapper, complex_input_schema, complex_input_output_schema);
}

struct App;

impl TriggersGuest for App {
    fn trigger_ids() -> Result<Vec<String>, TriggersAppError> {
        register_triggers();
        trigger_ids()
    }

    fn input_schema(trigger_id: String) -> Result<String, TriggersAppError> {
        register_triggers();
        trigger_registry::input_schema(&trigger_id)
    }

    fn output_schema(trigger_id: String) -> Result<String, TriggersAppError> {
        register_triggers();
        trigger_registry::output_schema(&trigger_id)
    }

    fn fetch_events(context: TriggerContext) -> Result<TriggerResponse, TriggersAppError> {
        register_triggers();
        // Call the trigger function
        call_trigger(context)
    }
}

impl ActionsGuest for App {
    fn action_ids() -> Result<Vec<String>, ActionsAppError> {
        register_actions();
        action_ids()
    }

    fn input_schema(action_id: String) -> Result<String, ActionsAppError> {
        register_actions();
        input_schema(&action_id)
    }

    fn output_schema(action_id: String) -> Result<String, ActionsAppError> {
        register_actions();
        output_schema(&action_id)
    }

    fn execute(context: ActionContext) -> Result<ActionResponse, ActionsAppError> {
        register_actions();
        // Call the action function
        call_action(context)
    }
}

export!(App);
