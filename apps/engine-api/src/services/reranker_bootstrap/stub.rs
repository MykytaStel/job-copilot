use std::collections::VecDeque;
use std::sync::Mutex;

use super::{BootstrapRequestPayload, BootstrapResponsePayload, RerankerBootstrapError};

#[derive(Default)]
pub struct RerankerBootstrapServiceStub {
    responses: Mutex<VecDeque<Result<BootstrapResponsePayload, RerankerBootstrapError>>>,
}

impl RerankerBootstrapServiceStub {
    pub fn with_response(
        self,
        response: Result<BootstrapResponsePayload, RerankerBootstrapError>,
    ) -> Self {
        self.responses
            .lock()
            .expect("reranker bootstrap stub mutex should not be poisoned")
            .push_back(response);
        self
    }

    pub(crate) fn bootstrap(
        &self,
        _payload: &BootstrapRequestPayload,
    ) -> Result<BootstrapResponsePayload, RerankerBootstrapError> {
        self.responses
            .lock()
            .expect("reranker bootstrap stub mutex should not be poisoned")
            .pop_front()
            .unwrap_or_else(|| {
                Ok(BootstrapResponsePayload {
                    retrained: false,
                    example_count: 0,
                    reason: Some("no stub bootstrap response configured".to_string()),
                    model_path: None,
                    artifact_version: None,
                    model_type: None,
                    training: None,
                    evaluation: None,
                    benchmark: None,
                    feature_importances: None,
                })
            })
    }
}
