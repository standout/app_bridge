mod trigger_builder;
mod action_builder;
mod triggers;
mod actions;
use wit_bindgen::generate;
use ctor::ctor;

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
            "include_extra_data": {
                "type": "boolean",
                "description": "Whether to include additional data in the response"
            }
        },
        "required": ["include_extra_data"],
        "additionalProperties": false
    }"#;

    let trigger_output_schema = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {
            "events": { "type": "array" },
            "store": { "type": "string" }
        },
        "required": ["events", "store"],
        "additionalProperties": false
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

#[ctor]
fn init_registries() {
    register_triggers();
    register_actions();
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
                "description": "The parsed JSON response from the HTTP request",
                "properties": {
                    "status": {
                        "type": "integer",
                        "description": "HTTP status code"
                    },
                    "headers": {
                        "type": "object",
                        "description": "Response headers",
                        "additionalProperties": {
                            "type": "string"
                        }
                    },
                    "data": {
                        "type": "object",
                        "description": "Response data",
                        "additionalProperties": true
                    }
                },
                "required": ["status"],
                "additionalProperties": true
            }
        },
        "required": ["url", "response"],
        "additionalProperties": false
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
                "description": "The body that was sent with the request",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "Request body content"
                    },
                    "content_type": {
                        "type": "string",
                        "description": "Content type of the request"
                    }
                },
                "additionalProperties": true
            },
            "response": {
                "type": "object",
                "description": "The parsed JSON response from the HTTP request",
                "properties": {
                    "status": {
                        "type": "integer",
                        "description": "HTTP status code"
                    },
                    "headers": {
                        "type": "object",
                        "description": "Response headers",
                        "additionalProperties": {
                            "type": "string"
                        }
                    },
                    "data": {
                        "type": "object",
                        "description": "Response data",
                        "additionalProperties": true
                    }
                },
                "required": ["status"],
                "additionalProperties": true
            }
        },
        "required": ["url", "body", "response"],
        "additionalProperties": false
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
                "title": "Customer name",
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
            },
            "metadata": {
                "type": "object",
                "title": "Custom Metadata",
                "description": "Additional metadata as key-value pairs",
                "propertyNames": {
                    "type": "string",
                    "title": "Field Name"
                },
                "additionalProperties": {
                    "type": "string",
                    "title": "Field Value"
                }
            }
        },
        "required": ["customer"],
        "additionalProperties": false
    }"#;

    let complex_input_output_schema = r#"{
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "properties": {
            "customer": {
                "type": "object",
                "description": "Customer information with status and order history",
                "properties": {
                    "status": {
                        "type": "string",
                        "enum": ["active", "inactive", "pending"],
                        "description": "Current status of the customer account"
                    },
                    "orders": {
                        "type": "array",
                        "description": "List of customer orders",
                        "items": {
                            "type": "object",
                            "description": "Individual order containing items",
                            "properties": {
                                "items": {
                                    "type": "array",
                                    "description": "Items within this order",
                                    "items": {
                                        "type": "object",
                                        "description": "Individual item in the order",
                                        "properties": {
                                            "sku": {
                                                "type": "string",
                                                "description": "Stock Keeping Unit identifier for the product"
                                            },
                                            "quantity": {
                                                "type": "integer",
                                                "description": "Number of units of this item ordered"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "metadata": {
                "type": "object",
                "title": "Custom Metadata",
                "description": "Additional metadata as key-value pairs",
                "propertyNames": {
                    "type": "string",
                    "title": "Field Name"
                },
                "additionalProperties": {
                    "type": "string",
                    "title": "Field Value"
                }
            },
            "environment_variables": {
                "type": "object",
                "description": "Environment variables passed to the app at runtime",
                "propertyNames": {
                    "type": "string",
                    "title": "Variable Name"
                },
                "additionalProperties": {
                    "type": "string",
                    "title": "Variable Value"
                }
            }
        },
        "required": ["customer", "environment_variables"],
        "additionalProperties": false
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

    fn input_schema(context: TriggerContext) -> Result<String, TriggersAppError> {
        register_triggers();

        // Check if account has custom field enabled
        if let Some(account) = &context.account {
            if let Ok(account_data) = serde_json::from_str::<serde_json::Value>(&account.serialized_data) {
                if let Some(custom) = account_data.get("custom") {
                    if custom.as_bool() == Some(true) {
                        // Return enhanced schema with custom field for new-posts trigger
                        if context.trigger_id == "new-posts" {
                                                    return Ok(r#"{
                            "$schema": "https://json-schema.org/draft/2020-12/schema",
                            "type": "object",
                            "properties": {
                                "include_extra_data": { "type": "boolean", "description": "Whether to include additional data in the response" },
                                "include_custom_data": { "type": "boolean", "description": "Whether to include custom data for premium accounts" }
                            },
                            "required": ["include_extra_data"],
                            "additionalProperties": false
                        }"#.to_string());
                        }
                    }
                }
            }
        }

        // Return base schema
        trigger_registry::input_schema(&context.trigger_id)
    }

    fn output_schema(context: TriggerContext) -> Result<String, TriggersAppError> {
        register_triggers();

        // Check if account has custom field enabled
        if let Some(account) = &context.account {
            if let Ok(account_data) = serde_json::from_str::<serde_json::Value>(&account.serialized_data) {
                if let Some(custom) = account_data.get("custom") {
                    if custom.as_bool() == Some(true) {
                        // Return enhanced schema with custom metadata for new-posts trigger
                        if context.trigger_id == "new-posts" {
                            return Ok(r#"{
                                "$schema": "https://json-schema.org/draft/2020-12/schema",
                                "type": "object",
                                "properties": {
                                    "events": { "type": "array" },
                                    "store": { "type": "string" },
                                    "custom_metadata": {
                                        "type": "object",
                                        "description": "Additional metadata for premium accounts",
                                        "properties": {
                                            "priority": {
                                                "type": "string",
                                                "enum": ["low", "medium", "high"],
                                                "description": "Priority level for the trigger"
                                            },
                                            "tags": {
                                                "type": "array",
                                                "items": {
                                                    "type": "string"
                                                },
                                                "description": "Tags associated with the trigger"
                                            }
                                        },
                                        "additionalProperties": false
                                    }
                                },
                                "required": ["events", "store"],
                                "additionalProperties": false
                            }"#.to_string());
                        }
                    }
                }
            }
        }

        // Return base schema
        trigger_registry::output_schema(&context.trigger_id)
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

    fn input_schema(context: ActionContext) -> Result<String, ActionsAppError> {
        register_actions();

        // Check if account has custom field enabled
        if let Some(account) = &context.account {
            if let Ok(account_data) = serde_json::from_str::<serde_json::Value>(&account.serialized_data) {
                if let Some(custom) = account_data.get("custom") {
                    if custom.as_bool() == Some(true) {
                        // Return enhanced schema with custom headers for http-post action
                        if context.action_id == "http-post" {
                            return Ok(r#"{
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
                                    },
                                    "custom_headers": {
                                        "type": "object",
                                        "description": "Custom headers for premium accounts",
                                        "properties": {
                                            "authorization": {
                                                "type": "string",
                                                "description": "Authorization header value"
                                            },
                                            "x_custom_id": {
                                                "type": "string",
                                                "description": "Custom identifier header"
                                            }
                                        },
                                        "additionalProperties": false
                                    }
                                },
                                "required": ["url"],
                                "additionalProperties": false
                            }"#.to_string());
                        }
                    }
                }
            }
        }

        // Return base schema
        input_schema(&context.action_id)
    }

    fn output_schema(context: ActionContext) -> Result<String, ActionsAppError> {
        register_actions();

        // Check if account has custom field enabled
        if let Some(account) = &context.account {
            if let Ok(account_data) = serde_json::from_str::<serde_json::Value>(&account.serialized_data) {
                if let Some(custom) = account_data.get("custom") {
                    if custom.as_bool() == Some(true) {
                        // Return enhanced schema with custom metadata for http-post action
                        if context.action_id == "http-post" {
                            return Ok(r#"{
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
                                    },
                                    "custom_metadata": {
                                        "type": "object",
                                        "description": "Additional metadata for premium accounts",
                                        "properties": {
                                            "execution_time": {
                                                "type": "number",
                                                "description": "Time taken to execute the action in milliseconds"
                                            },
                                            "rate_limit_info": {
                                                "type": "object",
                                                "properties": {
                                                    "remaining": {
                                                        "type": "integer",
                                                        "description": "Remaining API calls"
                                                    },
                                                    "reset_time": {
                                                        "type": "string",
                                                        "format": "date-time",
                                                        "description": "When the rate limit resets"
                                                    }
                                                },
                                                "additionalProperties": false
                                            }
                                        },
                                        "additionalProperties": false
                                    }
                                },
                                "required": ["url", "body", "response"],
                                "additionalProperties": false
                            }"#.to_string());
                        }
                    }
                }
            }
        }

        // Return base schema
        output_schema(&context.action_id)
    }

    fn execute(context: ActionContext) -> Result<ActionResponse, ActionsAppError> {
        register_actions();
        // Call the action function
        call_action(context)
    }
}

export!(App);
