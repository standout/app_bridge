use magnus::{prelude::*, Error, TryConvert, Value};
use crate::component::standout::app::types::TriggerContext;
use super::account::RAccount;

#[magnus::wrap(class = "AppBridge::TriggerContext")]
pub struct RTriggerContext {
    inner: TriggerContext,
    wrapped_account: RAccount,
}

impl RTriggerContext {
    pub fn new(trigger_id: String, account: Value, store: String) -> Self {
        let account: RAccount = TryConvert::try_convert(account).unwrap();

        let inner = TriggerContext {
            trigger_id: trigger_id,
            account: account.clone().into(),
            store: store,
        };
        Self {
            inner,
            wrapped_account: account.clone(),
        }
    }

    pub fn trigger_id(&self) -> String {
        self.inner.trigger_id.clone()
    }

    pub fn account(&self) -> RAccount {
        self.wrapped_account.clone()
    }

    pub fn store(&self) -> String {
        self.inner.store.clone()
    }
}

impl TryConvert for RTriggerContext {
    fn try_convert(val: Value) -> Result<Self, Error> {
        let account_val: Value = val.funcall("account", ())?;
        let store: String = val.funcall("store", ())?;
        let trigger_id: String = val.funcall("trigger_id", ())?;

        let account: RAccount = TryConvert::try_convert(account_val)?;

        let inner = TriggerContext {
            trigger_id: trigger_id,
            account: account.clone().inner,
            store: store,
        };

        Ok(Self {
            inner,
            wrapped_account: account.clone(),
        })
    }
}

impl From<RTriggerContext> for TriggerContext {
    fn from(rtrigger_context: RTriggerContext) -> Self {
        rtrigger_context.inner
    }
}
