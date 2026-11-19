wit_bindgen::generate!({ path: "../../../../ext/app_bridge/wit/v6", world: "bridge", with: {} });

use serde_json::json;
use standout::app::types::*;
struct Fixture;

impl exports::standout::app::actions::Guest for Fixture {
    fn action_ids() -> Result<Vec<String>, AppError> {
        Ok(vec!["example".into()])
    }
    fn input_schema(ctx: ActionSchemaContext) -> Result<SchemaResponse, AppError> {
        if ctx.action_id != "example" {
            return Err(AppError {
                code: ErrorCode::Misconfigured,
                message: "unknown action".into(),
            });
        }
        Ok(match ctx.connection {
            None => SchemaResponse::Partial("{}".into()),
            Some(account) => SchemaResponse::Complete(json!({"title": account.name}).to_string()),
        })
    }
    fn output_schema(_: ActionSchemaContext) -> Result<SchemaResponse, AppError> {
        Ok(SchemaResponse::Complete("{}".into()))
    }
    fn execute(ctx: ActionContext) -> Result<ActionResponse, AppError> {
        Ok(ActionResponse {
            serialized_output: ctx.serialized_input,
        })
    }
}

impl exports::standout::app::triggers::Guest for Fixture {
    fn trigger_ids() -> Result<Vec<String>, AppError> {
        Ok(vec!["example".into()])
    }
    fn input_schema(ctx: TriggerSchemaContext) -> Result<SchemaResponse, AppError> {
        Ok(if ctx.connection.is_some() {
            SchemaResponse::Complete("{}".into())
        } else {
            SchemaResponse::ConnectionRequired
        })
    }
    fn output_schema(ctx: TriggerSchemaContext) -> Result<SchemaResponse, AppError> {
        Ok(SchemaResponse::Complete(
            json!({"title":ctx.store, "description":ctx.serialized_input}).to_string(),
        ))
    }
    fn fetch_events(ctx: TriggerContext) -> Result<TriggerResponse, AppError> {
        Ok(TriggerResponse {
            store: ctx.store,
            events: vec![],
        })
    }
}
impl exports::standout::app::connections::Guest for Fixture {
    fn connection_config() -> String {
        "{}".into()
    }
}
export!(Fixture);
