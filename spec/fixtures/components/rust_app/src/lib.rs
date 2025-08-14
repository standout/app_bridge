mod trigger_builder;
mod action_builder;
mod triggers;
mod actions;
use wit_bindgen::generate;

use triggers::registry::call_trigger;
use triggers::registry::trigger_ids;
use triggers::registry::register_trigger;
use actions::registry::call_action;
use actions::registry::action_ids;
use actions::registry::register_action;

generate!({
    path: "./../../../../ext/app_bridge/wit",
    world: "bridge",
});

use crate::exports::standout::app::triggers::{Guest as TriggersGuest, AppError as TriggersAppError};
use crate::exports::standout::app::actions::{Guest as ActionsGuest, AppError as ActionsAppError};
use crate::standout::app::types::{TriggerContext, TriggerResponse, ActionContext, ActionResponse};

fn register_triggers() {
    register_trigger("new-photos", { |context|
        trigger_builder::get_jsonplaceholder("photos", context)
    });
    register_trigger("new-posts", { |context|
        trigger_builder::get_jsonplaceholder("posts", context)
    });
    register_trigger("new-comments", { |context|
        trigger_builder::get_jsonplaceholder("comments", context)
    });
    register_trigger("new-albums", { |context|
        trigger_builder::get_jsonplaceholder("albums", context)
    });
    register_trigger("new-todos", { |context|
        trigger_builder::get_jsonplaceholder("todos", context)
    });
    register_trigger("new-users", { |context|
        trigger_builder::get_jsonplaceholder("users", context)
    });
}

fn register_actions() {
    register_action("http-get", { |context|
        action_builder::http_action("http-get", context)
    });
    register_action("http-post", { |context|
        action_builder::http_action("http-post", context)
    });
}

struct App;

impl TriggersGuest for App {
    fn trigger_ids() -> Result<Vec<String>, TriggersAppError> {
        register_triggers();
        trigger_ids()
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

    fn execute(context: ActionContext) -> Result<ActionResponse, ActionsAppError> {
        register_actions();
        // Call the action function
        call_action(context)
    }
}

export!(App);
