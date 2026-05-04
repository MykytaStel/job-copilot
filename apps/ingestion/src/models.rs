#[allow(unused_imports)]
pub use job_copilot_domain::job::{
    CompanyMeta, IngestionBatch, IngestionInput, InputDocument, JobVariant, NormalizationResult,
    NormalizedJob, RawSnapshot, canonical_job_id, compute_dedupe_key,
};

#[cfg(any(feature = "mock", test))]
pub use job_copilot_domain::job::{MockCompensation, MockSourceInput, MockSourceJob};
