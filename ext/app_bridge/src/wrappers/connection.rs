use magnus::{prelude::*, Error, TryConvert, Value};
use crate::component::standout::app::types::Connection;

#[magnus::wrap(class = "AppBridge::Connection")]
pub struct RConnection {
    pub inner: Connection,
}

impl RConnection {
    pub fn new(id: String, name: String, serialized_data: String) -> Self {
        let inner = Connection {
            id: id,
            name: name,
            serialized_data: serialized_data,
        };
        Self { inner }
    }

    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    pub fn name(&self) -> String {
        self.inner.name.clone()
    }

    pub fn serialized_data(&self) -> String {
        self.inner.serialized_data.clone()
    }
}

impl Clone for RConnection {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl TryConvert for RConnection {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let id: String = val.funcall("id", ())?;
        let name: String = val.funcall("name", ())?;
        let serialized_data: String = val.funcall("serialized_data", ())?;

        let inner = Connection {
            id,
            name,
            serialized_data,
        };

        Ok(Self { inner })
    }
}

impl From<RConnection> for Connection {
    fn from(rconnection: RConnection) -> Self {
        rconnection.inner
    }
}
