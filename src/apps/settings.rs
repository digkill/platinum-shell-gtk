use crate::app_sdk::{AppContext, AppManifest, AppPermission, LauncherApp};
use crate::theme::{apply_theme, load_theme_mode, resolve_theme, save_theme_mode, ThemeMode};
use relm4::gtk;
use relm4::gtk::prelude::*;

pub struct SettingsApp;

impl LauncherApp for SettingsApp {
    fn manifest(&self) -> AppManifest {
        AppManifest {
            id: "settings",
            title: "Settings",
            icon_name: "settings",
            description: "Shell tuning, appearance, and system preferences.",
        }
    }

    fn permissions(&self) -> &'static [AppPermission] {
        &[
            AppPermission::Battery,
            AppPermission::Network,
            AppPermission::ShellState,
        ]
    }

    fn build_root(&self, context: &AppContext) -> gtk::Widget {
        let root = super::app_surface_root();
        root.append(&super::app_hero(
            "Settings",
            "Native system preferences for shell appearance, hardware state, and handheld profile.",
        ));

        let theme_card = super::app_card(
            "Appearance",
            "Preferred theme mode and launcher chrome density.",
        );
        let theme_row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10)
            .build();
        let theme_status = super::pill(&format!("Theme {}", load_theme_mode().title()));
        theme_status.set_hexpand(false);
        let light_button = theme_button("Light");
        let dark_button = theme_button("Dark");
        let auto_button = theme_button("Auto");
        sync_theme_buttons(load_theme_mode(), &light_button, &dark_button, &auto_button);

        connect_theme_button(
            &light_button,
            ThemeMode::Light,
            &theme_status,
            &light_button,
            &dark_button,
            &auto_button,
        );
        connect_theme_button(
            &dark_button,
            ThemeMode::Dark,
            &theme_status,
            &light_button,
            &dark_button,
            &auto_button,
        );
        connect_theme_button(
            &auto_button,
            ThemeMode::Auto,
            &theme_status,
            &light_button,
            &dark_button,
            &auto_button,
        );

        theme_row.append(&theme_status);
        theme_row.append(&super::pill("Launcher Native"));
        theme_row.append(&super::pill("Portrait 720x1560"));
        theme_card.append(&theme_row);

        let theme_buttons = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10)
            .build();
        theme_buttons.append(&light_button);
        theme_buttons.append(&dark_button);
        theme_buttons.append(&auto_button);
        theme_card.append(&theme_buttons);
        root.append(&theme_card);

        root.append(&super::app_row(
            "Battery",
            "Current handheld power state from the device snapshot.",
            &format!("{}%", context.snapshot.battery_level),
        ));
        root.append(&super::app_row(
            "Network",
            "Connectivity state used by AI, messaging, and future relay services.",
            if context.snapshot.network_is_online {
                "Connected"
            } else {
                "Offline"
            },
        ));
        root.append(&super::app_row(
            "Signal",
            "Current interpreted signal bars from the active transport.",
            &format!("{}/5", context.snapshot.signal_level),
        ));

        let actions_card = super::app_card(
            "Device Actions",
            "Reserved hooks for service restart, sync, and profile switching.",
        );
        let action_row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10)
            .build();
        action_row.append(&super::action_button("Sync Shell"));
        action_row.append(&super::action_button("Refresh State"));
        actions_card.append(&action_row);
        root.append(&actions_card);

        root.upcast()
    }
}

fn theme_button(label: &str) -> gtk::Button {
    let button = super::action_button(label);
    button.set_hexpand(true);
    button
}

fn connect_theme_button(
    button: &gtk::Button,
    mode: ThemeMode,
    status: &gtk::Label,
    light_button: &gtk::Button,
    dark_button: &gtk::Button,
    auto_button: &gtk::Button,
) {
    let status = status.clone();
    let light_button = light_button.clone();
    let dark_button = dark_button.clone();
    let auto_button = auto_button.clone();

    button.connect_clicked(move |clicked| {
        save_theme_mode(mode);
        apply_theme(mode);
        apply_window_theme(clicked, mode);
        status.set_label(&format!("Theme {}", mode.title()));
        sync_theme_buttons(mode, &light_button, &dark_button, &auto_button);
    });
}

fn sync_theme_buttons(
    mode: ThemeMode,
    light_button: &gtk::Button,
    dark_button: &gtk::Button,
    auto_button: &gtk::Button,
) {
    set_theme_button_state(light_button, mode == ThemeMode::Light);
    set_theme_button_state(dark_button, mode == ThemeMode::Dark);
    set_theme_button_state(auto_button, mode == ThemeMode::Auto);
}

fn set_theme_button_state(button: &gtk::Button, is_active: bool) {
    if is_active {
        button.add_css_class("theme-button-active");
    } else {
        button.remove_css_class("theme-button-active");
    }
}

fn apply_window_theme(button: &gtk::Button, mode: ThemeMode) {
    let resolved_theme = resolve_theme(mode);
    let Some(root) = button.root() else {
        return;
    };
    let Ok(window) = root.downcast::<gtk::Window>() else {
        return;
    };

    window.remove_css_class("theme-light");
    window.remove_css_class("theme-dark");

    match resolved_theme {
        crate::theme::ResolvedTheme::Light => window.add_css_class("theme-light"),
        crate::theme::ResolvedTheme::Dark => window.add_css_class("theme-dark"),
    }
}
