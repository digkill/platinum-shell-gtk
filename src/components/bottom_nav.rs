use crate::navigation::{AppRoute, AppSurface};
use crate::ui::nav_icon;
use relm4::gtk::glib;
use relm4::gtk::prelude::*;
use relm4::gtk::{self, Align, Orientation};
use relm4::prelude::*;

pub struct BottomNav {
    active_route: AppRoute,
    shell_box: gtk::Box,
    home_button: gtk::Button,
    apps_button: gtk::Button,
    call_button: gtk::Button,
    message_button: gtk::Button,
}

#[derive(Debug, Clone, Copy)]
pub enum BottomNavInput {
    SetActive(AppRoute),
    Navigate(AppRoute),
}

#[derive(Debug, Clone, Copy)]
pub enum BottomNavOutput {
    Navigate(AppRoute),
}

#[relm4::component(pub)]
impl SimpleComponent for BottomNav {
    type Init = AppRoute;
    type Input = BottomNavInput;
    type Output = BottomNavOutput;

    view! {
        #[name = "shell_box"]
        gtk::Box {
            set_orientation: Orientation::Horizontal,
            set_spacing: 6,
            set_halign: Align::Center,
            set_margin_bottom: 0,
            add_css_class: "nav-card",

            #[name = "home_button"]
            gtk::Button {
                add_css_class: "nav-button",
                connect_clicked[sender] => move |_| {
                    sender.input(BottomNavInput::Navigate(AppRoute::Surface(AppSurface::Home)));
                },
                set_tooltip_text: Some("Home"),
                set_child = Some(&nav_content("dock-home")),
            },
            #[name = "apps_button"]
            gtk::Button {
                add_css_class: "nav-button",
                connect_clicked[sender] => move |_| {
                    sender.input(BottomNavInput::Navigate(AppRoute::Surface(AppSurface::Apps)));
                },
                set_tooltip_text: Some("Apps"),
                set_child = Some(&nav_content("dock-apps")),
            },
            #[name = "call_button"]
            gtk::Button {
                add_css_class: "nav-button",
                connect_clicked[sender] => move |_| {
                    sender.input(BottomNavInput::Navigate(AppRoute::App("call")));
                },
                set_tooltip_text: Some("Call"),
                set_child = Some(&nav_content("dock-call")),
            },
            #[name = "message_button"]
            gtk::Button {
                add_css_class: "nav-button",
                connect_clicked[sender] => move |_| {
                    sender.input(BottomNavInput::Navigate(AppRoute::App("message")));
                },
                set_tooltip_text: Some("Message"),
                set_child = Some(&nav_content("dock-message")),
            },
        }
    }

    fn init(
        active_route: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();
        let model = BottomNav {
            active_route,
            shell_box: widgets.shell_box.clone(),
            home_button: widgets.home_button.clone(),
            apps_button: widgets.apps_button.clone(),
            call_button: widgets.call_button.clone(),
            message_button: widgets.message_button.clone(),
        };
        attach_press_feedback(&model.home_button);
        attach_press_feedback(&model.apps_button);
        attach_press_feedback(&model.call_button);
        attach_press_feedback(&model.message_button);
        model.apply_active_state();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            BottomNavInput::SetActive(route) => {
                self.active_route = route;
                self.apply_active_state();
            }
            BottomNavInput::Navigate(route) => {
                sender.output(BottomNavOutput::Navigate(route)).ok();
            }
        }
    }
}

fn nav_content(icon_name: &str) -> gtk::Box {
    let content = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(6)
        .halign(Align::Center)
        .valign(Align::Center)
        .build();

    let icon = nav_icon(icon_name);
    let underline = gtk::Box::builder()
        .width_request(28)
        .height_request(3)
        .halign(Align::Center)
        .build();
    underline.add_css_class("nav-underline");

    let underline_revealer = gtk::Revealer::builder()
        .transition_type(gtk::RevealerTransitionType::Crossfade)
        .transition_duration(160)
        .reveal_child(false)
        .halign(Align::Center)
        .build();
    underline_revealer.set_child(Some(&underline));

    content.append(&icon);
    content.append(&underline_revealer);
    content
}

impl BottomNav {
    fn apply_active_state(&self) {
        pulse_nav_shell(&self.shell_box);
        sync_button_state(
            &self.home_button,
            self.active_route == AppRoute::Surface(AppSurface::Home),
        );
        sync_button_state(
            &self.apps_button,
            self.active_route == AppRoute::Surface(AppSurface::Apps),
        );
        sync_button_state(
            &self.call_button,
            self.active_route == AppRoute::App("call"),
        );
        sync_button_state(
            &self.message_button,
            self.active_route == AppRoute::App("message"),
        );
    }
}

fn sync_button_state(button: &gtk::Button, is_active: bool) {
    if is_active {
        button.add_css_class("nav-button-active");
        pulse_nav_button(button);
    } else {
        button.remove_css_class("nav-button-active");
    }

    if let Some(content) = button.child().and_downcast::<gtk::Box>() {
        let underline = content.last_child().and_downcast::<gtk::Revealer>();
        if let Some(underline) = underline {
            underline.set_reveal_child(is_active);
        }
    }
}

fn attach_press_feedback(button: &gtk::Button) {
    let gesture = gtk::GestureClick::new();
    let pressed_button = button.clone();
    gesture.connect_pressed(move |_, _, _, _| {
        if let Some(icon) = nav_icon_picture(&pressed_button) {
            icon.set_size_request(39, 39);
        }
    });

    let released_button = button.clone();
    gesture.connect_released(move |_, _, _, _| {
        if let Some(icon) = nav_icon_picture(&released_button) {
            icon.set_size_request(42, 42);
        }
    });

    let cancelled_button = button.clone();
    gesture.connect_stopped(move |_| {
        if let Some(icon) = nav_icon_picture(&cancelled_button) {
            icon.set_size_request(42, 42);
        }
    });

    button.add_controller(gesture);
}

fn pulse_nav_button(button: &gtk::Button) {
    let Some(icon) = nav_icon_picture(button) else {
        return;
    };

    icon.set_size_request(42, 42);

    let grow = icon.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(18), move || {
        grow.set_size_request(45, 45);
    });

    let rebound = icon.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(92), move || {
        rebound.set_size_request(43, 43);
    });

    let settle = icon.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(168), move || {
        settle.set_size_request(42, 42);
    });
}

fn nav_icon_picture(button: &gtk::Button) -> Option<gtk::Picture> {
    let content = button.child()?.downcast::<gtk::Box>().ok()?;
    content.first_child()?.downcast::<gtk::Picture>().ok()
}

fn pulse_nav_shell(shell: &gtk::Box) {
    shell.add_css_class("nav-card-pulse");

    let soften = shell.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(110), move || {
        soften.remove_css_class("nav-card-pulse");
        soften.add_css_class("nav-card-pulse-settle");
    });

    let settle = shell.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(210), move || {
        settle.remove_css_class("nav-card-pulse");
        settle.remove_css_class("nav-card-pulse-settle");
    });
}
