use std::collections::VecDeque;
use std::sync::Mutex;

use super::{CvTailoringError, CvTailoringMlRequest, CvTailoringMlResponse};

#[derive(Default)]
pub struct CvTailoringServiceStub {
    responses: Mutex<VecDeque<Result<CvTailoringMlResponse, CvTailoringError>>>,
}

impl CvTailoringServiceStub {
    pub fn with_response(self, response: Result<CvTailoringMlResponse, CvTailoringError>) -> Self {
        self.responses
            .lock()
            .expect("cv tailoring stub mutex should not be poisoned")
            .push_back(response);
        self
    }

    pub(crate) fn tailor(
        &self,
        _payload: &CvTailoringMlRequest,
    ) -> Result<CvTailoringMlResponse, CvTailoringError> {
        self.responses
            .lock()
            .expect("cv tailoring stub mutex should not be poisoned")
            .pop_front()
            .unwrap_or_else(|| Err(CvTailoringError::Http("no stub response configured".to_string())))
    }
}
