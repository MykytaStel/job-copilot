use crate::domain::candidate::profile::{CandidateProfile, RoleScore};
use crate::domain::role::RoleId;

use super::matching::PreparedText;
use super::presentation::{build_search_terms, build_summary};
use super::rules::{
    CombinationBonusRule, KNOWN_KEYWORDS, KNOWN_SKILLS, MAX_ROLE_CANDIDATES, MIN_ROLE_SCORE,
    ROLE_RULES, RoleRule, SignalGroup,
};

#[derive(Clone, Default)]
pub struct ProfileAnalysisService;

impl ProfileAnalysisService {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, raw_text: &str) -> CandidateProfile {
        let prepared_text = PreparedText::new(raw_text);

        let skills = self.extract_skills(&prepared_text);
        let keywords = self.extract_keywords(&prepared_text);
        let seniority = self.detect_seniority(&prepared_text);
        let role_candidates = self.score_roles(&prepared_text);

        let primary_role = role_candidates
            .first()
            .map(|candidate| candidate.role.clone())
            .unwrap_or_else(RoleId::fallback);

        let suggested_search_terms = build_search_terms(&role_candidates, &skills, &seniority);
        let summary = build_summary(primary_role, &seniority, &role_candidates, &skills);

        CandidateProfile {
            summary,
            primary_role,
            seniority,
            skills,
            keywords,
            role_candidates,
            suggested_search_terms,
        }
    }

    fn extract_skills(&self, text: &PreparedText) -> Vec<String> {
        let mut result = Vec::new();

        for skill in KNOWN_SKILLS {
            if text.matches_signal(skill) {
                push_unique(&mut result, (*skill).to_string());
            }
        }

        result
    }

    fn extract_keywords(&self, text: &PreparedText) -> Vec<String> {
        let mut result = Vec::new();

        for keyword in KNOWN_KEYWORDS {
            if text.matches_signal(keyword) {
                push_unique(&mut result, (*keyword).to_string());
            }
        }

        result
    }

    fn detect_seniority(&self, text: &PreparedText) -> String {
        if text.matches_signal("principal") {
            "principal".to_string()
        } else if text.matches_signal("staff") {
            "staff".to_string()
        } else if text.matches_signal("lead") {
            "lead".to_string()
        } else if text.matches_signal("senior") {
            "senior".to_string()
        } else if text.matches_signal("middle")
            || text.matches_signal("mid-level")
            || text.matches_signal("mid")
        {
            "middle".to_string()
        } else if text.matches_signal("junior") {
            "junior".to_string()
        } else {
            "unknown".to_string()
        }
    }

    fn score_roles(&self, text: &PreparedText) -> Vec<RoleScore> {
        let mut candidates: Vec<RoleScore> = ROLE_RULES
            .iter()
            .map(|rule| self.score_role(rule, text))
            .filter(|candidate| candidate.score >= MIN_ROLE_SCORE)
            .collect();

        candidates.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| right.confidence.cmp(&left.confidence))
                .then_with(|| left.role.canonical_key().cmp(right.role.canonical_key()))
        });

        candidates.truncate(MAX_ROLE_CANDIDATES);

        if candidates.is_empty() {
            return vec![self.generalist_candidate()];
        }

        candidates
    }

    fn score_role(&self, rule: &RoleRule, text: &PreparedText) -> RoleScore {
        let mut score = 0;
        let mut matched_signals = Vec::new();

        for (signal, weight) in rule.signals {
            if text.matches_signal(signal) {
                score += *weight;
                push_unique(&mut matched_signals, (*signal).to_string());
            }
        }

        score += self.apply_combination_bonus(rule, text, &mut matched_signals);

        let confidence = self.calculate_confidence(score, self.strongest_possible_score(rule));

        RoleScore {
            role: rule.role,
            score,
            confidence,
            matched_signals,
        }
    }

    fn apply_combination_bonus(
        &self,
        rule: &RoleRule,
        text: &PreparedText,
        matched_signals: &mut Vec<String>,
    ) -> u32 {
        let mut bonus_score = 0;

        for bonus_rule in rule.combination_bonuses {
            if self.matches_combination_bonus(bonus_rule, text) {
                bonus_score += bonus_rule.bonus;
                push_unique(matched_signals, bonus_rule.label.to_string());
            }
        }

        bonus_score
    }

    fn matches_combination_bonus(
        &self,
        bonus_rule: &CombinationBonusRule,
        text: &PreparedText,
    ) -> bool {
        bonus_rule
            .required_groups
            .iter()
            .all(|group| self.count_group_matches(group, text) >= group.min_matches)
    }

    fn count_group_matches(&self, group: &SignalGroup, text: &PreparedText) -> usize {
        group
            .signals
            .iter()
            .filter(|signal| text.matches_signal(signal))
            .count()
    }

    fn calculate_confidence(&self, score: u32, strongest_possible_score: u32) -> u8 {
        if score == 0 || strongest_possible_score == 0 {
            return 0;
        }

        let scaled = ((score * 100) + (strongest_possible_score / 2)) / strongest_possible_score;
        scaled.min(100) as u8
    }

    fn strongest_possible_score(&self, rule: &RoleRule) -> u32 {
        let signal_score = rule.signals.iter().map(|(_, weight)| *weight).sum::<u32>();
        let bonus_score = rule
            .combination_bonuses
            .iter()
            .map(|bonus_rule| bonus_rule.bonus)
            .sum::<u32>();

        signal_score + bonus_score
    }

    fn generalist_candidate(&self) -> RoleScore {
        RoleScore {
            role: RoleId::fallback(),
            score: 0,
            confidence: 0,
            matched_signals: vec![],
        }
    }
}

fn push_unique(target: &mut Vec<String>, value: String) {
    if !target.iter().any(|existing| existing == &value) {
        target.push(value);
    }
}
