use super::types::ActivityType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpsDefault {
    On,
    Off,
}

#[derive(Debug, Clone, Copy)]
pub struct ActivityProfile {
    pub activity_type: ActivityType,
    pub label: &'static str,
    pub gps_default: GpsDefault,
    pub uses_distance: bool,
    pub uses_pace: bool,
    pub uses_speed: bool,
    pub uses_steps: bool,
    pub uses_heart_rate: bool,
}

pub const ACTIVITY_PROFILES: &[ActivityProfile] = &[
    ActivityProfile {
        activity_type: ActivityType::Walk,
        label: "Walk",
        gps_default: GpsDefault::On,
        uses_distance: true,
        uses_pace: true,
        uses_speed: false,
        uses_steps: true,
        uses_heart_rate: true,
    },
    ActivityProfile {
        activity_type: ActivityType::Run,
        label: "Run",
        gps_default: GpsDefault::On,
        uses_distance: true,
        uses_pace: true,
        uses_speed: false,
        uses_steps: true,
        uses_heart_rate: true,
    },
    ActivityProfile {
        activity_type: ActivityType::Biking,
        label: "Biking",
        gps_default: GpsDefault::On,
        uses_distance: true,
        uses_pace: false,
        uses_speed: true,
        uses_steps: false,
        uses_heart_rate: true,
    },
    ActivityProfile {
        activity_type: ActivityType::StrengthTraining,
        label: "Strength Training",
        gps_default: GpsDefault::Off,
        uses_distance: false,
        uses_pace: false,
        uses_speed: false,
        uses_steps: false,
        uses_heart_rate: true,
    },
    ActivityProfile {
        activity_type: ActivityType::GeneralWorkout,
        label: "General Workout",
        gps_default: GpsDefault::Off,
        uses_distance: false,
        uses_pace: false,
        uses_speed: false,
        uses_steps: false,
        uses_heart_rate: true,
    },
    ActivityProfile {
        activity_type: ActivityType::Cardio,
        label: "Cardio",
        gps_default: GpsDefault::Off,
        uses_distance: false,
        uses_pace: false,
        uses_speed: false,
        uses_steps: false,
        uses_heart_rate: true,
    },
];

pub fn all_activity_profiles() -> &'static [ActivityProfile] {
    ACTIVITY_PROFILES
}
