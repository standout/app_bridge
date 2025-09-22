use magnus::{prelude::*, Error, TryConvert, Value};
use crate::component::standout::app::types::TriggerContext;
use super::connection::RConnection;

#[magnus::wrap(class = "AppBridge::TriggerContext")]
pub struct RTriggerContext {
    inner: TriggerContext,
    wrapped_connection: Option<RConnection>,
}

impl RTriggerContext {
    pub fn new(trigger_id: String, connection: Value, store: String, serialized_input: String) -> Result<Self, Error> {
        if connection.is_nil() {
            return Err(Error::new(magnus::exception::runtime_error(), "Connection is required"));
        }

        let wrapped_connection: RConnection = match TryConvert::try_convert(connection) {
            Ok(conn) => conn,
            Err(_) => return Err(Error::new(magnus::exception::runtime_error(), "Connection is required")),
        };

        let inner = TriggerContext {
            trigger_id: trigger_id,
            connection: wrapped_connection.clone().into(),
            store: store,
            serialized_input: serialized_input,
        };
        Ok(Self {
            inner,
            wrapped_connection: Some(wrapped_connection),
        })
    }

    pub fn trigger_id(&self) -> String {
        self.inner.trigger_id.clone()
    }

    pub fn connection(&self) -> Option<RConnection> {
        self.wrapped_connection.clone()
    }

    pub fn store(&self) -> String {
        self.inner.store.clone()
    }

    pub fn serialized_input(&self) -> String {
        self.inner.serialized_input.clone()
    }
}

impl TryConvert for RTriggerContext {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let connection_val: Value = val.funcall("connection", ())?;
        let store: String = val.funcall("store", ())?;
        let trigger_id: String = val.funcall("trigger_id", ())?;
        let serialized_input: String = val.funcall("serialized_input", ())?;

        if connection_val.is_nil() {
            return Err(Error::new(magnus::exception::runtime_error(), "Connection is required"));
        }

        let wrapped_connection: RConnection = match TryConvert::try_convert(connection_val) {
            Ok(conn) => conn,
            Err(_) => return Err(Error::new(magnus::exception::runtime_error(), "Connection is required")),
        };

        let inner = TriggerContext {
            trigger_id: trigger_id,
            connection: wrapped_connection.clone().inner,
            store: store,
            serialized_input: serialized_input,
        };

        Ok(Self {
            inner,
            wrapped_connection: Some(wrapped_connection),
        })
    }
}

impl From<RTriggerContext> for TriggerContext {
    fn from(rtrigger_context: RTriggerContext) -> Self {
        rtrigger_context.inner
    }
}
