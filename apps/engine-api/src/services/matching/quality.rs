use std::collections::BTreeMap;

use crate::domain::job::presentation::JobTextQuality;

use super::{MatchQualitySummary, RankedJob};

pub(crate) fn summarize_match_quality(ranked_jobs: &[RankedJob]) -> MatchQualitySummary {
    let mut low_evidence_jobs = 0usize;
    let mut weak_description_jobs = 0usize;
    let mut role_mismatch_jobs = 0usize;
    let mut seniority_mismatch_jobs = 0usize;
    let mut source_mismatch_jobs = 0usize;
    let mut missing_counts = BTreeMap::<String, usize>::new();

    for ranked in ranked_jobs {
        let fit = &ranked.fit;

        if fit.matched_roles.is_empty()
            && fit.matched_skills.is_empty()
            && fit.matched_keywords.is_empty()
        {
            low_evidence_jobs += 1;
        }

        if matches!(fit.description_quality, JobTextQuality::Weak) {
            weak_description_jobs += 1;
        }

        if fit
            .reasons
            .iter()
            .any(|reason| reason.contains("Role mismatch penalty applied"))
        {
            role_mismatch_jobs += 1;
        }

        if fit
            .reasons
            .iter()
            .any(|reason| reason.contains("Seniority mismatch penalty applied"))
        {
            seniority_mismatch_jobs += 1;
        }

        if !fit.source_match {
            source_mismatch_jobs += 1;
        }

        for signal in &fit.missing_signals {
            *missing_counts.entry(signal.clone()).or_insert(0usize) += 1;
        }
    }

    let mut top_missing_signals = missing_counts.into_iter().collect::<Vec<_>>();
    top_missing_signals
        .sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    MatchQualitySummary {
        low_evidence_jobs,
        weak_description_jobs,
        role_mismatch_jobs,
        seniority_mismatch_jobs,
        source_mismatch_jobs,
        top_missing_signals: top_missing_signals
            .into_iter()
            .take(8)
            .map(|(signal, _)| signal)
            .collect(),
    }
}
