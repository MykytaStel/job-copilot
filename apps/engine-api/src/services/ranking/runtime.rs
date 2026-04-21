#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RerankerRuntimeMode {
    Deterministic,
    Learned,
    Trained,
}

impl RerankerRuntimeMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "deterministic" => Some(Self::Deterministic),
            "learned" => Some(Self::Learned),
            "trained" => Some(Self::Trained),
            _ => None,
        }
    }

    pub fn default_from_flags(learned_enabled: bool, trained_enabled: bool) -> Self {
        if trained_enabled {
            Self::Trained
        } else if learned_enabled {
            Self::Learned
        } else {
            Self::Deterministic
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Deterministic => "deterministic",
            Self::Learned => "learned",
            Self::Trained => "trained",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TrainedRerankerAvailability {
    DisabledByFlag,
    Ready,
    MissingPath,
    InvalidArtifact(String),
}

impl TrainedRerankerAvailability {
    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DisabledByFlag => "disabled_by_flag",
            Self::Ready => "ready",
            Self::MissingPath => "missing_path",
            Self::InvalidArtifact(_) => "invalid_artifact",
        }
    }

    fn unavailable_detail(&self) -> Option<String> {
        match self {
            Self::DisabledByFlag => Some("TRAINED_RERANKER_ENABLED is false".to_string()),
            Self::Ready => None,
            Self::MissingPath => Some("TRAINED_RERANKER_MODEL_PATH is empty".to_string()),
            Self::InvalidArtifact(error) => Some(format!(
                "trained artifact is invalid or unreadable: {error}"
            )),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedRerankerRuntime {
    pub requested_mode: RerankerRuntimeMode,
    pub active_mode: RerankerRuntimeMode,
    pub fallback_reason: Option<String>,
    pub apply_learned: bool,
    pub apply_trained: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedRerankerRuntimeComparison {
    pub baseline: ResolvedRerankerRuntime,
    pub learned: ResolvedRerankerRuntime,
    pub trained: ResolvedRerankerRuntime,
}

pub fn resolve_reranker_runtime(
    requested_mode: RerankerRuntimeMode,
    learned_enabled: bool,
    learned_available: bool,
    trained_availability: &TrainedRerankerAvailability,
) -> ResolvedRerankerRuntime {
    match requested_mode {
        RerankerRuntimeMode::Deterministic => ResolvedRerankerRuntime {
            requested_mode,
            active_mode: RerankerRuntimeMode::Deterministic,
            fallback_reason: None,
            apply_learned: false,
            apply_trained: false,
        },
        RerankerRuntimeMode::Learned => {
            if learned_enabled && learned_available {
                return ResolvedRerankerRuntime {
                    requested_mode,
                    active_mode: RerankerRuntimeMode::Learned,
                    fallback_reason: None,
                    apply_learned: true,
                    apply_trained: false,
                };
            }

            let fallback_reason = if !learned_enabled {
                "Learned reranker unavailable: LEARNED_RERANKER_ENABLED is false; kept deterministic ranking"
                    .to_string()
            } else {
                "Learned reranker unavailable: profile learning aggregates were unavailable; kept deterministic ranking"
                    .to_string()
            };

            ResolvedRerankerRuntime {
                requested_mode,
                active_mode: RerankerRuntimeMode::Deterministic,
                fallback_reason: Some(fallback_reason),
                apply_learned: false,
                apply_trained: false,
            }
        }
        RerankerRuntimeMode::Trained => {
            if trained_availability.is_ready() {
                return ResolvedRerankerRuntime {
                    requested_mode,
                    active_mode: RerankerRuntimeMode::Trained,
                    fallback_reason: None,
                    apply_learned: learned_enabled && learned_available,
                    apply_trained: true,
                };
            }

            let trained_reason = trained_availability
                .unavailable_detail()
                .unwrap_or_else(|| "trained reranker is unavailable".to_string());

            if learned_enabled && learned_available {
                return ResolvedRerankerRuntime {
                    requested_mode,
                    active_mode: RerankerRuntimeMode::Learned,
                    fallback_reason: Some(format!(
                        "Trained reranker unavailable: {trained_reason}; fell back to learned reranker"
                    )),
                    apply_learned: true,
                    apply_trained: false,
                };
            }

            let learned_reason = if !learned_enabled {
                "LEARNED_RERANKER_ENABLED is false"
            } else {
                "profile learning aggregates were unavailable"
            };

            ResolvedRerankerRuntime {
                requested_mode,
                active_mode: RerankerRuntimeMode::Deterministic,
                fallback_reason: Some(format!(
                    "Trained reranker unavailable: {trained_reason}; kept deterministic ranking because {learned_reason}"
                )),
                apply_learned: false,
                apply_trained: false,
            }
        }
    }
}

pub fn resolve_reranker_runtime_comparison(
    learned_enabled: bool,
    learned_available: bool,
    trained_availability: &TrainedRerankerAvailability,
) -> ResolvedRerankerRuntimeComparison {
    ResolvedRerankerRuntimeComparison {
        baseline: resolve_reranker_runtime(
            RerankerRuntimeMode::Deterministic,
            learned_enabled,
            learned_available,
            trained_availability,
        ),
        learned: resolve_reranker_runtime(
            RerankerRuntimeMode::Learned,
            learned_enabled,
            learned_available,
            trained_availability,
        ),
        trained: resolve_reranker_runtime(
            RerankerRuntimeMode::Trained,
            learned_enabled,
            learned_available,
            trained_availability,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        RerankerRuntimeMode, TrainedRerankerAvailability, resolve_reranker_runtime,
        resolve_reranker_runtime_comparison,
    };

    #[test]
    fn defaults_runtime_mode_from_feature_flags() {
        assert_eq!(
            RerankerRuntimeMode::default_from_flags(false, false),
            RerankerRuntimeMode::Deterministic
        );
        assert_eq!(
            RerankerRuntimeMode::default_from_flags(true, false),
            RerankerRuntimeMode::Learned
        );
        assert_eq!(
            RerankerRuntimeMode::default_from_flags(true, true),
            RerankerRuntimeMode::Trained
        );
    }

    #[test]
    fn trained_mode_uses_ready_artifact() {
        let resolved = resolve_reranker_runtime(
            RerankerRuntimeMode::Trained,
            true,
            true,
            &TrainedRerankerAvailability::Ready,
        );

        assert_eq!(resolved.active_mode, RerankerRuntimeMode::Trained);
        assert!(resolved.apply_learned);
        assert!(resolved.apply_trained);
        assert_eq!(resolved.fallback_reason, None);
    }

    #[test]
    fn trained_mode_falls_back_to_learned_when_artifact_is_missing() {
        let resolved = resolve_reranker_runtime(
            RerankerRuntimeMode::Trained,
            true,
            true,
            &TrainedRerankerAvailability::MissingPath,
        );

        assert_eq!(resolved.active_mode, RerankerRuntimeMode::Learned);
        assert!(resolved.apply_learned);
        assert!(!resolved.apply_trained);
        assert!(
            resolved
                .fallback_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("fell back to learned reranker"))
        );
    }

    #[test]
    fn trained_mode_falls_back_to_deterministic_when_no_safe_layer_is_available() {
        let resolved = resolve_reranker_runtime(
            RerankerRuntimeMode::Trained,
            false,
            false,
            &TrainedRerankerAvailability::InvalidArtifact("bad json".to_string()),
        );

        assert_eq!(resolved.active_mode, RerankerRuntimeMode::Deterministic);
        assert!(!resolved.apply_learned);
        assert!(!resolved.apply_trained);
        assert!(
            resolved
                .fallback_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("kept deterministic ranking"))
        );
    }

    #[test]
    fn learned_mode_falls_back_to_deterministic_when_aggregates_are_missing() {
        let resolved = resolve_reranker_runtime(
            RerankerRuntimeMode::Learned,
            true,
            false,
            &TrainedRerankerAvailability::DisabledByFlag,
        );

        assert_eq!(resolved.active_mode, RerankerRuntimeMode::Deterministic);
        assert!(!resolved.apply_learned);
        assert!(!resolved.apply_trained);
        assert!(
            resolved
                .fallback_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("learning aggregates were unavailable"))
        );
    }

    #[test]
    fn availability_serializes_to_stable_status_values() {
        assert_eq!(
            TrainedRerankerAvailability::DisabledByFlag.as_str(),
            "disabled_by_flag"
        );
        assert_eq!(TrainedRerankerAvailability::Ready.as_str(), "ready");
        assert_eq!(
            TrainedRerankerAvailability::MissingPath.as_str(),
            "missing_path"
        );
        assert_eq!(
            TrainedRerankerAvailability::InvalidArtifact("bad json".to_string()).as_str(),
            "invalid_artifact"
        );
    }

    #[test]
    fn comparison_resolution_reuses_safe_fallbacks() {
        let comparison = resolve_reranker_runtime_comparison(
            true,
            true,
            &TrainedRerankerAvailability::MissingPath,
        );

        assert_eq!(
            comparison.baseline.active_mode,
            RerankerRuntimeMode::Deterministic
        );
        assert_eq!(comparison.learned.active_mode, RerankerRuntimeMode::Learned);
        assert_eq!(comparison.trained.active_mode, RerankerRuntimeMode::Learned);
        assert!(
            comparison
                .trained
                .fallback_reason
                .as_deref()
                .is_some_and(|reason| reason.contains("fell back to learned reranker"))
        );
    }
}
