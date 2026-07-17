use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingStep {
    Cv,
    Profile,
    Preferences,
    Recommendations,
    Complete,
}

impl OnboardingStep {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cv => "cv",
            Self::Profile => "profile",
            Self::Preferences => "preferences",
            Self::Recommendations => "recommendations",
            Self::Complete => "complete",
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Cv => Self::Profile,
            Self::Profile => Self::Preferences,
            Self::Preferences => Self::Recommendations,
            Self::Recommendations | Self::Complete => Self::Complete,
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "cv" => Some(Self::Cv),
            "profile" => Some(Self::Profile),
            "preferences" => Some(Self::Preferences),
            "recommendations" => Some(Self::Recommendations),
            "complete" => Some(Self::Complete),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OnboardingOutcome {
    Completed,
    Skipped,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ProfileOnboarding {
    pub profile_id: String,
    pub current_step: OnboardingStep,
    pub completed_steps: Vec<OnboardingStep>,
    pub skipped_steps: Vec<OnboardingStep>,
    pub completed_at: Option<String>,
    pub updated_at: String,
}

impl ProfileOnboarding {
    #[cfg(test)]
    pub fn new(profile_id: impl Into<String>) -> Self {
        Self {
            profile_id: profile_id.into(),
            current_step: OnboardingStep::Cv,
            completed_steps: Vec::new(),
            skipped_steps: Vec::new(),
            completed_at: None,
            updated_at: String::new(),
        }
    }

    pub fn advance(
        &mut self,
        step: OnboardingStep,
        outcome: OnboardingOutcome,
    ) -> Result<(), String> {
        if self.completed_steps.contains(&step) || self.skipped_steps.contains(&step) {
            return Ok(());
        }
        if self.current_step != step || step == OnboardingStep::Complete {
            return Err(format!(
                "onboarding step {} cannot be completed while current step is {}",
                step.as_str(),
                self.current_step.as_str()
            ));
        }

        match outcome {
            OnboardingOutcome::Completed => self.completed_steps.push(step),
            OnboardingOutcome::Skipped => self.skipped_steps.push(step),
        }
        self.current_step = step.next();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{OnboardingOutcome, OnboardingStep, ProfileOnboarding};

    #[test]
    fn advances_in_canonical_order_and_tracks_skips() {
        let mut state = ProfileOnboarding::new("profile-1");

        state
            .advance(OnboardingStep::Cv, OnboardingOutcome::Skipped)
            .expect("cv step should advance");

        assert_eq!(state.current_step, OnboardingStep::Profile);
        assert_eq!(state.skipped_steps, vec![OnboardingStep::Cv]);
    }

    #[test]
    fn rejects_out_of_order_updates() {
        let mut state = ProfileOnboarding::new("profile-1");

        let error = state
            .advance(OnboardingStep::Preferences, OnboardingOutcome::Completed)
            .expect_err("out-of-order step should fail");

        assert!(error.contains("current step is cv"));
    }
}
