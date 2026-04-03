use crate::device::{DeviceSnapshot, NetworkKind};
use crate::ui::{battery_indicator, network_kind_icon, signal_indicator};
use relm4::gtk::prelude::*;
use relm4::gtk::{self, Align, Orientation};
use relm4::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickToggle {
    Wifi,
    Lte,
    Silent,
    BatterySaver,
}

#[derive(Debug, Clone)]
pub struct TopDrawerState {
    pub snapshot: DeviceSnapshot,
    pub wifi_enabled: bool,
    pub lte_enabled: bool,
    pub silent_mode: bool,
    pub battery_saver: bool,
    pub notice: Option<String>,
}

pub struct TopDrawer {
    content: gtk::Box,
    open: bool,
    revealer: gtk::Revealer,
    state: TopDrawerState,
}

#[derive(Debug)]
pub enum TopDrawerInput {
    SetOpen(bool),
    SetState(TopDrawerState),
}

#[derive(Debug)]
pub enum TopDrawerOutput {
    ToggleRequested,
    ToggleQuick(QuickToggle),
}

#[relm4::component(pub)]
impl SimpleComponent for TopDrawer {
    type Init = TopDrawerState;
    type Input = TopDrawerInput;
    type Output = TopDrawerOutput;

    view! {
        gtk::Box {
            set_orientation: Orientation::Vertical,
            set_halign: Align::Fill,
            set_valign: Align::Start,
            set_can_target: false,
            add_css_class: "top-drawer-layer",

            append = &gtk::Button {
                set_halign: Align::Center,
                set_margin_bottom: 6,
                add_css_class: "top-drawer-handle-button",
                connect_clicked[sender] => move |_| {
                    sender.output(TopDrawerOutput::ToggleRequested).ok();
                },

                gtk::Box {
                    add_css_class: "top-drawer-handle",
                }
            },

            #[name(revealer)]
            gtk::Revealer {
                set_transition_type: gtk::RevealerTransitionType::SlideDown,
                set_transition_duration: 220,
                set_reveal_child: false,

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: Orientation::Vertical,
                    set_spacing: 14,
                    set_margin_start: 6,
                    set_margin_end: 6,
                    add_css_class: "top-drawer-sheet",

                    #[name(content)]
                    gtk::Box {
                        set_orientation: Orientation::Vertical,
                        set_spacing: 14,
                    }
                }
            }
        }
    }

    fn init(
        state: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();
        rebuild_content(&widgets.content, &state, sender.output_sender().clone());
        let model = TopDrawer {
            content: widgets.content.clone(),
            open: false,
            revealer: widgets.revealer.clone(),
            state,
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            TopDrawerInput::SetOpen(open) => {
                self.open = open;
                self.revealer.set_reveal_child(self.open);
            }
            TopDrawerInput::SetState(state) => {
                self.state = state;
                rebuild_content(&self.content, &self.state, sender.output_sender().clone());
            }
        }
    }
}

fn rebuild_content(
    container: &gtk::Box,
    state: &TopDrawerState,
    output: relm4::Sender<TopDrawerOutput>,
) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    container.append(&header(&state.snapshot));
    if let Some(notice) = &state.notice {
        container.append(&notice_row(notice));
    }
    container.append(&quick_grid(state, output));
    container.append(&system_rows(state));
}

fn header(snapshot: &DeviceSnapshot) -> gtk::Box {
    let row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();

    let copy = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();

    let title = gtk::Label::new(Some(&snapshot.time_label));
    title.set_halign(Align::Start);
    title.add_css_class("top-drawer-time");

    let subtitle = gtk::Label::new(Some(&snapshot.date_label));
    subtitle.set_halign(Align::Start);
    subtitle.add_css_class("top-drawer-date");

    let battery = battery_indicator(snapshot.battery_level, snapshot.battery_charging);
    battery.add_css_class("top-drawer-battery");

    copy.append(&title);
    copy.append(&subtitle);
    row.append(&copy);
    row.append(&battery);
    row
}

fn quick_grid(state: &TopDrawerState, output: relm4::Sender<TopDrawerOutput>) -> gtk::Grid {
    let grid = gtk::Grid::builder()
        .row_spacing(10)
        .column_spacing(10)
        .build();

    grid.attach(
        &toggle_tile(
            "Wi-Fi",
            if state.wifi_enabled { "On" } else { "Off" },
            Some(network_kind_icon(NetworkKind::Wifi).upcast()),
            state.wifi_enabled,
            QuickToggle::Wifi,
            output.clone(),
        ),
        0,
        0,
        1,
        1,
    );
    grid.attach(
        &toggle_tile(
            "LTE",
            if state.lte_enabled { "On" } else { "Off" },
            Some(network_kind_icon(NetworkKind::Lte).upcast()),
            state.lte_enabled,
            QuickToggle::Lte,
            output.clone(),
        ),
        1,
        0,
        1,
        1,
    );
    grid.attach(
        &toggle_tile(
            "Silent",
            if state.silent_mode { "Enabled" } else { "Off" },
            None,
            state.silent_mode,
            QuickToggle::Silent,
            output.clone(),
        ),
        0,
        1,
        1,
        1,
    );
    grid.attach(
        &toggle_tile(
            "Saver",
            if state.battery_saver {
                "Active"
            } else {
                "Normal"
            },
            Some(signal_indicator(state.snapshot.signal_level).upcast()),
            state.battery_saver,
            QuickToggle::BatterySaver,
            output,
        ),
        1,
        1,
        1,
        1,
    );

    grid
}

fn toggle_tile(
    title: &str,
    value: &str,
    leading: Option<gtk::Widget>,
    active: bool,
    toggle: QuickToggle,
    output: relm4::Sender<TopDrawerOutput>,
) -> gtk::Button {
    let button = gtk::Button::builder().build();
    button.add_css_class("top-drawer-tile-button");
    if active {
        button.add_css_class("top-drawer-tile-active");
    }

    let tile = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .hexpand(true)
        .build();
    tile.add_css_class("top-drawer-tile");

    if let Some(leading) = leading {
        tile.append(&leading);
    }

    let title_label = gtk::Label::new(Some(title));
    title_label.set_halign(Align::Start);
    title_label.add_css_class("eyebrow-label");

    let value_label = gtk::Label::new(Some(value));
    value_label.set_halign(Align::Start);
    value_label.add_css_class("section-row-title");

    tile.append(&title_label);
    tile.append(&value_label);
    button.set_child(Some(&tile));
    button.connect_clicked(move |_| {
        let _ = output.send(TopDrawerOutput::ToggleQuick(toggle));
    });
    button
}

fn system_rows(state: &TopDrawerState) -> gtk::Box {
    let rows = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .build();

    rows.append(&detail_row(
        "Connectivity",
        network_label(
            state.snapshot.network_kind,
            state.snapshot.network_is_online,
        ),
    ));
    rows.append(&detail_row(
        "Power",
        if state.snapshot.battery_charging {
            "Charging"
        } else if state.battery_saver {
            "Battery Saver"
        } else {
            "On battery"
        },
    ));
    rows.append(&detail_row(
        "Profile",
        if state.silent_mode {
            "Silent"
        } else {
            "Default"
        },
    ));
    rows
}

fn detail_row(title: &str, value: &str) -> gtk::Box {
    let row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    row.add_css_class("top-drawer-row");

    let title_label = gtk::Label::new(Some(title));
    title_label.set_halign(Align::Start);
    title_label.set_hexpand(true);
    title_label.add_css_class("section-row-title");

    let value_label = gtk::Label::new(Some(value));
    value_label.set_halign(Align::End);
    value_label.add_css_class("section-row-subtitle");

    row.append(&title_label);
    row.append(&value_label);
    row
}

fn notice_row(message: &str) -> gtk::Box {
    let row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(10)
        .build();
    row.add_css_class("top-drawer-notice");

    let label = gtk::Label::new(Some(message));
    label.set_halign(Align::Start);
    label.set_wrap(true);
    label.add_css_class("section-row-subtitle");

    row.append(&label);
    row
}

fn network_label(kind: NetworkKind, online: bool) -> &'static str {
    if !online {
        return "Offline";
    }

    match kind {
        NetworkKind::Wifi => "Wi-Fi",
        NetworkKind::Ethernet => "Ethernet",
        NetworkKind::Lte => "LTE",
        NetworkKind::Unknown => "Connected",
        NetworkKind::Offline => "Offline",
    }
}
