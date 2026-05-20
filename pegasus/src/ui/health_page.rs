use crate::health::{
    add_daily_heart_rate_sample,
    load_daily_heart_rate_samples,
    summarize_daily_heart_rate,
    DailyHeartRateSample,
};

use gtk::prelude::*;
use relm4::{
    adw, gtk, menu, Component, ComponentParts, ComponentSender, RelmWidgetExt,
};

#[derive(Debug)]
pub enum Input {
    AddTestSample,
}

#[derive(Debug)]
pub enum Output {}

pub struct Model {
    samples: Vec<DailyHeartRateSample>,
}

fn summary_text(samples: &[DailyHeartRateSample]) -> String {
    let summary = summarize_daily_heart_rate(samples);

    format!(
        "Samples: {}\nAverage: {} bpm\nMin: {} bpm\nMax: {} bpm",
        summary.sample_count,
        summary.average_bpm,
        summary.min_bpm,
        summary.max_bpm
    )
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
            "Activities" => super::ActivitiesViewAction,
            "PineTime Dashboard" => super::DashboardViewAction,
            "Devices" => super::DevicesViewAction,
            "Settings" => super::SettingsViewAction,
            "Quit" => super::QuitAction,
        }
    }

    view! {
        adw::ToolbarView {
            add_top_bar = &adw::HeaderBar {
                set_title_widget = Some(&adw::WindowTitle::new("Health", "")),

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
                        set_label: "Daily Heart Rate",
                        add_css_class: "title-1",
                        set_halign: gtk::Align::Start,
                    },

                    gtk::Label {
                        #[watch]
                        set_label: &summary_text(&model.samples),
                        set_halign: gtk::Align::Start,
                        set_wrap: true,
                        set_selectable: true,
                    },

                    gtk::Button {
                        set_label: "Add Test Sample",
                        connect_clicked => Input::AddTestSample,
                    },

                    gtk::Label {
                        set_label: "This button is temporary. Later, Pegasus will add samples automatically every 5 minutes while the PineTime is connected.",
                        set_halign: gtk::Align::Start,
                        set_wrap: true,
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
        let samples = load_daily_heart_rate_samples().unwrap_or_else(|error| {
            log::error!("Failed to load daily heart-rate samples: {}", error);
            Vec::new()
        });

        let model = Model { samples };
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
            Input::AddTestSample => {
                let bpm = 72 + (self.samples.len() as u8 % 40);

                match add_daily_heart_rate_sample(bpm) {
                    Ok(sample) => {
                        log::info!("Added test heart-rate sample: {:?}", sample);
                        self.samples.push(sample);
                    }
                    Err(error) => {
                        log::error!("Failed to add test heart-rate sample: {}", error);
                    }
                }
            }
        }

        self.update_view(widgets, _sender);
    }
}
