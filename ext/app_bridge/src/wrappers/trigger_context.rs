use magnus::{prelude::*, Error, TryConvert, Value};
use crate::component::standout::app::types::TriggerContext;
use super::account::RAccount;

#[magnus::wrap(class = "AppBridge::TriggerContext")]
pub struct RTriggerContext {
    inner: TriggerContext,
    wrapped_account: Option<RAccount>,
}

impl RTriggerContext {
    pub fn new(trigger_id: String, account: Value, store: String, serialized_input: String) -> Self {
        let wrapped_account = if account.is_nil() {
            None
        } else {
            match TryConvert::try_convert(account) {
                Ok(acc) => Some(acc),
                Err(_) => None,
            }
        };

        let inner = TriggerContext {
            trigger_id: trigger_id,
            account: wrapped_account.as_ref().map(|acc: &RAccount| acc.clone().into()),
            store: store,
            serialized_input: serialized_input,
        };
        Self {
            inner,
            wrapped_account,
        }
    }

    pub fn trigger_id(&self) -> String {
        self.inner.trigger_id.clone()
    }

    pub fn account(&self) -> Option<RAccount> {
        self.wrapped_account.clone()
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
        let account_val: Value = val.funcall("account", ())?;
        let store: String = val.funcall("store", ())?;
        let trigger_id: String = val.funcall("trigger_id", ())?;
        let serialized_input: String = val.funcall("serialized_input", ())?;

        let wrapped_account = if account_val.is_nil() {
            None
        } else {
            match TryConvert::try_convert(account_val) {
                Ok(acc) => Some(acc),
                Err(_) => None,
            }
        };

        let inner = TriggerContext {
            trigger_id: trigger_id,
            account: wrapped_account.as_ref().map(|acc: &RAccount| acc.clone().inner),
            store: store,
            serialized_input: serialized_input,
        };

        Ok(Self {
            inner,
            wrapped_account,
        })
    }
}

impl From<RTriggerContext> for TriggerContext {
    fn from(rtrigger_context: RTriggerContext) -> Self {
        rtrigger_context.inner
    }
}
