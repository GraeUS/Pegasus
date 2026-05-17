use crate::activity::{ActivitySession, ActivityState, ActivityType};

use gtk::prelude::*;
use relm4::{
    adw, gtk, menu, Component, ComponentParts, ComponentSender, RelmWidgetExt,
};

#[derive(Debug)]
pub enum Input {
    Select(ActivityType),
    Start,
    Stop,
    Discard,
    Tick,
    Save,
}

#[derive(Debug)]
pub enum Output {}

pub struct Model {
    activity_state: ActivityState,
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

#[relm4::component(pub)]
impl Component for Model {
    type CommandOutput = ();
    type Init = ();
    type Input = Input;
    type Output = Output;
    type Widgets = Widgets;

    menu! {
        primary_menu: {
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
                        },
                        add_css_class: "title-1",
                        set_halign: gtk::Align::Start,
                    },

                    gtk::Label {
                        #[watch]
                        set_visible: !matches!(model.activity_state, ActivityState::Idle),

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
        let model = Model {
            activity_state: ActivityState::Idle,
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
                }
            }
            Input::Stop => {
                if let ActivityState::Active(session) = self.activity_state.clone() {
                    self.activity_state = ActivityState::Completed(session.stop());
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

                    log::info!("Saving activity session: {:?}", session);
                }

                self.activity_state = ActivityState::Idle;
            }
        }

         self.update_view(widgets, _sender);

    }
}
