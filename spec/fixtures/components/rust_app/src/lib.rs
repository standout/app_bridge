mod trigger_builder;
mod triggers;
use wit_bindgen::generate;

use triggers::registry::call_trigger;
use triggers::registry::trigger_ids;
use triggers::registry::register_trigger;


generate!({
    path: "./../../../../ext/app_bridge/wit",
    world: "bridge",
});

use crate::exports::standout::app::triggers::*;

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

struct App;

impl Guest for App {
    fn trigger_ids() -> Result<Vec<String>, AppError> {
        register_triggers();
        trigger_ids()
    }

    fn fetch_events(context: TriggerContext) -> Result<TriggerResponse, AppError> {
        register_triggers();
        // Call the trigger function
        call_trigger(context)
    }
}

export!(App);
