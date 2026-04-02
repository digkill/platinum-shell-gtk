use crate::navigation::NavDestination;
use crate::ui::nav_icon;
use relm4::gtk::prelude::*;
use relm4::gtk::{self, Align, Orientation};
use relm4::prelude::*;

pub struct BottomNav {
    active_nav: NavDestination,
    home_button: gtk::Button,
    apps_button: gtk::Button,
    ai_button: gtk::Button,
    settings_button: gtk::Button,
}

#[derive(Debug, Clone, Copy)]
pub enum BottomNavInput {
    SetActive(NavDestination),
    Navigate(NavDestination),
}

#[derive(Debug, Clone, Copy)]
pub enum BottomNavOutput {
    Navigate(NavDestination),
}

#[relm4::component(pub)]
impl SimpleComponent for BottomNav {
    type Init = NavDestination;
    type Input = BottomNavInput;
    type Output = BottomNavOutput;

    view! {
        gtk::Box {
            set_orientation: Orientation::Horizontal,
            set_spacing: 8,
            set_halign: Align::Center,
            add_css_class: "nav-card",

            #[name = "home_button"]
            gtk::Button {
                add_css_class: "nav-button",
                connect_clicked[sender] => move |_| {
                    sender.input(BottomNavInput::Navigate(NavDestination::Home));
                },

                set_child = Some(&nav_content("home", "Home")),
            },
            #[name = "apps_button"]
            gtk::Button {
                add_css_class: "nav-button",
                connect_clicked[sender] => move |_| {
                    sender.input(BottomNavInput::Navigate(NavDestination::Apps));
                },

                set_child = Some(&nav_content("apps", "Apps")),
            },
            #[name = "ai_button"]
            gtk::Button {
                add_css_class: "nav-button",
                connect_clicked[sender] => move |_| {
                    sender.input(BottomNavInput::Navigate(NavDestination::Ai));
                },

                set_child = Some(&nav_content("call", "Call")),
            },
            #[name = "settings_button"]
            gtk::Button {
                add_css_class: "nav-button",
                connect_clicked[sender] => move |_| {
                    sender.input(BottomNavInput::Navigate(NavDestination::Settings));
                },

                set_child = Some(&nav_content("message", "Message")),
            },
        }
    }

    fn init(
        active_nav: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();
        let model = BottomNav {
            active_nav,
            home_button: widgets.home_button.clone(),
            apps_button: widgets.apps_button.clone(),
            ai_button: widgets.ai_button.clone(),
            settings_button: widgets.settings_button.clone(),
        };
        model.apply_active_state();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            BottomNavInput::SetActive(destination) => {
                self.active_nav = destination;
                self.apply_active_state();
            }
            BottomNavInput::Navigate(destination) => {
                sender.output(BottomNavOutput::Navigate(destination)).ok();
            }
        }
    }
}

fn nav_content(icon_name: &str, title: &str) -> gtk::Box {
    let content = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    let icon = nav_icon(icon_name);

    let label = gtk::Label::new(Some(title));
    label.add_css_class("nav-label");

    content.append(&icon);
    content.append(&label);
    content
}

impl BottomNav {
    fn apply_active_state(&self) {
        sync_button_state(&self.home_button, self.active_nav == NavDestination::Home);
        sync_button_state(&self.apps_button, self.active_nav == NavDestination::Apps);
        sync_button_state(&self.ai_button, self.active_nav == NavDestination::Ai);
        sync_button_state(
            &self.settings_button,
            self.active_nav == NavDestination::Settings,
        );
    }
}

fn sync_button_state(button: &gtk::Button, is_active: bool) {
    if is_active {
        button.add_css_class("nav-button-active");
    } else {
        button.remove_css_class("nav-button-active");
    }
}
