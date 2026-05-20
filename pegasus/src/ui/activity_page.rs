use crate::activity::{
    load_activity_history,
    save_activity_history,
    save_activity_session,
    ActivitySession,
    ActivityState,
    ActivityType,
};

use infinitime::bt;
use std::sync::Arc;

use gtk::prelude::*;
use relm4::{
    adw, gtk, menu, Component, ComponentParts, ComponentSender, RelmWidgetExt,
};

#[derive(Debug)]
pub enum Input {
    Connected(Arc<bt::InfiniTime>),
    Disconnected,

    Select(ActivityType),
    Start,
    Stop,
    Discard,
    Tick,
    Save,
    ViewHistory(usize),
    DeleteCurrentHistory,
    SampleHeartRate,
    HeartRateSample(u8),
}

#[derive(Debug)]
pub enum Output {}

pub struct Model {
    activity_state: ActivityState,
    activity_history: Vec<ActivitySession>,
    infinitime: Option<Arc<bt::InfiniTime>>,
}

fn format_duration(duration: std::time::Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

fn actual_history_index_from_recent_index(
    history: &[ActivitySession],
    recent_index: usize,
) -> Option<usize> {
    if recent_index >= history.len() {
        return None;
    }

    Some(history.len() - 1 - recent_index)
}

fn format_activity_details(session: &ActivitySession) -> String {
    format!(
        "Type: {}\nDuration: {}\nDistance: {:.2} km\nSteps: {}\nAverage Heart Rate: {} bpm\nMax Heart Rate: {} bpm\nHeart Rate Samples: {}\nNotes: {}\n\n{}",
        session.activity_type.label(),
        format_duration(std::time::Duration::from_secs(session.duration_seconds)),
        session.distance_meters / 1000.0,
        session.steps,
        session.avg_heart_rate,
        session.max_heart_rate,
        session.heart_rate_samples.len(),
        if session.notes.is_empty() { "None" } else { &session.notes },
        format_heart_rate_samples(session)
    )
}

fn format_activity_summary(session: &ActivitySession) -> String {
    let distance_km = session.distance_meters / 1000.0;

    format!(
        "{} · {} · {:.2} km · {} steps",
        session.activity_type.label(),
        format_duration(std::time::Duration::from_secs(session.duration_seconds)),
        distance_km,
        session.steps
    )
}

fn format_heart_rate_samples(session: &ActivitySession) -> String {
    if session.heart_rate_samples.is_empty() {
        return "Heart Rate Samples:\nNone".to_string();
    }

    let samples = session
        .heart_rate_samples
        .iter()
        .map(|sample| {
            format!(
                "{}s: {} bpm",
                sample.timestamp_offset_seconds,
                sample.bpm
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!("Heart Rate Samples:\n{}", samples)
}

fn recent_activity_label(history: &[ActivitySession], index: usize) -> String {
    history
        .iter()
        .rev()
        .nth(index)
        .map(format_activity_summary)
        .unwrap_or_default()
}

fn has_recent_activity(history: &[ActivitySession], index: usize) -> bool {
    history.iter().rev().nth(index).is_some()
}

#[relm4::component(pub)]
impl Component for Model {
    type CommandOutput = ();
    type Init = ();
    type Input = Input;
    type Output = Output;
    type Widgets = Widgets;

    menu! {
        primary_menu: {
            "Health" => super::HealthViewAction,
            "PineTime Dashboard" => super::DashboardViewAction,
            "Devices" => super::DevicesViewAction,
            "Settings" => super::SettingsViewAction,
            "Quit" => super::QuitAction,
        }
    }

    view! {
        adw::ToolbarView {
            add_top_bar = &adw::HeaderBar {
                set_title_widget = Some(&adw::WindowTitle::new("Activities", "")),

                pack_end = &gtk::MenuButton {
                    set_icon_name: "open-menu-symbolic",
                    set_menu_model: Some(&primary_menu),
                },
            },

            gtk::ScrolledWindow {
                set_vexpand: true,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 12,
                    set_margin_all: 16,

                    gtk::Label {
                        #[watch]
                        set_label: &match &model.activity_state {
                            ActivityState::Idle => "Start an Activity".to_string(),
                            ActivityState::Selected(activity_type) => {
                                activity_type.label().to_string()
                            }
                            ActivityState::Active(session) => {
                                session.activity_type.label().to_string()
                            }
                            ActivityState::Completed(session) => {
                                format!("{} Complete", session.activity_type.label())
                            }
                            ActivityState::ViewingHistory(recent_index) => {
                                actual_history_index_from_recent_index(&model.activity_history, *recent_index)
                                    .and_then(|actual_index| model.activity_history.get(actual_index))
                                    .map(|session| format!("{} Details", session.activity_type.label()))
                                    .unwrap_or_else(|| "Activity Details".to_string())
                            }
                        },
                        add_css_class: "title-1",
                        set_halign: gtk::Align::Start,
                    },

                    gtk::Label {
                        #[watch]
                        set_visible: !matches!(
                            model.activity_state,
                            ActivityState::Idle | ActivityState::ViewingHistory(_)
                        ),

                        #[watch]
                        set_label: &match &model.activity_state {
                            ActivityState::Idle => "".to_string(),
                            ActivityState::Selected(_) => "00:00".to_string(),
                            ActivityState::Active(session) => {
                                format_duration(session.elapsed())
                            }
                            ActivityState::Completed(session) => {
                                format!("Duration: {}", format_duration(session.elapsed()))
                            }
                            ActivityState::ViewingHistory(_) => "".to_string(),
                        },
                        add_css_class: "title-2",
                        set_halign: gtk::Align::Start,
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 8,

                        #[watch]
                        set_visible: matches!(model.activity_state, ActivityState::Completed(_)),

                        gtk::Label {
                                set_label: "Steps",
                                set_halign: gtk::Align::Start,
                        },

                        #[name = "steps_entry"]
                        gtk::Entry {
                            set_text: "0",
                            set_input_purpose: gtk::InputPurpose::Digits,
                        },

                        gtk::Label {
                            set_label: "Distance (km)",
                            set_halign: gtk::Align::Start,
                        },

                        #[name = "distance_entry"]
                        gtk::Entry {
                            set_text: "0.00",
                            set_input_purpose: gtk::InputPurpose::Number,
                        },

                        gtk::Label {
                            set_label: "Average Heart Rate",
                            set_halign: gtk::Align::Start,
                        },

                        #[name = "avg_hr_entry"]
                        gtk::Entry {
                            set_text: "0",
                            set_input_purpose: gtk::InputPurpose::Digits,
                        },

                        gtk::Label {
                            set_label: "Max Heart Rate",
                            set_halign: gtk::Align::Start,
                        },

                        #[name = "max_hr_entry"]
                        gtk::Entry {
                            set_text: "0",
                            set_input_purpose: gtk::InputPurpose::Digits,
                        },

                        gtk::Label {
                            set_label: "Notes",
                            set_halign: gtk::Align::Start,
                        },

                        #[name = "notes_entry"]
                        gtk::Entry {
                            set_placeholder_text: Some("Optional notes"),
                        },
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 12,

                        #[watch]
                        set_visible: matches!(model.activity_state, ActivityState::Idle),

                        gtk::Button {
                            set_label: "Walk",
                            set_hexpand: true,
                            connect_clicked => Input::Select(ActivityType::Walk),
                        },

                        gtk::Button {
                            set_label: "Run",
                            set_hexpand: true,
                            connect_clicked => Input::Select(ActivityType::Run),
                        },

                        gtk::Button {
                            set_label: "Biking",
                            set_hexpand: true,
                            connect_clicked => Input::Select(ActivityType::Biking),
                        },

                        gtk::Button {
                            set_label: "Strength Training",
                            set_hexpand: true,
                            connect_clicked => Input::Select(ActivityType::StrengthTraining),
                        },

                        gtk::Button {
                            set_label: "General Workout",
                            set_hexpand: true,
                            connect_clicked => Input::Select(ActivityType::GeneralWorkout),
                        },

                        gtk::Button {
                            set_label: "Cardio",
                            set_hexpand: true,
                            connect_clicked => Input::Select(ActivityType::Cardio),
                        },
                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 8,

                        #[watch]
                        set_visible: matches!(model.activity_state, ActivityState::Idle)
                            && !model.activity_history.is_empty(),

                        gtk::Label {
                            set_label: "Recent Activities",
                            add_css_class: "title-2",
                            set_halign: gtk::Align::Start,
                        },

                        gtk::Button {
                            #[watch]
                            set_visible: has_recent_activity(&model.activity_history, 0),

                            #[watch]
                            set_label: &recent_activity_label(&model.activity_history, 0),

                            connect_clicked => Input::ViewHistory(0),

                            set_halign: gtk::Align::Start,
                        },

                        gtk::Button {
                            #[watch]
                            set_visible: has_recent_activity(&model.activity_history, 1),

                            #[watch]
                            set_label: &recent_activity_label(&model.activity_history, 1),

                            connect_clicked => Input::ViewHistory(1),

                            set_halign: gtk::Align::Start,
                        },

                        gtk::Button {
                            #[watch]
                            set_visible: has_recent_activity(&model.activity_history, 2),

                            #[watch]
                            set_label: &recent_activity_label(&model.activity_history, 2),

                            connect_clicked => Input::ViewHistory(2),

                            set_halign: gtk::Align::Start,
                        },

                        gtk::Button {
                            #[watch]
                            set_visible: has_recent_activity(&model.activity_history, 3),

                            #[watch]
                            set_label: &recent_activity_label(&model.activity_history, 3),

                            connect_clicked => Input::ViewHistory(3),

                            set_halign: gtk::Align::Start,
                        },

                        gtk::Button {
                            #[watch]
                            set_visible: has_recent_activity(&model.activity_history, 4),

                            #[watch]
                            set_label: &recent_activity_label(&model.activity_history, 4),

                            connect_clicked => Input::ViewHistory(4),

                            set_halign: gtk::Align::Start,
                        },


                    },

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 12,

                        #[watch]
                        set_visible: matches!(model.activity_state, ActivityState::ViewingHistory(_)),

                        gtk::Label {
                            #[watch]
                            set_label: &match &model.activity_state {
                                ActivityState::ViewingHistory(recent_index) => {
                                    actual_history_index_from_recent_index(&model.activity_history, *recent_index)
                                        .and_then(|actual_index| model.activity_history.get(actual_index))
                                        .map(format_activity_details)
                                        .unwrap_or_else(|| "Activity not found".to_string())
                                }
                                _ => "".to_string(),
                            },
                            set_halign: gtk::Align::Start,
                            set_wrap: true,
                            set_selectable: true,
                        },

                        gtk::Button {
                            #[watch]
                            set_visible: matches!(model.activity_state, ActivityState::ViewingHistory(_)),
                            set_label: "Delete Activity",
                            connect_clicked => Input::DeleteCurrentHistory,
                        },

                        gtk::Button {
                            set_label: "Back",
                            connect_clicked => Input::Discard,
                        },
                    },

                    gtk::Button {
                        #[watch]
                        set_visible: matches!(model.activity_state, ActivityState::Selected(_)),
                        set_label: "Start",
                        connect_clicked => Input::Start,
                    },

                    gtk::Button {
                        #[watch]
                        set_visible: matches!(model.activity_state, ActivityState::Active(_)),
                        set_label: "Stop",
                        connect_clicked => Input::Stop,
                    },

                    gtk::Button {
                        #[watch]
                        set_visible: matches!(model.activity_state, ActivityState::Completed(_)),
                        set_label: "Save Activity",
                        connect_clicked => Input::Save,
                    },

                    gtk::Button {
                        #[watch]
                        set_visible: matches!(
                            model.activity_state,
                            ActivityState::Selected(_) | ActivityState::Completed(_)
                        ),

                        #[watch]
                        set_label: match &model.activity_state {
                            ActivityState::Selected(_) => "Back",
                            ActivityState::Completed(_) => "Discard",
                                _ => "",
                        },
                        connect_clicked => Input::Discard,
                    },
                }


            }
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let activity_history = load_activity_history().unwrap_or_else(|error| {
            log::error!("Failed to load activity history: {}", error);
            Vec::new()
        });

        let model = Model {
            activity_state: ActivityState::Idle,
            activity_history,
            infinitime: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        msg: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match msg {
            Input::Select(activity_type) => {
                self.activity_state = ActivityState::Selected(activity_type);
            }
            Input::Start => {
                if let ActivityState::Selected(activity_type) = self.activity_state.clone() {
                    self.activity_state = ActivityState::Active(ActivitySession::start(activity_type));

                    let sender = _sender.clone();

                    relm4::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        sender.input(Input::Tick);
                    });

                    let hr_sender = _sender.clone();

                    relm4::spawn(async move {
                        hr_sender.input(Input::SampleHeartRate);

                        loop {
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                            hr_sender.input(Input::SampleHeartRate);
                            }
                    });
                }
            }
            Input::Stop => {
                if let ActivityState::Active(session) = self.activity_state.clone() {
                    let completed_session = session.stop();

                    widgets
                        .avg_hr_entry
                        .set_text(&completed_session.avg_heart_rate.to_string());

                    widgets
                        .max_hr_entry
                        .set_text(&completed_session.max_heart_rate.to_string());

                    widgets
                        .steps_entry
                        .set_text(&completed_session.steps.to_string());

                    widgets
                        .distance_entry
                        .set_text(&format!("{:.2}", completed_session.distance_meters / 1000.0));

                    self.activity_state = ActivityState::Completed(completed_session);
                }
            }
            Input::Discard => {
                self.activity_state = ActivityState::Idle;
            }
            Input::Tick => {
                if matches!(self.activity_state, ActivityState::Active(_)) {
                    let sender = _sender.clone();

                    relm4::spawn(async move {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        sender.input(Input::Tick);
                    });
                }
            }

            Input::SampleHeartRate => {
                if !matches!(self.activity_state, ActivityState::Active(_)) {
                    return;
                }

                let Some(infinitime) = self.infinitime.clone() else {
                    return;
                };

                let sender = _sender.clone();

                relm4::spawn(async move {
                    match infinitime.read_heart_rate().await {
                        Ok(bpm) => {
                            sender.input(Input::HeartRateSample(bpm));
                        }
                        Err(error) => {
                            log::warn!("Failed to read activity heart-rate sample: {}", error);
                        }
                    }
                });
            }

            Input::HeartRateSample(bpm) => {
                if let ActivityState::Active(session) = &mut self.activity_state {
                    session.add_heart_rate_sample(bpm);
                    log::info!("Added activity heart-rate sample: {} bpm", bpm);
                }
            }

            Input::Connected(infinitime) => {
                self.infinitime = Some(infinitime);
            }

            Input::Disconnected => {
                self.infinitime = None;
            }

            Input::ViewHistory(recent_index) => {
                if actual_history_index_from_recent_index(&self.activity_history, recent_index).is_some() {
                    self.activity_state = ActivityState::ViewingHistory(recent_index);
                }
            }
            Input::DeleteCurrentHistory => {
                let recent_index = match self.activity_state {
                    ActivityState::ViewingHistory(index) => Some(index),
                    _ => None,
                };

                if let Some(recent_index) = recent_index {
                    if let Some(actual_index) =
                        actual_history_index_from_recent_index(&self.activity_history, recent_index)
                    {
                        self.activity_history.remove(actual_index);

                        match save_activity_history(&self.activity_history) {
                            Ok(_) => {
                                log::info!("Deleted activity at history index {}", actual_index);
                            }
                            Err(error) => {
                                log::error!("Failed to save activity history after delete: {}", error);
                            }
                        }
                    }
                }

                self.activity_state = ActivityState::Idle;
            }

            Input::Save => {
                if let ActivityState::Completed(session) = &mut self.activity_state {
                    session.steps = widgets
                        .steps_entry
                        .text()
                        .parse::<u32>()
                        .unwrap_or(0);

                    session.distance_meters = widgets
                        .distance_entry
                        .text()
                        .parse::<f64>()
                        .unwrap_or(0.0)
                        * 1000.0;

                    session.avg_heart_rate = widgets
                        .avg_hr_entry
                        .text()
                        .parse::<u8>()
                        .unwrap_or(0);

                    session.max_heart_rate = widgets
                        .max_hr_entry
                        .text()
                        .parse::<u8>()
                        .unwrap_or(0);

                    session.notes = widgets.notes_entry.text().to_string();

                    match save_activity_session(session) {
                        Ok(_) => {
                            log::info!("Saved activity session: {:?}", session);
                            self.activity_history.push(session.clone());
                        }
                        Err(error) => {
                            log::error!("Failed to save activity session: {}", error);
                        }
                    }
                }

                self.activity_state = ActivityState::Idle;
            }
        }

         self.update_view(widgets, _sender);

    }
}
