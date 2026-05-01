use crate::domain::job::presentation::JobTextQuality;
use crate::domain::role::RoleId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RerankerMode {
    Deterministic,
    Learned,
    Trained,
    Fallback,
}

impl RerankerMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Deterministic => "deterministic",
            Self::Learned => "learned",
            Self::Trained => "trained",
            Self::Fallback => "fallback",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobScorePenalty {
    pub kind: String,
    pub score_delta: i16,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobScoreBreakdown {
    pub total_score: u8,
    pub matching_score: i16,
    pub salary_score: i16,
    pub reranker_score: i16,
    pub freshness_score: i16,
    pub penalties: Vec<JobScorePenalty>,
    pub reranker_mode: RerankerMode,
}

impl JobScoreBreakdown {
    pub fn new(
        matching_score: i16,
        salary_score: i16,
        freshness_score: i16,
        penalties: Vec<JobScorePenalty>,
    ) -> Self {
        let mut breakdown = Self {
            total_score: 0,
            matching_score,
            salary_score,
            reranker_score: 0,
            freshness_score,
            penalties,
            reranker_mode: RerankerMode::Deterministic,
        };
        breakdown.refresh_total();
        breakdown
    }

    #[cfg(test)]
    pub fn deterministic(total_score: u8) -> Self {
        Self {
            total_score,
            matching_score: i16::from(total_score),
            salary_score: 0,
            reranker_score: 0,
            freshness_score: 0,
            penalties: Vec::new(),
            reranker_mode: RerankerMode::Deterministic,
        }
    }

    fn refresh_total(&mut self) {
        let penalties = self
            .penalties
            .iter()
            .map(|penalty| i32::from(penalty.score_delta))
            .sum::<i32>();
        let total = i32::from(self.matching_score)
            + i32::from(self.salary_score)
            + i32::from(self.reranker_score)
            + i32::from(self.freshness_score)
            + penalties;

        self.total_score = total.clamp(0, 100) as u8;
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissingSignalDetail {
    pub term: String,
    pub category: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobFit {
    pub job_id: String,
    pub score: u8,
    pub score_breakdown: JobScoreBreakdown,
    pub matched_roles: Vec<RoleId>,
    pub matched_skills: Vec<String>,
    pub matched_keywords: Vec<String>,
    pub source_match: bool,
    pub work_mode_match: Option<bool>,
    pub region_match: Option<bool>,
    pub missing_signals: Vec<String>,
    pub missing_signal_details: Vec<MissingSignalDetail>,
    pub description_quality: JobTextQuality,
    pub reasons: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_score_clamps_at_zero_when_components_sum_negative() {
        let breakdown = JobScoreBreakdown::new(0, 0, -15, vec![]);
        assert_eq!(breakdown.total_score, 0, "negative component sum must clamp to 0");
    }

    #[test]
    fn total_score_does_not_exceed_100() {
        let breakdown = JobScoreBreakdown::new(200, 0, 0, vec![]);
        assert_eq!(breakdown.total_score, 100, "component sum above 100 must clamp to 100");
    }
}

impl JobFit {
    pub fn apply_matching_adjustment(&mut self, score_delta: i16, reason: Option<String>) {
        if score_delta == 0 {
            return;
        }

        self.score_breakdown.matching_score += score_delta;
        self.sync_score();

        if let Some(reason) = reason {
            self.reasons.push(reason);
        }
    }

    pub fn apply_salary_score(&mut self, score_delta: i16, reason: Option<String>) {
        if score_delta == 0 {
            return;
        }

        self.score_breakdown.salary_score += score_delta;
        self.sync_score();

        if let Some(reason) = reason {
            self.reasons.push(reason);
        }
    }

    pub fn add_penalty(
        &mut self,
        kind: impl Into<String>,
        score_delta: i16,
        reason: impl Into<String>,
    ) {
        if score_delta >= 0 {
            return;
        }

        let reason = reason.into();
        self.score_breakdown.penalties.push(JobScorePenalty {
            kind: kind.into(),
            score_delta,
            reason: reason.clone(),
        });
        self.sync_score();
        self.reasons.push(reason);
    }

    pub fn apply_reranker_score(
        &mut self,
        score_delta: i16,
        reranker_mode: RerankerMode,
        reasons: Vec<String>,
    ) {
        if !matches!(reranker_mode, RerankerMode::Deterministic) {
            self.score_breakdown.reranker_mode = reranker_mode;
        }

        if score_delta != 0 {
            self.score_breakdown.reranker_score += score_delta;
            self.sync_score();
        }

        self.reasons.extend(reasons);
    }

    pub fn mark_reranker_fallback(&mut self, reason: impl Into<String>) {
        self.score_breakdown.reranker_mode = RerankerMode::Fallback;
        self.reasons.push(reason.into());
    }

    fn sync_score(&mut self) {
        self.score_breakdown.refresh_total();
        self.score = self.score_breakdown.total_score;
    }
}
