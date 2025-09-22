use magnus::{prelude::*, Error, TryConvert, Value};
use crate::component::standout::app::types::ActionContext;
use super::connection::RConnection;

#[magnus::wrap(class = "AppBridge::ActionContext")]
pub struct RActionContext {
    inner: ActionContext,
    wrapped_connection: Option<RConnection>,
}

impl RActionContext {
    pub fn new(action_id: String, connection: Value, serialized_input: String) -> Result<Self, Error> {
        if connection.is_nil() {
            return Err(Error::new(magnus::exception::runtime_error(), "Connection is required"));
        }

        let wrapped_connection: RConnection = match TryConvert::try_convert(connection) {
            Ok(conn) => conn,
            Err(_) => return Err(Error::new(magnus::exception::runtime_error(), "Connection is required")),
        };

        let inner = ActionContext {
            action_id: action_id,
            connection: wrapped_connection.clone().into(),
            serialized_input,
        };

        Ok(Self {
            inner,
            wrapped_connection: Some(wrapped_connection),
        })
    }

    pub fn action_id(&self) -> String {
        self.inner.action_id.clone()
    }

    pub fn connection(&self) -> Option<RConnection> {
        self.wrapped_connection.clone()
    }

    pub fn serialized_input(&self) -> String {
        self.inner.serialized_input.clone()
    }
}

impl TryConvert for RActionContext {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let connection_val: Value = val.funcall("connection", ())?;
        let serialized_input: String = val.funcall("serialized_input", ())?;
        let action_id: String = val.funcall("action_id", ())?;

        if connection_val.is_nil() {
            return Err(Error::new(magnus::exception::runtime_error(), "Connection is required"));
        }

        let wrapped_connection: RConnection = match TryConvert::try_convert(connection_val) {
            Ok(conn) => conn,
            Err(_) => return Err(Error::new(magnus::exception::runtime_error(), "Connection is required")),
        };

        let inner = ActionContext {
            action_id: action_id,
            connection: wrapped_connection.clone().inner,
            serialized_input,
        };

        Ok(Self {
            inner,
            wrapped_connection: Some(wrapped_connection),
        })
    }
}

impl From<RActionContext> for ActionContext {
    fn from(raction_context: RActionContext) -> Self {
        raction_context.inner
    }
}
