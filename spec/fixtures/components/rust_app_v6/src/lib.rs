wit_bindgen::generate!({ path: "../../../../ext/app_bridge/wit/v6", world: "bridge", with: {} });

use serde_json::json;
use standout::app::types::*;
struct Fixture;

fn tenant_fields(connection: Option<Connection>) -> SchemaResponse {
    let field = connection
        .as_ref()
        .map_or("base", |account| account.name.as_str());
    let schema = json!({"type":"object","properties":{(field):{"type":"string"}}}).to_string();
    if connection.is_some() {
        SchemaResponse::Complete(schema)
    } else {
        SchemaResponse::Partial(schema)
    }
}

impl exports::standout::app::actions::Guest for Fixture {
    fn action_ids() -> Result<Vec<String>, AppError> {
        Ok(vec!["example".into(), "tenant-fields".into()])
    }
    fn input_schema(ctx: ActionSchemaContext) -> Result<SchemaResponse, AppError> {
        if ctx.action_id == "tenant-fields" {
            return Ok(tenant_fields(ctx.connection));
        }
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
    fn output_schema(ctx: ActionSchemaContext) -> Result<SchemaResponse, AppError> {
        if ctx.action_id == "tenant-fields" {
            return Ok(tenant_fields(ctx.connection));
        }
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
        json!({"strategies": [], "default_strategy": "", "runtime": {}}).to_string()
    }
}
export!(Fixture);
