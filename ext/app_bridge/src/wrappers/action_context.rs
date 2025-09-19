use magnus::{prelude::*, Error, TryConvert, Value};
use crate::component::standout::app::types::ActionContext;
use super::account::RAccount;

#[magnus::wrap(class = "AppBridge::ActionContext")]
pub struct RActionContext {
    inner: ActionContext,
    wrapped_account: Option<RAccount>,
}

impl RActionContext {
    pub fn new(action_id: String, account: Value, serialized_input: String) -> Result<Self, Error> {
        if account.is_nil() {
            return Err(Error::new(magnus::exception::runtime_error(), "Account is required"));
        }

        let wrapped_account: RAccount = match TryConvert::try_convert(account) {
            Ok(acc) => acc,
            Err(_) => return Err(Error::new(magnus::exception::runtime_error(), "Account is required")),
        };

        let inner = ActionContext {
            action_id: action_id,
            account: wrapped_account.clone().into(),
            serialized_input,
        };

        Ok(Self {
            inner,
            wrapped_account: Some(wrapped_account),
        })
    }

    pub fn action_id(&self) -> String {
        self.inner.action_id.clone()
    }

    pub fn account(&self) -> Option<RAccount> {
        self.wrapped_account.clone()
    }

    pub fn serialized_input(&self) -> String {
        self.inner.serialized_input.clone()
    }
}

impl TryConvert for RActionContext {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let account_val: Value = val.funcall("account", ())?;
        let serialized_input: String = val.funcall("serialized_input", ())?;
        let action_id: String = val.funcall("action_id", ())?;

        if account_val.is_nil() {
            return Err(Error::new(magnus::exception::runtime_error(), "Account is required"));
        }

        let wrapped_account: RAccount = match TryConvert::try_convert(account_val) {
            Ok(acc) => acc,
            Err(_) => return Err(Error::new(magnus::exception::runtime_error(), "Account is required")),
        };

        let inner = ActionContext {
            action_id: action_id,
            account: wrapped_account.clone().inner,
            serialized_input,
        };

        Ok(Self {
            inner,
            wrapped_account: Some(wrapped_account),
        })
    }
}

impl From<RActionContext> for ActionContext {
    fn from(raction_context: RActionContext) -> Self {
        raction_context.inner
    }
}
