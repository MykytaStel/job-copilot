pub mod mock_source;

use crate::models::NormalizationResult;

pub trait SourceAdapter {
    type Input;

    fn source_name(&self) -> &'static str;
    fn normalize(&self, input: Self::Input) -> Result<Vec<NormalizationResult>, String>;
}
