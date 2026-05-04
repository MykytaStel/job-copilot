pub mod dedupe;
pub mod input;
pub mod model;
pub mod normalized;
pub mod variant;

pub use dedupe::{canonical_job_id, compute_dedupe_key};
pub use input::{IngestionInput, InputDocument};
pub use model::{Job, JobFeedSummary, JobLifecycleStage, JobSourceVariant, JobView};
pub use normalized::{CompanyMeta, NormalizedJob};
pub use variant::{IngestionBatch, JobVariant, NormalizationResult, RawSnapshot};

#[cfg(any(feature = "mock", test))]
pub use input::{MockCompensation, MockSourceInput, MockSourceJob};
