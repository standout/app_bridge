use wit_bindgen::generate;

generate!({
    path: "./../../../../ext/app_bridge/wit",
    world: "bridge",
});

struct App;

impl crate::exports::standout::app::triggers::Guest for App {
    fn get_triggers() -> Vec<String> {
        vec![
            "new-invoice-payment".to_string(),
            "new-invoice".to_string()
        ]
    }
}

export!(App);
