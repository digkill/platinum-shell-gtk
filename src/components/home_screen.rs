use crate::device::DeviceSnapshot;
use crate::navigation::NavDestination;
use crate::ui::picture_icon;
use relm4::gtk::prelude::*;
use relm4::gtk::{self, Align, Orientation};
use relm4::prelude::*;

pub struct HomeScreen {
    snapshot: DeviceSnapshot,
}

#[derive(Debug, Clone)]
pub enum HomeScreenInput {
    SetSnapshot(DeviceSnapshot),
    OpenSection(NavDestination),
}

#[derive(Debug, Clone, Copy)]
pub enum HomeScreenOutput {
    Navigate(NavDestination),
}

#[relm4::component(pub)]
impl SimpleComponent for HomeScreen {
    type Init = DeviceSnapshot;
    type Input = HomeScreenInput;
    type Output = HomeScreenOutput;

    view! {
        gtk::Box {
            set_orientation: Orientation::Vertical,
            set_spacing: 18,
            add_css_class: "home-screen",

            append = &gtk::Box {
                set_orientation: Orientation::Vertical,
                set_spacing: 8,
                set_halign: Align::Fill,
                add_css_class: "home-clock-card",

                append = &gtk::Box {
                    set_orientation: Orientation::Horizontal,
                    set_halign: Align::Center,
                    add_css_class: "home-clock-row",

                    append = &gtk::Label {
                        set_halign: Align::Center,
                        add_css_class: "home-clock-value",
                        #[watch]
                        set_label: &hours_label(&model.snapshot),
                    },

                    append = &gtk::Label {
                        set_halign: Align::Center,
                        add_css_class: "home-clock-separator",
                        set_label: ":",
                    },

                    append = &gtk::Label {
                        set_halign: Align::Center,
                        add_css_class: "home-clock-value",
                        #[watch]
                        set_label: &minutes_label(&model.snapshot),
                    },
                },

                append = &gtk::Label {
                    set_halign: Align::Center,
                    add_css_class: "home-clock-date",
                    #[watch]
                    set_label: &model.snapshot.date_label,
                }
            },

            append = &gtk::Box {
                set_orientation: Orientation::Vertical,
                set_spacing: 20,
                add_css_class: "home-launcher-sheet",

                append = &gtk::Box {
                    set_height_request: 2,
                    add_css_class: "home-divider",
                },

                append = &launcher_grid(sender.input_sender().clone()),

                append = &gtk::Box {
                    set_height_request: 2,
                    add_css_class: "home-divider",
                },
            },
        }
    }

    fn init(
        snapshot: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = HomeScreen { snapshot };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            HomeScreenInput::SetSnapshot(snapshot) => {
                self.snapshot = snapshot;
            }
            HomeScreenInput::OpenSection(destination) => {
                sender.output(HomeScreenOutput::Navigate(destination)).ok();
            }
        }
    }
}

fn hours_label(snapshot: &DeviceSnapshot) -> String {
    snapshot
        .time_label
        .split_once(':')
        .map(|(hours, _)| hours.to_string())
        .unwrap_or_else(|| String::from("00"))
}

fn minutes_label(snapshot: &DeviceSnapshot) -> String {
    snapshot
        .time_label
        .split_once(':')
        .map(|(_, minutes)| minutes.to_string())
        .unwrap_or_else(|| String::from("00"))
}

fn launcher_grid(input_sender: relm4::Sender<HomeScreenInput>) -> gtk::FlowBox {
    let flow = gtk::FlowBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .row_spacing(18)
        .column_spacing(12)
        .halign(Align::Center)
        .min_children_per_line(4)
        .max_children_per_line(4)
        .activate_on_single_click(false)
        .build();
    flow.add_css_class("home-app-grid");

    flow.insert(&static_app_button("calendar-home", "Calendar"), -1);
    flow.insert(&static_app_button("clock-home", "Clock"), -1);
    flow.insert(&static_app_button("contacts-home", "Contacts"), -1);
    flow.insert(
        &nav_app_button(
            "platinum-one-home",
            "Platinum One",
            input_sender.clone(),
            NavDestination::Apps,
        ),
        -1,
    );
    flow.insert(
        &nav_app_button("ai-home", "AI", input_sender.clone(), NavDestination::Ai),
        -1,
    );
    flow.insert(
        &nav_app_button(
            "settings-home",
            "Settings",
            input_sender,
            NavDestination::Settings,
        ),
        -1,
    );

    flow
}

fn static_app_button(icon_name: &str, title: &str) -> gtk::Button {
    let button = gtk::Button::builder().build();
    button.add_css_class("home-app-button");
    button.set_child(Some(&app_button_content(icon_name, title)));
    button
}

fn nav_app_button(
    icon_name: &str,
    title: &str,
    input_sender: relm4::Sender<HomeScreenInput>,
    destination: NavDestination,
) -> gtk::Button {
    let button = gtk::Button::builder().build();
    button.add_css_class("home-app-button");
    button.set_child(Some(&app_button_content(icon_name, title)));
    button.connect_clicked(move |_| {
        let _ = input_sender.send(HomeScreenInput::OpenSection(destination));
    });
    button
}

fn app_button_content(icon_name: &str, title: &str) -> gtk::Box {
    let content = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    let icon_shell = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .width_request(74)
        .height_request(74)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();
    icon_shell.add_css_class("home-app-icon-shell");
    icon_shell.append(&picture_icon(icon_name, 74));

    let label = gtk::Label::new(Some(title));
    label.set_halign(Align::Center);
    label.set_wrap(true);
    label.set_wrap_mode(relm4::gtk::pango::WrapMode::WordChar);
    label.set_justify(relm4::gtk::Justification::Center);
    label.add_css_class("home-app-label");

    content.append(&icon_shell);
    content.append(&label);
    content
}
