use crate::domain::application::model::Application;

use super::signals::EventSignals;
use super::types::{OutcomeLabel, OutcomeSignals};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct OutcomeLabelAssignment {
    pub(super) label: OutcomeLabel,
    pub(super) label_score: u8,
    pub(super) reasons: Vec<String>,
}

pub(super) fn resolve_label_observed_at(
    signals: &OutcomeSignals,
    event_signals: &EventSignals,
    application: Option<&Application>,
    feedback_updated_at: Option<&str>,
) -> Option<String> {
    if signals.applied {
        if let Some(record) = application {
            if let Some(outcome_date) = record.outcome_date.as_ref() {
                return Some(outcome_date.clone());
            }
            return Some(record.updated_at.clone());
        }
    }

    if let Some(created_at) = event_signals.latest_event_at.as_ref() {
        return Some(created_at.clone());
    }

    feedback_updated_at.map(str::to_string)
}

pub(super) fn assign_label(signals: &OutcomeSignals) -> Option<OutcomeLabelAssignment> {
    if signals.legitimacy_spam || signals.legitimacy_suspicious {
        let mut reasons = vec!["dismissed".to_string(), "suspicious_posting".to_string()];
        if signals.legitimacy_spam {
            reasons.push("spam".to_string());
        }
        return Some(OutcomeLabelAssignment {
            label: OutcomeLabel::Negative,
            label_score: 0,
            reasons,
        });
    }

    if signals.work_mode_deal_breaker && !signals.applied {
        let mut reasons = vec![
            "dismissed".to_string(),
            "work_mode_deal_breaker".to_string(),
        ];
        if signals.dismissed {
            if signals.bad_fit {
                reasons.push("bad_fit".to_string());
            }
            if signals.hidden {
                reasons.push("hidden".to_string());
            }
        }
        return Some(OutcomeLabelAssignment {
            label: OutcomeLabel::Negative,
            label_score: 0,
            reasons,
        });
    }

    if signals.applied {
        let mut reasons = vec!["applied".to_string()];
        if signals.received_offer {
            reasons.push("offer_received".to_string());
        } else if signals.reached_interview {
            reasons.push("reached_interview".to_string());
        } else if signals.was_rejected {
            reasons.push("outcome_rejected".to_string());
        } else if signals.was_ghosted {
            reasons.push("outcome_ghosted".to_string());
        }
        return Some(OutcomeLabelAssignment {
            label: OutcomeLabel::Positive,
            label_score: 2,
            reasons,
        });
    }

    if signals.dismissed {
        let mut reasons = vec!["dismissed".to_string()];
        if signals.bad_fit {
            reasons.push("bad_fit".to_string());
        }
        if signals.hidden {
            reasons.push("hidden".to_string());
        }
        if signals.has_salary_rejection {
            reasons.push("salary_too_low".to_string());
        }
        if signals.salary_below_expectation && !reasons.contains(&"salary_too_low".to_string()) {
            reasons.push("salary_too_low".to_string());
        }

        return Some(OutcomeLabelAssignment {
            label: OutcomeLabel::Negative,
            label_score: 0,
            reasons,
        });
    }

    if signals.saved {
        let mut reasons = vec!["saved".to_string()];
        if signals.interest_rating == Some(2) {
            reasons.push("love_it".to_string());
        }
        return Some(OutcomeLabelAssignment {
            label: OutcomeLabel::Medium,
            label_score: 1,
            reasons,
        });
    }

    if signals.viewed {
        let mut reasons = vec!["viewed".to_string()];
        if signals.returned_count >= 2 {
            reasons.push("high_engagement".to_string());
        }
        return Some(OutcomeLabelAssignment {
            label: OutcomeLabel::Medium,
            label_score: 1,
            reasons,
        });
    }

    None
}
