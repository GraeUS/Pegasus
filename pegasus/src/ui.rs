use infinitime::{bluer, bt};
use std::{sync::Arc, path::PathBuf, env};
use futures::{pin_mut, StreamExt};
use gtk::{gio, glib, prelude::{ApplicationExt, BoxExt, GtkWindowExt, SettingsExt, WidgetExt}};
use relm4::{
    adw, gtk, actions::{AccelsPlus, RelmAction, RelmActionGroup},
    Component, ComponentController, ComponentParts,
    ComponentSender, Controller, RelmApp, MessageBroker
};
use tokio::time::{sleep, timeout, Duration};

mod dashboard_page;
mod devices_page;
mod fwupd_page;
mod settings_page;
mod activity_page;
mod icon_names {
    include!(concat!(env!("OUT_DIR"), "/icon_names.rs"));
}


static APP_ID: &'static str = "io.github.graeus.Pegasus";
static SETTING_NOTIFICATIONS: &'static str = "forward-notifications";
static SETTING_BACKGROUND: &'static str = "run-in-background";
static SETTING_AUTO_START: &'static str = "auto-start";
static SETTING_DEVICE_ADDRESS: &'static str = "auto-connect-address";

static BROKER: relm4::MessageBroker<Input> = MessageBroker::new();


relm4::new_action_group!(ViewActionGroup, "view");
relm4::new_stateless_action!(DashboardViewAction, ViewActionGroup, "dashboard");
relm4::new_stateless_action!(DevicesViewAction, ViewActionGroup, "devices");
relm4::new_stateless_action!(ActivitiesViewAction, ViewActionGroup, "activities");
relm4::new_stateless_action!(SettingsViewAction, ViewActionGroup, "settings");
relm4::new_stateless_action!(AboutAction, ViewActionGroup, "about");
relm4::new_action_group!(WindowActionGroup, "win");
relm4::new_stateless_action!(CloseAction, WindowActionGroup, "close");
relm4::new_stateless_action!(QuitAction, WindowActionGroup, "quit");


#[derive(Debug)]
enum Input {
    SetView(View),
    DeviceConnected(Arc<bluer::Device>),
    DeviceDisconnected,
    DeviceReady(Arc<bt::InfiniTime>),
    DeviceRejected,
    FlashAssetFromFile(PathBuf, fwupd_page::AssetType),
    FlashAssetFromUrl(String, fwupd_page::AssetType),
    Toast(String),
    ToastStatic(&'static str),
    DeviceConnectionLost(bluer::Address),
    ToastWithLink {
        message: &'static str,
        label: &'static str,
        url: &'static str,
    },
    WindowShown, // Temporary hack
    About,
    Close,
    Quit,
}

struct Model {
    // UI state
    active_view: View,
    is_connected: bool,
    // Components
    dashboard_page: Controller<dashboard_page::Model>,
    devices_page: Controller<devices_page::Model>,
    fwupd_page: Controller<fwupd_page::Model>,
    settings_page: Controller<settings_page::Model>,
    activity_page: Controller<activity_page::Model>,
    // Other
    infinitime: Option<Arc<bt::InfiniTime>>,
    toast_overlay: adw::ToastOverlay,
    hide_on_startup: bool,  // Temporary hack
}

async fn wait_for_services_resolved(
    device: Arc<bluer::Device>,
    max_wait: Duration,
) -> bluer::Result<bool> {
    if device.is_connected().await? && device.is_services_resolved().await? {
        return Ok(true);
    }

    let stream = device.events().await?;
    pin_mut!(stream);

    let wait = async {
        while let Some(event) = stream.next().await {
            if let bluer::DeviceEvent::PropertyChanged(property) = event {
                log::debug!("Device property changed while waiting for GATT: {:?}", property);

                if let bluer::DeviceProperty::ServicesResolved(true) = property {
                    return true;
                }

                if let bluer::DeviceProperty::Connected(false) = property {
                    log::warn!("Device disconnected before GATT services resolved");
                    return false;
                }
            }
        }

        false
    };

    Ok(timeout(max_wait, wait).await.unwrap_or(false))
}

#[relm4::component]
impl Component for Model {
    type CommandOutput = ();
    type Init = bool;
    type Input = Input;
    type Output = ();
    type Widgets = Widgets;

    view! {
        #[name = "main_window"]
        adw::ApplicationWindow {
            set_default_width: 480,
            set_default_height: 720,
            set_hide_on_close: settings.boolean(SETTING_BACKGROUND),

            // Temporary hack
            connect_show => Input::WindowShown,

            #[local]
            toast_overlay -> adw::ToastOverlay {
                // TODO: Use Relm 0.5 conditional widgets here (automatic stack)
                // I can't make it work here for some reason for now.
                #[wrap(Some)]
                set_child = &gtk::Stack {
                    add_named[Some("dashboard_view")] = &gtk::Box {
                        // set_visible: watch!(components.dashboard.model.device.is_some()),
                        append: model.dashboard_page.widget(),
                    },
                    add_named[Some("devices_view")] = &gtk::Box {
                        append: model.devices_page.widget(),
                    },
                    add_named[Some("fwupd_view")] = &gtk::Box {
                        append: model.fwupd_page.widget(),
                    },
                    add_named[Some("settings_view")] = &gtk::Box {
                        append: model.settings_page.widget(),
                    },
                    add_named[Some("activities_view")] = &gtk::Box {
                        append: model.activity_page.widget(),
                    },
                    #[watch]
                    set_visible_child_name: match model.active_view {
                        View::Dashboard => "dashboard_view",
                        View::Devices => "devices_view",
                        View::FirmwareUpdate => "fwupd_view",
                        View::Settings => "settings_view",
                        View::Activities => "activities_view",
                    },
                },
            },
        }
    }

    fn init(start_in_background: Self::Init, root: Self::Root, sender: ComponentSender<Self>) -> ComponentParts<Self> {
        let settings = gio::Settings::new(APP_ID);

        // Components
        let dashboard_page = dashboard_page::Model::builder()
            .launch((root.clone(), settings.clone()))
            .forward(&sender.input_sender(), |message| match message {
                dashboard_page::Output::FlashAssetFromFile(file, atype) => Input::FlashAssetFromFile(file, atype),
                dashboard_page::Output::FlashAssetFromUrl(url, atype) => Input::FlashAssetFromUrl(url, atype),
            });

        let devices_page = devices_page::Model::builder()
            .launch(settings.clone())
            .forward(&sender.input_sender(), |message| match message {
                devices_page::Output::DeviceConnected(device) => Input::DeviceConnected(device),
            });

        let fwupd_page = fwupd_page::Model::builder()
            .launch(())
            .detach();

        let settings_page = settings_page::Model::builder()
            .launch(settings.clone())
            .detach();

        let activity_page = activity_page::Model::builder()
            .launch(())
            .detach();

        // Initialize model
        let model = Model {
            // UI state
            active_view: View::Activities,
            is_connected: false,
            // Components
            dashboard_page,
            devices_page,
            fwupd_page,
            settings_page,
            activity_page,
            // Other
            infinitime: None,
            toast_overlay: adw::ToastOverlay::new(),
            hide_on_startup: start_in_background,
        };

        // Widgets
        let toast_overlay = model.toast_overlay.clone();
        let widgets = view_output!();

        // Settings
        let window = widgets.main_window.clone();
        settings.connect_changed(Some(SETTING_BACKGROUND), move |settings, _| {
            window.set_hide_on_close(settings.boolean(SETTING_BACKGROUND));
        });

        // Actions
        let app = relm4::main_application();
        app.set_accelerators_for_action::<CloseAction>(&["<primary>W"]);
        app.set_accelerators_for_action::<QuitAction>(&["<primary>Q"]);

        let mut view_group = RelmActionGroup::<ViewActionGroup>::new();
        view_group.add_action(RelmAction::<DashboardViewAction>::new_stateless(
            glib::clone!(#[strong] sender, move |_| {
                sender.input(Input::SetView(View::Dashboard));
            }
        )));
        view_group.add_action(RelmAction::<DevicesViewAction>::new_stateless(
            glib::clone!(#[strong] sender, move |_| {
                sender.input(Input::SetView(View::Devices));
            }
        )));
        view_group.add_action(RelmAction::<SettingsViewAction>::new_stateless(
            glib::clone!(#[strong] sender, move |_| {
                sender.input(Input::SetView(View::Settings));
            }
        )));
        view_group.add_action(RelmAction::<ActivitiesViewAction>::new_stateless(
            glib::clone!(#[strong] sender, move |_| {
                sender.input(Input::SetView(View::Activities));
            }
        )));
        view_group.add_action(RelmAction::<AboutAction>::new_stateless(
            glib::clone!(#[strong] sender, move |_| {
                sender.input(Input::About);
            }
        )));
        view_group.register_for_widget(&widgets.main_window);

        let mut global_group = RelmActionGroup::<WindowActionGroup>::new();
        global_group.add_action(RelmAction::<QuitAction>::new_stateless(
            glib::clone!(#[strong] sender, move |_| {
                sender.input(Input::Quit);
            }
        )));
        global_group.add_action(RelmAction::<CloseAction>::new_stateless(
            glib::clone!(#[strong] sender, move |_| {
                sender.input(Input::Close);
            }
        )));
        global_group.register_for_widget(&widgets.main_window);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match msg {
            Input::SetView(view) => {
                if self.active_view != view {
                    if self.active_view == View::Devices {
                        self.devices_page.emit(devices_page::Input::StopDiscovery);
                    }
                    if view == View::Devices {
                        self.devices_page.emit(devices_page::Input::StartDiscovery);
                    }
                    self.active_view = view;
                }
            }
            Input::DeviceConnected(device) => {
                log::info!("Device connected: {}", device.address());
                self.is_connected = true;
                let address = device.address();

                relm4::spawn(async move {
                    let mut attempts = 0;

                    loop {
                        attempts += 1;

                        log::info!(
                            "Waiting for InfiniTime GATT services, attempt {}",
                            attempts
                        );

                        match wait_for_services_resolved(device.clone(), Duration::from_secs(20)).await {
                            Ok(true) => {
                                log::info!("InfiniTime GATT services resolved");
                            }
                            Ok(false) => {
                                log::warn!(
                                    "InfiniTime GATT services did not resolve on attempt {}",
                                    attempts
                                );
                            }
                            Err(error) => {
                                log::warn!(
                                    "Failed while waiting for GATT services on attempt {}: {}",
                                    attempts,
                                    error
                                );
                            }
                        }

                        match bt::InfiniTime::new(device.clone()).await {
                            Ok(infinitime) => {
                                log::info!(
                                    "InfiniTime services loaded after {} attempt(s)",
                                    attempts
                                );

                                sender.input(Input::DeviceReady(Arc::new(infinitime)));
                                break;
                            }

                            Err(error) => {
                                log::warn!(
                                    "InfiniTime services still unavailable on attempt {}: {}",
                                    attempts,
                                    error
                                );

                                if attempts >= 5 {
                                    log::warn!(
                                        "Failed to load InfiniTime services after {} attempts",
                                        attempts
                                    );

                                    sender.input(Input::DeviceRejected);
                                    sender.input(Input::ToastStatic("Device is rejected by the app"));
                                    sender.input(Input::DeviceConnectionLost(address));
                                    break;

                                }

                                sleep(Duration::from_secs(3)).await;
                            }
                        }
                    }
                });
            }
            Input::DeviceDisconnected => {
                let Some(infinitime) = self.infinitime.take() else {
                    log::debug!("Ignoring duplicate disconnect event");
                    return;
                };

                let address = infinitime.device().address();

                log::info!("PineTime disconnected: {}", address);

                self.is_connected = false;

                self.devices_page
                    .emit(devices_page::Input::DeviceConnectionLost(address));

                self.dashboard_page.emit(dashboard_page::Input::Disconnected);
                self.fwupd_page.emit(fwupd_page::Input::Disconnected);

            }
            Input::DeviceConnectionLost(address) => {
                log::info!("PineTime connection lost before ready: {}", address);

                self.is_connected = false;

                self.devices_page
                    .emit(devices_page::Input::DeviceConnectionLost(address));

                self.dashboard_page.emit(dashboard_page::Input::Disconnected);
                self.fwupd_page.emit(fwupd_page::Input::Disconnected);

            }
            Input::DeviceReady(infinitime) => {
                log::info!("PineTime recognized");
                self.infinitime = Some(infinitime.clone());

                //if self.active_view == View::Devices {
                //    self.active_view = View::Dashboard;
                //}

                self.dashboard_page.emit(dashboard_page::Input::Connected(infinitime.clone()));
                self.fwupd_page.emit(fwupd_page::Input::Connected(infinitime.clone()));

                // Handle explicit Bluetooth disconnect events
                let disconnect_sender = sender.clone();
                let infinitime_for_disconnect = infinitime.clone();

                relm4::spawn(async move {
                    match infinitime_for_disconnect.get_property_stream().await {
                        Ok(stream) => {
                            pin_mut!(stream);

                            while let Some(property) = stream.next().await {
                                log::debug!("Device property changed: {:?}", property);

                                if let bluer::DeviceProperty::Connected(false) = property {
                                    log::info!("PineTime connection lost");
                                    break;
                                }
                            }
                        }
                        Err(error) => {
                            log::error!("Failed to get property stream: {}", error);
                        }
                    }

                    disconnect_sender.input(Input::DeviceDisconnected);
                });

                // Keep the BLE connection alive and detect silent drops
                let healthcheck_sender = sender.clone();
                let infinitime_for_healthcheck = infinitime.clone();

                relm4::spawn(async move {
                    loop {
                        sleep(Duration::from_secs(60)).await;

                        match infinitime_for_healthcheck.read_battery_level().await {
                            Ok(level) => {
                                log::debug!("PineTime health check OK. Battery: {}%", level);
                            }
                            Err(error) => {
                                log::warn!("PineTime health check failed: {}", error);
                                healthcheck_sender.input(Input::DeviceDisconnected);
                                break;
                            }
                        }
                    }
                });
            }
            Input::DeviceRejected => {
                self.devices_page.emit(devices_page::Input::StartDiscovery);
            }
            Input::FlashAssetFromFile(file, atype) => {
                self.fwupd_page.emit(fwupd_page::Input::FlashAssetFromFile(file, atype));
                sender.input(Input::SetView(View::FirmwareUpdate));
            }
            Input::FlashAssetFromUrl(url, atype) => {
                self.fwupd_page.emit(fwupd_page::Input::FlashAssetFromUrl(url, atype));
                sender.input(Input::SetView(View::FirmwareUpdate));
            }
            Input::Toast(message) => {
                self.toast_overlay.add_toast(adw::Toast::new(&message));
            }
            Input::ToastStatic(message) => {
                self.toast_overlay.add_toast(adw::Toast::new(message));
            }
            Input::ToastWithLink { message, label, url } => {
                let toast = adw::Toast::new(message);
                let root = root.clone();
                toast.set_button_label(Some(label));
                toast.connect_button_clicked(move |_| {
                    gtk::UriLauncher::new(&url)
                        .launch(Some(&root), gio::Cancellable::NONE, |_| ());
                });
                self.toast_overlay.add_toast(toast);
            }
            Input::WindowShown => {
                // Temporary hack required because Relm4 unconditionaly makes the
                // main window visible upon gtk::Application::activate signal
                if self.hide_on_startup {
                    root.set_visible(false);
                }
                self.hide_on_startup = false;
            }
            Input::About => {
                adw::AboutWindow::builder()
                    .transient_for(root)
                    .application_icon(APP_ID)
                    .application_name("Pegasus")
                    .version("v0.5.3")
                    .website("https://github.com/GraeUS/Pegasus")
                    .issue_url("https://github.com/GraeUS/Pegasus/issues")
                    .license_type(gtk::License::Gpl30)
                    .build()
                    .present();
            }
            Input::Close => {
                root.close();
            }
            Input::Quit => {
                root.application().unwrap().quit();
            }
        }
    }
}



#[derive(Debug, PartialEq)]
pub enum View {
    Dashboard,
    Devices,
    FirmwareUpdate,
    Settings,
    Activities,
}


pub fn run() {
    // Init GTK before libadwaita (ToastOverlay)
    gtk::init().unwrap();

    // Init icons
    relm4_icons::initialize_icons(
        icon_names::GRESOURCE_BYTES,
        icon_names::RESOURCE_PREFIX
    );

    // Handle CLI args
    let known_args = ["--background"];
    let (local_args, other_args): (Vec<_>, Vec<_>) = env::args()
        .partition(|a| known_args.contains(&a.as_str()));
    let start_in_background = local_args.contains(&String::from("--background"));

    // Run app
    RelmApp::new(APP_ID)
        .with_args(other_args)
        .with_broker(&BROKER)
        .run::<Model>(start_in_background);
}
