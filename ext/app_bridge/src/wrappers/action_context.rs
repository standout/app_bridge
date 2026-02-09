use magnus::{prelude::*, scan_args::scan_args, Error, RHash, Ruby, Symbol, TryConvert, Value};
use crate::types::{ActionContext, ReferenceObject};
use super::connection::RConnection;

#[magnus::wrap(class = "AppBridge::ActionContext")]
pub struct RActionContext {
    inner: ActionContext,
    wrapped_connection: Option<RConnection>,
}

impl RActionContext {
    fn parse_reference_object(value: Value) -> Result<ReferenceObject, Error> {
        let reference: String = fetch_hash_string(value, "reference")?;
        let status: String = fetch_hash_string(value, "status")?;
        Ok(ReferenceObject {
            reference,
            status,
        })
    }

    pub fn new(args: &[Value]) -> Result<Self, Error> {
        let args = scan_args::<(String, Value, String), (Option<Value>,), (), (), (), ()>(args)?;
        let (action_id, connection, serialized_input) = args.required;
        let (retry,) = args.optional;

        Self::build(action_id, connection, serialized_input, retry)
    }

    fn build(
        action_id: String,
        connection: Value,
        serialized_input: String,
        retry: Option<Value>,
    ) -> Result<Self, Error> {
        if connection.is_nil() {
            return Err(Error::new(magnus::exception::runtime_error(), "Connection is required"));
        }

        let wrapped_connection: RConnection = match TryConvert::try_convert(connection) {
            Ok(conn) => conn,
            Err(_) => return Err(Error::new(magnus::exception::runtime_error(), "Connection is required")),
        };

        let reference_object = match retry {
            Some(value) if !value.is_nil() => Some(Self::parse_reference_object(value)?),
            _ => None,
        };

        Ok(Self {
            inner: ActionContext {
                action_id,
                connection: wrapped_connection.clone().into(),
                serialized_input,
                reference_object,
            },
            wrapped_connection: Some(wrapped_connection),
        })
    }

    pub fn action_id(&self) -> String {
        self.inner.action_id.clone()
    }

    pub fn connection(&self) -> RConnection {
        self.wrapped_connection.clone().unwrap()
    }

    pub fn serialized_input(&self) -> String {
        self.inner.serialized_input.clone()
    }

    pub fn reference_object(&self) -> Result<Value, Error> {
        let ruby = Ruby::get().unwrap();
        if let Some(reference_object) = &self.inner.reference_object {
            let hash: RHash = ruby.hash_new();
            hash.aset("reference", reference_object.reference.clone())?;
            hash.aset("status", reference_object.status.clone())?;
            Ok(hash.as_value())
        } else {
            Ok(ruby.qnil().as_value())
        }
    }
}

impl TryConvert for RActionContext {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let connection_val: Value = val.funcall("connection", ())?;
        let serialized_input: String = val.funcall("serialized_input", ())?;
        let action_id: String = val.funcall("action_id", ())?;
        let reference_object = if val.funcall("respond_to?", ("reference_object",))? {
            let reference_object_val: Value = val.funcall("reference_object", ())?;
            if reference_object_val.is_nil() {
                None
            } else {
                Some(Self::parse_reference_object(reference_object_val)?)
            }
        } else {
            None
        };

        if connection_val.is_nil() {
            return Err(Error::new(magnus::exception::runtime_error(), "Connection is required"));
        }

        let wrapped_connection: RConnection = match TryConvert::try_convert(connection_val) {
            Ok(conn) => conn,
            Err(_) => return Err(Error::new(magnus::exception::runtime_error(), "Connection is required")),
        };

        let inner = ActionContext {
            action_id,
            connection: wrapped_connection.clone().inner,
            serialized_input,
            reference_object,
        };

        Ok(Self {
            inner,
            wrapped_connection: Some(wrapped_connection),
        })
    }
}

fn fetch_hash_string(hash: Value, key: &str) -> Result<String, Error> {
    let value: Value = hash.funcall("[]", (key,))?;
    let value = if value.is_nil() {
        let symbol = Symbol::new(key);
        hash.funcall("[]", (symbol,))?
    } else {
        value
    };
    TryConvert::try_convert(value)
}


impl From<RActionContext> for ActionContext {
    fn from(raction_context: RActionContext) -> Self {
        raction_context.inner
    }
}
