use magnus::{prelude::*, Error, TryConvert, Value};
use crate::component::standout::app::types::ActionContext;
use super::account::RAccount;

#[magnus::wrap(class = "AppBridge::ActionContext")]
pub struct RActionContext {
    inner: ActionContext,
    wrapped_account: RAccount,
}

impl RActionContext {
    pub fn new(action_id: String, account: Value, serialized_input: String) -> Self {
        let account: RAccount = TryConvert::try_convert(account).unwrap();

        let inner = ActionContext {
            action_id: action_id,
            account: account.clone().into(),
            serialized_input,
        };

        Self {
            inner,
            wrapped_account: account.clone(),
        }
    }

    pub fn action_id(&self) -> String {
        self.inner.action_id.clone()
    }

    pub fn account(&self) -> RAccount {
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

        let account: RAccount = TryConvert::try_convert(account_val)?;

        let inner = ActionContext {
            action_id: action_id,
            account: account.clone().inner,
            serialized_input,
        };

        Ok(Self {
            inner,
            wrapped_account: account.clone(),
        })
    }
}

impl From<RActionContext> for ActionContext {
    fn from(raction_context: RActionContext) -> Self {
        raction_context.inner
    }
}
