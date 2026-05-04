#[cfg(any(feature = "mock", test))]
pub mod mock_source;

#[cfg(any(feature = "mock", test))]
use crate::models::NormalizationResult;

#[cfg(any(feature = "mock", test))]
pub trait SourceAdapter {
    type Input;

    fn source_name(&self) -> &'static str;
    fn normalize(&self, input: Self::Input) -> crate::error::Result<Vec<NormalizationResult>>;
}
