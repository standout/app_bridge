use crate::component::standout::app::types::ActionResponse;

#[magnus::wrap(class = "AppBridge::ActionResponse")]
pub struct RActionResponse {
    inner: ActionResponse,
}

impl RActionResponse {
    pub fn new(serialized_output: String) -> Self {
        let inner = ActionResponse {
            serialized_output: serialized_output,
        };
        Self { inner }
    }

    pub fn serialized_output(&self) -> String {
        self.inner.serialized_output.clone()
    }

    /// Returns a new ActionResponse with the given output
    pub fn with_output(&self, value: String) -> Self {
        Self::new(value)
    }
}

impl From<ActionResponse> for RActionResponse {
    fn from(value: ActionResponse) -> Self {
        Self { inner: value }
    }
}

impl From<RActionResponse> for ActionResponse {
    fn from(value: RActionResponse) -> Self {
        value.inner
    }
}

