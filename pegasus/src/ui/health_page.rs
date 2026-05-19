use gtk::prelude::*;
use relm4::{
    adw, gtk, menu, Component, ComponentParts, ComponentSender, RelmWidgetExt,
};

#[derive(Debug)]
pub enum Input {}

#[derive(Debug)]
pub enum Output {}

pub struct Model;

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
                        set_label: "Background sampling is not active yet.",
                        set_halign: gtk::Align::Start,
                        set_wrap: true,
                    },

                    gtk::Label {
                        set_label: "Eventually, Pegasus will sample heart rate every 5 minutes while the PineTime is connected, then show daily average, max, min, and trends here.",
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
        let model = Model;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }
}
