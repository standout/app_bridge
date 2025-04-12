use magnus::{prelude::*, Error, TryConvert, Value};
use crate::component::standout::app::types::Account;

#[magnus::wrap(class = "AppBridge::Account")]
pub struct RAccount {
    pub inner: Account,
}

impl RAccount {
    pub fn new(id: String, name: String, serialized_data: String) -> Self {
        let inner = Account {
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

impl Clone for RAccount {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl TryConvert for RAccount {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let id: String = val.funcall("id", ())?;
        let name: String = val.funcall("name", ())?;
        let serialized_data: String = val.funcall("serialized_data", ())?;

        let inner = Account {
            id,
            name,
            serialized_data,
        };

        Ok(Self { inner })
    }
}

impl From<RAccount> for Account {
    fn from(raccount: RAccount) -> Self {
        raccount.inner
    }
}
