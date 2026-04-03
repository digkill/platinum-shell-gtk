use crate::app_sdk::{AppLaunchPayload, AppLaunchRequest, AppManifest};
use crate::ui::picture_icon;
use relm4::gtk::prelude::*;
use relm4::gtk::{self, Align, Orientation};
use relm4::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppDrawerEntry {
    pub manifest: AppManifest,
    pub payload: Option<AppLaunchPayload>,
    pub is_active: bool,
}

pub struct AppDrawer {
    entries: Vec<AppDrawerEntry>,
    list_box: gtk::Box,
    open: bool,
    revealer: gtk::Revealer,
}

#[derive(Debug)]
pub enum AppDrawerInput {
    SetEntries(Vec<AppDrawerEntry>),
    SetOpen(bool),
}

#[derive(Debug)]
pub enum AppDrawerOutput {
    OpenApp(AppLaunchRequest),
}

#[relm4::component(pub)]
impl SimpleComponent for AppDrawer {
    type Init = ();
    type Input = AppDrawerInput;
    type Output = AppDrawerOutput;

    view! {
        gtk::Box {
            set_orientation: Orientation::Vertical,
            set_hexpand: true,
            set_valign: Align::End,
            set_halign: Align::Fill,
            set_can_target: false,
            add_css_class: "app-drawer-layer",

            #[name(revealer)]
            gtk::Revealer {
                set_transition_type: gtk::RevealerTransitionType::SlideUp,
                set_transition_duration: 220,
                set_reveal_child: false,

                #[wrap(Some)]
                set_child = &gtk::Box {
                    set_orientation: Orientation::Vertical,
                    set_spacing: 12,
                    set_margin_start: 8,
                    set_margin_end: 8,
                    set_margin_bottom: 16,
                    add_css_class: "app-drawer-sheet",

                    append = &gtk::Box {
                        set_orientation: Orientation::Horizontal,
                        set_spacing: 12,

                        append = &gtk::Label {
                            set_hexpand: true,
                            set_halign: Align::Start,
                            add_css_class: "app-drawer-title",
                            set_label: "Recent Apps",
                        },

                        append = &gtk::Button {
                            set_label: "Close",
                            add_css_class: "app-drawer-close",
                            connect_clicked[sender] => move |_| {
                                sender.input(AppDrawerInput::SetOpen(false));
                            }
                        }
                    },

                    append = &gtk::ScrolledWindow {
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        set_vscrollbar_policy: gtk::PolicyType::Automatic,
                        set_min_content_height: 236,
                        set_max_content_height: 320,

                        #[name(list_box)]
                        gtk::Box {
                            set_orientation: Orientation::Vertical,
                            set_spacing: 10,
                            add_css_class: "app-drawer-list",
                        }
                    }
                }
            }
        }
    }

    fn init(
        _: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();
        let model = AppDrawer {
            entries: Vec::new(),
            list_box: widgets.list_box.clone(),
            open: false,
            revealer: widgets.revealer.clone(),
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AppDrawerInput::SetEntries(entries) => {
                self.entries = entries;
                rebuild_entries(
                    &self.list_box,
                    &self.entries,
                    sender.output_sender().clone(),
                );
            }
            AppDrawerInput::SetOpen(open) => {
                self.open = open;
                self.revealer.set_reveal_child(self.open);
            }
        }
    }
}

fn rebuild_entries(
    container: &gtk::Box,
    entries: &[AppDrawerEntry],
    output: relm4::Sender<AppDrawerOutput>,
) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    if entries.is_empty() {
        let empty = gtk::Label::new(Some("No running apps yet"));
        empty.set_halign(Align::Center);
        empty.add_css_class("section-row-subtitle");
        empty.add_css_class("app-drawer-empty");
        container.append(&empty);
        return;
    }

    for entry in entries {
        container.append(&drawer_row(entry.clone(), output.clone()));
    }
}

fn drawer_row(entry: AppDrawerEntry, output: relm4::Sender<AppDrawerOutput>) -> gtk::Button {
    let button = gtk::Button::builder().build();
    button.add_css_class("app-drawer-entry");
    if entry.is_active {
        button.add_css_class("app-drawer-entry-active");
    }

    let row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();

    let icon_shell = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .width_request(44)
        .height_request(44)
        .valign(Align::Center)
        .build();
    icon_shell.add_css_class("app-drawer-entry-icon-shell");
    icon_shell.append(&picture_icon(entry.manifest.icon_name, 28));

    let copy = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();

    let title = gtk::Label::new(Some(entry.manifest.title));
    title.set_halign(Align::Start);
    title.add_css_class("section-row-title");

    let subtitle = gtk::Label::new(Some(if entry.is_active {
        "Running now"
    } else {
        "Hold to restore"
    }));
    subtitle.set_halign(Align::Start);
    subtitle.add_css_class("section-row-subtitle");

    let state = gtk::Label::new(Some(if entry.is_active { "Live" } else { "Ready" }));
    state.set_halign(Align::End);
    state.add_css_class("eyebrow-label");

    copy.append(&title);
    copy.append(&subtitle);
    row.append(&icon_shell);
    row.append(&copy);
    row.append(&state);
    button.set_child(Some(&row));

    if entry.is_active {
        let click_output = output.clone();
        let click_app_id = entry.manifest.id;
        let click_payload = entry.payload.clone();
        button.connect_clicked(move |_| {
            let _ = click_output.send(AppDrawerOutput::OpenApp(AppLaunchRequest {
                app_id: click_app_id,
                payload: click_payload.clone(),
            }));
        });
    } else {
        let hold_output = output.clone();
        let hold_app_id = entry.manifest.id;
        let hold_payload = entry.payload.clone();
        let gesture = gtk::GestureLongPress::new();
        gesture.connect_pressed(move |_, _, _| {
            let _ = hold_output.send(AppDrawerOutput::OpenApp(AppLaunchRequest {
                app_id: hold_app_id,
                payload: hold_payload.clone(),
            }));
        });
        button.add_controller(gesture);
    }

    button
}
