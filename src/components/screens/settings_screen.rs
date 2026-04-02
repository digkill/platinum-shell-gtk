use crate::theme::ThemeMode;
use crate::ui::{action_row, marker_pill, preference_group, surface_header, titled_card};
use adw::prelude::PreferencesGroupExt;
use relm4::gtk::prelude::*;
use relm4::gtk::{self, Orientation};
use relm4::prelude::*;

pub struct SettingsScreen {
    theme_mode: ThemeMode,
}

#[derive(Debug, Clone, Copy)]
pub enum SettingsScreenInput {
    SetThemeMode(ThemeMode),
    ChooseTheme(ThemeMode),
}

#[derive(Debug, Clone, Copy)]
pub enum SettingsScreenOutput {
    ChooseTheme(ThemeMode),
}

#[relm4::component(pub)]
impl SimpleComponent for SettingsScreen {
    type Init = ThemeMode;
    type Input = SettingsScreenInput;
    type Output = SettingsScreenOutput;

    view! {
        gtk::Box {
            set_orientation: Orientation::Vertical,
            set_spacing: 16,

            append = &surface_header("Settings", "Shell appearance and device profile controls.", model.theme_mode.title()),

            append = &appearance_card(model.theme_mode, sender.input_sender().clone()),

            append = &device_group(
                "Device Controls",
                "Shell-level placeholders before hardware integration.",
                &[
                    ("Glass Density", "High translucency tuned for premium shell feel", "Enabled"),
                    ("Battery Saver", "Reduce refresh and transmit peaks", "Off"),
                    ("Background Notes", "Keep assistant context warm", "On"),
                ],
            ),
        }
    }

    fn init(
        theme_mode: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = SettingsScreen { theme_mode };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            SettingsScreenInput::SetThemeMode(theme_mode) => {
                self.theme_mode = theme_mode;
            }
            SettingsScreenInput::ChooseTheme(theme_mode) => {
                self.theme_mode = theme_mode;
                sender
                    .output(SettingsScreenOutput::ChooseTheme(theme_mode))
                    .ok();
            }
        }
    }
}

fn appearance_card(
    theme_mode: ThemeMode,
    input_sender: relm4::Sender<SettingsScreenInput>,
) -> gtk::Box {
    let card = titled_card(
        "Appearance",
        "Adaptive shell theme with bright glass, dark field mode, or automatic switching.",
    );
    let state_marker = marker_pill(theme_mode.title(), "accent-cyan");

    let segmented = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .build();
    segmented.add_css_class("theme-segmented");

    segmented.append(&theme_button(
        "Light",
        theme_mode == ThemeMode::Light,
        input_sender.clone(),
        ThemeMode::Light,
    ));
    segmented.append(&theme_button(
        "Dark",
        theme_mode == ThemeMode::Dark,
        input_sender.clone(),
        ThemeMode::Dark,
    ));
    segmented.append(&theme_button(
        "Auto",
        theme_mode == ThemeMode::Auto,
        input_sender,
        ThemeMode::Auto,
    ));

    card.append(&crate::ui::divider());
    card.append(&state_marker);
    card.append(&segmented);
    card
}

fn theme_button(
    title: &str,
    is_active: bool,
    input_sender: relm4::Sender<SettingsScreenInput>,
    mode: ThemeMode,
) -> gtk::Button {
    let button = gtk::Button::with_label(title);
    button.add_css_class("theme-button");

    if is_active {
        button.add_css_class("theme-button-active");
    }

    button.connect_clicked(move |_| {
        let _ = input_sender.send(SettingsScreenInput::ChooseTheme(mode));
    });

    button
}

fn device_group(title: &str, subtitle: &str, items: &[(&str, &str, &str)]) -> gtk::Box {
    let container = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .build();

    let group = preference_group(title, subtitle);
    group.add_css_class("material-group");

    for (item_title, item_subtitle, item_trailing) in items.iter() {
        group.add(&action_row(item_title, item_subtitle, item_trailing));
    }

    container.append(&group);
    container
}
