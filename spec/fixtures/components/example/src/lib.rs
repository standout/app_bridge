use standout::app::types::{TriggerEvent, TriggerResponse};
use wit_bindgen::generate;
use std::time::{SystemTime, UNIX_EPOCH};

generate!({
    path: "./../../../../ext/app_bridge/wit",
    world: "bridge",
});

use crate::exports::standout::app::triggers::*;

fn timestamp() -> u64 {
    let now = SystemTime::now();
    let timestamp = now.duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as u64;

    timestamp
}

struct App;

impl Guest for App {
    fn get_triggers() -> Vec<String> {
        vec![
            "new-invoice-payment".to_string(),
            "new-invoice".to_string()
        ]
    }

    fn fetch_events(context: TriggerContext) -> TriggerResponse {
        TriggerResponse {
            store: context.store,
            events: vec![
                TriggerEvent {
                    id: "1".to_string(),
                    timestamp: timestamp(),
                    serialized_data: r#"{"invoice_id": "1", "amount": 100}"#.to_string(),
                },
                TriggerEvent {
                    id: "2".to_string(),
                    timestamp: timestamp(),
                    serialized_data: r#"{"invoice_id": "2", "amount": 100}"#.to_string(),
                },
            ],
        }
    }
}

export!(App);
