mod date_utils;
mod fetch_events;
use wit_bindgen::generate;

generate!({
    path: "./../../../../ext/app_bridge/wit",
    world: "bridge",
});

use crate::exports::standout::app::triggers::*;

struct App;

impl Guest for App {
    fn get_triggers() -> Vec<String> {
        vec![
            "new-invoice-payment".to_string(),
            "new-invoice".to_string()
        ]
    }

    fn fetch_events(context: TriggerContext) -> TriggerResponse {
        fetch_events::main(context)
    }
}

export!(App);
