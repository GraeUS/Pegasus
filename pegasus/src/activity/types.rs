use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    Walk,
    Run,
    Biking,
    StrengthTraining,
    GeneralWorkout,
    Cardio,
}

impl ActivityType {
    pub fn label(&self) -> &'static str {
        match self {
            ActivityType::Walk => "Walk",
            ActivityType::Run => "Run",
            ActivityType::Biking => "Biking",
            ActivityType::StrengthTraining => "Strength Training",
            ActivityType::GeneralWorkout => "General Workout",
            ActivityType::Cardio => "Cardio",
        }
    }
}
