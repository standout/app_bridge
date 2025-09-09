use magnus::{prelude::*, Error, TryConvert, Value};
use crate::component::standout::app::types::ActionContext;
use super::account::RAccount;

#[magnus::wrap(class = "AppBridge::ActionContext")]
pub struct RActionContext {
    inner: ActionContext,
    wrapped_account: Option<RAccount>,
}

impl RActionContext {
    pub fn new(action_id: String, account: Value, serialized_input: String) -> Self {
        let wrapped_account = if account.is_nil() {
            None
        } else {
            match TryConvert::try_convert(account) {
                Ok(acc) => Some(acc),
                Err(_) => None,
            }
        };

        let inner = ActionContext {
            action_id: action_id,
            account: wrapped_account.as_ref().map(|acc: &RAccount| acc.clone().into()),
            serialized_input,
        };

        Self {
            inner,
            wrapped_account,
        }
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

        let wrapped_account = if account_val.is_nil() {
            None
        } else {
            match TryConvert::try_convert(account_val) {
                Ok(acc) => Some(acc),
                Err(_) => None,
            }
        };

        let inner = ActionContext {
            action_id: action_id,
            account: wrapped_account.as_ref().map(|acc: &RAccount| acc.clone().inner),
            serialized_input,
        };

        Ok(Self {
            inner,
            wrapped_account,
        })
    }
}

impl From<RActionContext> for ActionContext {
    fn from(raction_context: RActionContext) -> Self {
        raction_context.inner
    }
}
