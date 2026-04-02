use crate::components::bottom_nav::{BottomNav, BottomNavInput, BottomNavOutput};
use crate::components::home_screen::{HomeScreen, HomeScreenInput, HomeScreenOutput};
use crate::components::screens::ai_screen::AiScreen;
use crate::components::screens::apps_screen::AppsScreen;
use crate::components::screens::settings_screen::{
    SettingsScreen, SettingsScreenInput, SettingsScreenOutput,
};
use crate::device::{load_device_snapshot, DeviceSnapshot, NetworkKind};
use crate::navigation::NavDestination;
use crate::shell_state::{load_shell_state, ShellState};
use crate::theme::{
    apply_theme, load_theme_mode, resolve_theme, save_theme_mode, ResolvedTheme, ThemeMode,
};
use crate::ui::{
    animate_battery_indicator, battery_indicator, network_kind_icon, signal_indicator,
    signal_status_dot, update_signal_indicator, update_signal_status_dot,
};
use adw::prelude::*;
use relm4::gtk::gdk;
use relm4::gtk::glib;
use relm4::gtk::{self, Align, CssProvider, Orientation};
use relm4::prelude::*;

pub fn run() {
    let _ = adw::init();
    let app = RelmApp::new("com.platinum.shell.gtk");
    app.run::<ShellApp>(());
}

struct ShellApp {
    active_nav: NavDestination,
    apps_screen: Controller<AppsScreen>,
    ai_screen: Controller<AiScreen>,
    bottom_nav: Controller<BottomNav>,
    home_screen: Controller<HomeScreen>,
    layout_mode: LayoutMode,
    screen_carousel: adw::Carousel,
    settings_screen: Controller<SettingsScreen>,
    shell_state: ShellState,
    snapshot: DeviceSnapshot,
    stage_shell: gtk::Box,
    theme_mode: ThemeMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LayoutMode {
    Compact,
    Regular,
    Expanded,
}

#[derive(Debug)]
enum ShellAppMsg {
    Tick,
    Navigate(NavDestination),
    SetLayoutMode(LayoutMode),
    SwipeTo(NavDestination),
    SetTheme(ThemeMode),
}

#[relm4::component]
impl SimpleComponent for ShellApp {
    type Init = ();
    type Input = ShellAppMsg;
    type Output = ();

    view! {
        main_window = adw::ApplicationWindow {
            set_title: Some("Platinum Shell"),
            set_default_width: 460,
            set_default_height: 980,
            set_resizable: true,
            set_decorated: false,

            #[wrap(Some)]
            set_content = &gtk::Box {
                set_orientation: Orientation::Vertical,
                set_spacing: 4,
                set_vexpand: true,
                set_margin_top: 0,
                set_margin_bottom: 14,
                set_margin_start: 8,
                set_margin_end: 8,
                add_css_class: "shell-root",

                append = &gtk::Box {
                    set_orientation: Orientation::Horizontal,
                    set_spacing: 12,
                    set_valign: Align::Start,
                    set_margin_top: 5,
                    add_css_class: "status-card top-status-bar",

                    append = &gtk::Box {
                        set_orientation: Orientation::Vertical,
                        set_spacing: 0,
                        set_halign: Align::Start,
                        set_hexpand: true,
                        add_css_class: "status-network-card",

                        append = &gtk::Box {
                            set_orientation: Orientation::Horizontal,
                            set_spacing: 8,
                            set_halign: Align::Start,
                            add_css_class: "status-indicator-row",

                            #[local]
                            network_icon -> gtk::Picture {
                            },

                            #[local]
                            network_dot -> gtk::Box {
                            },

                            append = &gtk::Revealer {
                                set_transition_type: gtk::RevealerTransitionType::SlideRight,
                                set_transition_duration: 180,
                                #[watch]
                                set_reveal_child: model.snapshot.network_is_online,

                                #[local_ref]
                                signal_widget -> gtk::Box {}
                            },

                        },
                    },

                    append = &gtk::Box {
                        set_orientation: Orientation::Vertical,
                        set_spacing: 0,
                        set_halign: Align::End,
                        set_hexpand: true,
                        add_css_class: "status-battery-card",

                        append = &gtk::Box {
                            set_orientation: Orientation::Horizontal,
                            set_spacing: 6,
                            set_halign: Align::End,
                            add_css_class: "status-indicator-row",

                            #[local]
                            battery_widget -> gtk::Box {
                            },
                        },
                    }
                },

                #[local]
                stage_shell -> gtk::Box {
                    set_vexpand: true,
                    set_hexpand: true,
                },

                append = &gtk::Box {
                    set_halign: Align::Center,
                    add_css_class: "bottom-safe-area",

                    append = &gtk::Box {
                        add_css_class: "home-indicator",
                    }
                },
            }
        }
    }

    fn init(
        _: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        install_css();

        let theme_mode = load_theme_mode();
        apply_theme(theme_mode);

        let snapshot = load_device_snapshot();
        let shell_state = load_shell_state(&snapshot);

        let home_screen = HomeScreen::builder().launch(snapshot.clone()).forward(
            sender.input_sender(),
            |output| match output {
                HomeScreenOutput::Navigate(destination) => ShellAppMsg::Navigate(destination),
            },
        );
        let apps_screen = AppsScreen::builder()
            .launch(shell_state.apps.clone())
            .detach();
        let ai_screen = AiScreen::builder().launch(shell_state.ai.clone()).detach();
        let settings_screen =
            SettingsScreen::builder()
                .launch(theme_mode)
                .forward(sender.input_sender(), |output| match output {
                    SettingsScreenOutput::ChooseTheme(theme_mode) => {
                        ShellAppMsg::SetTheme(theme_mode)
                    }
                });
        let bottom_nav = BottomNav::builder().launch(NavDestination::Home).forward(
            sender.input_sender(),
            |output| match output {
                BottomNavOutput::Navigate(destination) => ShellAppMsg::Navigate(destination),
            },
        );

        let home_screen_widget = wrap_page(home_screen.widget());
        let apps_screen_widget = wrap_page(apps_screen.widget());
        let ai_screen_widget = wrap_page(ai_screen.widget());
        let settings_screen_widget = wrap_page(settings_screen.widget());
        let bottom_nav_widget = bottom_nav.widget();
        let screen_carousel = adw::Carousel::builder()
            .interactive(true)
            .allow_mouse_drag(true)
            .allow_long_swipes(false)
            .allow_scroll_wheel(false)
            .spacing(18)
            .vexpand(true)
            .hexpand(true)
            .build();
        screen_carousel.append(&home_screen_widget);
        screen_carousel.append(&apps_screen_widget);
        screen_carousel.append(&ai_screen_widget);
        screen_carousel.append(&settings_screen_widget);
        screen_carousel.connect_page_changed({
            let input_sender = sender.input_sender().clone();
            move |_, index| {
                let _ = input_sender.send(ShellAppMsg::SwipeTo(nav_destination_for_index(index)));
            }
        });
        let stage_shell = gtk::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(18)
            .vexpand(true)
            .hexpand(true)
            .build();
        stage_shell.append(&screen_carousel);
        bottom_nav_widget.set_valign(Align::End);
        stage_shell.append(bottom_nav_widget);
        let model = ShellApp {
            active_nav: NavDestination::Home,
            apps_screen,
            ai_screen,
            bottom_nav,
            home_screen,
            layout_mode: LayoutMode::Compact,
            screen_carousel: screen_carousel.clone(),
            settings_screen,
            shell_state,
            snapshot,
            stage_shell: stage_shell.clone(),
            theme_mode,
        };
        let network_icon = network_kind_icon(model.snapshot.network_kind);
        let network_dot = signal_status_dot(model.snapshot.network_is_online);
        let signal_widget = signal_indicator(model.snapshot.signal_level);
        let battery_widget = battery_indicator(
            model.snapshot.battery_level,
            model.snapshot.battery_charging,
        );
        let widgets = view_output!();

        root.fullscreen();
        sync_window_to_screen_height(&root);
        apply_device_profile(&root);
        root.connect_map(|window| {
            window.fullscreen();
            sync_window_to_screen_height(window);
            apply_device_profile(window);
        });
        install_breakpoints(&widgets.main_window, sender.input_sender().clone());
        apply_window_theme(&widgets.main_window, resolve_theme(model.theme_mode));
        sync_shell_state(&model, &widgets);

        let input_sender = sender.input_sender().clone();
        glib::timeout_add_seconds_local(1, move || {
            let _ = input_sender.send(ShellAppMsg::Tick);
            glib::ControlFlow::Continue
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            ShellAppMsg::Tick => {
                self.snapshot = load_device_snapshot();
                self.shell_state = load_shell_state(&self.snapshot);
                self.home_screen
                    .emit(HomeScreenInput::SetSnapshot(self.snapshot.clone()));
                self.apps_screen.emit(
                    crate::components::screens::apps_screen::AppsScreenInput::SetState(
                        self.shell_state.apps.clone(),
                    ),
                );
                self.ai_screen.emit(
                    crate::components::screens::ai_screen::AiScreenInput::SetState(
                        self.shell_state.ai.clone(),
                    ),
                );
            }
            ShellAppMsg::Navigate(destination) => {
                self.active_nav = destination;
                self.bottom_nav.emit(BottomNavInput::SetActive(destination));
            }
            ShellAppMsg::SetLayoutMode(layout_mode) => {
                self.layout_mode = layout_mode;
            }
            ShellAppMsg::SwipeTo(destination) => {
                self.active_nav = destination;
                self.bottom_nav.emit(BottomNavInput::SetActive(destination));
            }
            ShellAppMsg::SetTheme(theme_mode) => {
                self.theme_mode = theme_mode;
                apply_theme(theme_mode);
                save_theme_mode(theme_mode);
                self.settings_screen
                    .emit(SettingsScreenInput::SetThemeMode(theme_mode));
            }
        }
    }

    fn post_view() {
        apply_window_theme(&widgets.main_window, resolve_theme(model.theme_mode));
        apply_layout_mode(model, widgets);
        sync_shell_state(model, widgets);
    }
}

fn sync_shell_state(model: &ShellApp, widgets: &ShellAppWidgets) {
    widgets
        .battery_widget
        .set_visible(model.snapshot.battery_level > 0);
    animate_battery_indicator(
        &widgets.battery_widget,
        model.snapshot.battery_level,
        model.snapshot.battery_charging,
    );
    replace_network_icon(&widgets.network_icon, model.snapshot.network_kind);
    update_signal_status_dot(&widgets.network_dot, model.snapshot.network_is_online);
    update_signal_indicator(&widgets.signal_widget, model.snapshot.signal_level);

    let target_index = nav_index(model.active_nav);
    let current_index = model.screen_carousel.position().round() as u32;
    if current_index != target_index {
        let page = model.screen_carousel.nth_page(target_index);
        model.screen_carousel.scroll_to(&page, true);
    }
}

fn wrap_page(widget: &gtk::Box) -> gtk::ScrolledWindow {
    let scrolled = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .propagate_natural_height(false)
        .propagate_natural_width(true)
        .build();
    scrolled.add_css_class("page-scroll");
    scrolled.set_child(Some(widget));
    scrolled
}

fn sync_window_to_screen_height(window: &adw::ApplicationWindow) {
    let Some(surface) = window.surface() else {
        return;
    };
    let Some(display) = gdk::Display::default() else {
        return;
    };
    let Some(monitor) = display.monitor_at_surface(&surface) else {
        return;
    };

    let monitor_height = monitor.geometry().height();
    let current_width = if window.default_width() > 0 {
        window.default_width()
    } else {
        460
    };

    window.set_default_height(monitor_height);
    window.set_height_request(monitor_height);
    window.set_default_size(current_width, monitor_height);
}

fn apply_device_profile(window: &adw::ApplicationWindow) {
    window.remove_css_class("device-rpi-portrait");

    let Some(surface) = window.surface() else {
        return;
    };
    let Some(display) = gdk::Display::default() else {
        return;
    };
    let Some(monitor) = display.monitor_at_surface(&surface) else {
        return;
    };

    let geometry = monitor.geometry();
    let width = geometry.width();
    let height = geometry.height();

    if height > width && width <= 900 {
        window.add_css_class("device-rpi-portrait");
    }
}

fn install_breakpoints(window: &adw::ApplicationWindow, input_sender: relm4::Sender<ShellAppMsg>) {
    let regular = adw::Breakpoint::new(
        adw::BreakpointCondition::parse("min-width: 560sp and max-width: 959sp")
            .expect("valid regular breakpoint"),
    );
    let expanded = adw::Breakpoint::new(
        adw::BreakpointCondition::parse("min-width: 960sp").expect("valid expanded breakpoint"),
    );

    window.add_breakpoint(regular.clone());
    window.add_breakpoint(expanded.clone());

    let regular_for_notify = regular.clone();
    let expanded_for_notify = expanded.clone();
    let notify_sender = input_sender.clone();
    window.connect_current_breakpoint_notify(move |window| {
        let layout_mode = match window.current_breakpoint() {
            Some(current) if current == expanded_for_notify => LayoutMode::Expanded,
            Some(current) if current == regular_for_notify => LayoutMode::Regular,
            _ => LayoutMode::Compact,
        };
        let _ = notify_sender.send(ShellAppMsg::SetLayoutMode(layout_mode));
    });

    let initial_layout = match window.current_breakpoint() {
        Some(current) if current == expanded => LayoutMode::Expanded,
        Some(current) if current == regular => LayoutMode::Regular,
        _ => LayoutMode::Compact,
    };
    let _ = input_sender.send(ShellAppMsg::SetLayoutMode(initial_layout));
}

fn apply_layout_mode(model: &ShellApp, widgets: &ShellAppWidgets) {
    widgets.main_window.remove_css_class("layout-compact");
    widgets.main_window.remove_css_class("layout-regular");
    widgets.main_window.remove_css_class("layout-expanded");

    let bottom_nav_widget = model.bottom_nav.widget();

    match model.layout_mode {
        LayoutMode::Compact => {
            widgets.main_window.add_css_class("layout-compact");
            model.stage_shell.set_orientation(Orientation::Vertical);
            model.stage_shell.set_spacing(14);
            model.screen_carousel.set_vexpand(true);
            model.screen_carousel.set_hexpand(true);
            bottom_nav_widget.set_orientation(Orientation::Horizontal);
            bottom_nav_widget.set_halign(Align::Center);
            bottom_nav_widget.set_valign(Align::End);
            bottom_nav_widget.set_hexpand(false);
            bottom_nav_widget.set_vexpand(false);
            bottom_nav_widget.set_width_request(-1);
            model.screen_carousel.set_allow_long_swipes(false);
        }
        LayoutMode::Regular => {
            widgets.main_window.add_css_class("layout-regular");
            model.stage_shell.set_orientation(Orientation::Vertical);
            model.stage_shell.set_spacing(18);
            model.screen_carousel.set_vexpand(true);
            model.screen_carousel.set_hexpand(true);
            bottom_nav_widget.set_orientation(Orientation::Horizontal);
            bottom_nav_widget.set_halign(Align::Center);
            bottom_nav_widget.set_valign(Align::End);
            bottom_nav_widget.set_hexpand(false);
            bottom_nav_widget.set_vexpand(false);
            bottom_nav_widget.set_width_request(-1);
            model.screen_carousel.set_allow_long_swipes(true);
        }
        LayoutMode::Expanded => {
            widgets.main_window.add_css_class("layout-expanded");
            model.stage_shell.set_orientation(Orientation::Vertical);
            model.stage_shell.set_spacing(22);
            model.screen_carousel.set_vexpand(true);
            model.screen_carousel.set_hexpand(true);
            bottom_nav_widget.set_orientation(Orientation::Horizontal);
            bottom_nav_widget.set_halign(Align::Center);
            bottom_nav_widget.set_valign(Align::End);
            bottom_nav_widget.set_hexpand(false);
            bottom_nav_widget.set_vexpand(false);
            bottom_nav_widget.set_width_request(-1);
            model.screen_carousel.set_allow_long_swipes(true);
        }
    }
}

fn replace_network_icon(widget: &gtk::Picture, kind: NetworkKind) {
    let file = match kind {
        NetworkKind::Wifi => "status-wifi",
        NetworkKind::Ethernet => "status-ethernet",
        NetworkKind::Lte => "status-lte",
        NetworkKind::Offline | NetworkKind::Unknown => "status-offline",
    };
    widget.set_filename(Some(format!(
        "{}/assets/icons/{file}.svg",
        env!("CARGO_MANIFEST_DIR")
    )));
}

fn nav_index(destination: NavDestination) -> u32 {
    match destination {
        NavDestination::Home => 0,
        NavDestination::Apps => 1,
        NavDestination::Ai => 2,
        NavDestination::Settings => 3,
    }
}

fn nav_destination_for_index(index: u32) -> NavDestination {
    match index {
        0 => NavDestination::Home,
        1 => NavDestination::Apps,
        2 => NavDestination::Ai,
        3 => NavDestination::Settings,
        _ => NavDestination::Home,
    }
}

fn apply_window_theme(window: &impl IsA<gtk::Widget>, theme: ResolvedTheme) {
    window.remove_css_class("theme-light");
    window.remove_css_class("theme-dark");

    match theme {
        ResolvedTheme::Light => window.add_css_class("theme-light"),
        ResolvedTheme::Dark => window.add_css_class("theme-dark"),
    }
}

fn install_css() {
    let provider = CssProvider::new();
    provider.load_from_data(
        "
        window {
            font-family: SF Pro Display, Inter, Sans;
        }

        .status-card,
        .hero-card,
        .nav-card,
        .section-header-card,
        .section-card,
        .theme-segmented,
        preferencesgroup.material-group {
            border-radius: 30px;
            padding: 18px;
        }

        .top-status-bar {
            padding: 0 2px;
            border-radius: 0;
        }

        .shell-root {
            min-height: 0;
        }

        .page-scroll,
        .page-scroll viewport {
            background: transparent;
        }

        .page-scroll {
            border: none;
        }

        .top-safe-area,
        .bottom-safe-area {
            min-height: 12px;
        }

        .top-safe-area {
            min-height: 0;
        }

        .top-safe-island {
            min-width: 112px;
            min-height: 18px;
            border-radius: 999px;
        }

        .home-indicator {
            min-width: 118px;
            min-height: 4px;
            border-radius: 999px;
        }

        window.device-rpi-portrait .shell-root {
            margin-top: 0;
            margin-bottom: 10px;
            margin-left: 4px;
            margin-right: 4px;
        }

        window.device-rpi-portrait .top-safe-area {
            min-height: 1px;
        }

        window.device-rpi-portrait .top-safe-island {
            min-width: 104px;
            min-height: 18px;
        }

        window.device-rpi-portrait .bottom-safe-area {
            min-height: 8px;
        }

        window.device-rpi-portrait .home-indicator {
            min-width: 128px;
            min-height: 4px;
        }

        .section-card,
        .metric-card,
        .section-header-card,
        preferencesgroup.material-group {
            box-shadow:
                0 20px 46px rgba(122, 145, 176, 0.14),
                inset 0 1px 0 rgba(255, 255, 255, 0.4);
        }

        .eyebrow-label {
            font-size: 12px;
            letter-spacing: 0.11em;
            text-transform: uppercase;
            font-weight: 700;
        }

        .clock-label {
            font-size: 48px;
            font-weight: 760;
            letter-spacing: -0.03em;
        }

        window.layout-compact .clock-label {
            font-size: 38px;
        }

        window.layout-expanded .clock-label {
            font-size: 54px;
        }

        .hero-title {
            font-size: 34px;
            font-weight: 770;
            letter-spacing: -0.04em;
        }

        window.layout-compact .hero-title {
            font-size: 28px;
        }

        window.layout-expanded .hero-title {
            font-size: 38px;
        }

        .hero-time {
            font-size: 60px;
            font-weight: 800;
            letter-spacing: -0.05em;
        }

        window.layout-compact .hero-time {
            font-size: 48px;
        }

        window.layout-expanded .hero-time {
            font-size: 68px;
        }

        .hero-time-separator {
            font-size: 56px;
            font-weight: 400;
        }

        window.layout-compact .hero-time-separator {
            font-size: 46px;
        }

        window.layout-expanded .hero-time-separator {
            font-size: 62px;
        }

        .hero-body,
        .section-card-subtitle,
        .section-row-subtitle,
        .tile-copy {
            font-size: 15px;
            line-height: 1.45;
        }

        .status-label,
        .detail-label,
        .section-row-title {
            font-size: 16px;
        }

        .status-label-compact {
            font-size: 13px;
            letter-spacing: 0.02em;
        }

        .detail-label-muted {
            font-size: 14px;
        }

        .section-card-title,
        .tile-title {
            font-size: 19px;
            font-weight: 720;
            letter-spacing: -0.02em;
        }

        .metric-value {
            font-size: 28px;
            font-weight: 760;
            letter-spacing: -0.03em;
        }

        .home-screen {
            padding-top: 2px;
            padding-bottom: 8px;
        }

        .home-clock-card {
            min-height: 258px;
            padding-top: 8px;
            padding-bottom: 0;
            background: transparent;
            border: none;
            box-shadow: none;
        }

        .home-clock-row {
            margin-bottom: 2px;
        }

        .home-clock-value,
        .home-clock-separator {
            margin-top: 0;
            font-size: 184px;
            line-height: 0.82;
            letter-spacing: -0.06em;
            font-weight: 760;
            color: #232634;
        }

        .home-clock-date {
            font-size: 32px;
            line-height: 1.2;
            margin-top: 2px;
            color: rgba(76, 81, 94, 0.82);
        }

        .home-launcher-sheet {
            padding-top: 0;
            padding-bottom: 4px;
        }

        .home-divider {
            min-height: 5px;
            border-radius: 999px;
            margin-left: 12px;
            margin-right: 12px;
        }

        flowbox.home-app-grid {
            background: transparent;
            margin-top: 2px;
            margin-bottom: 2px;
        }

        flowbox.home-app-grid flowboxchild {
            padding: 0;
            margin: 0;
            min-width: 104px;
        }

        .home-app-button {
            border: 0;
            background: transparent;
            padding: 4px 2px 10px;
            border-radius: 18px;
        }

        .home-app-icon-shell {
            border-radius: 0;
            border: none;
            box-shadow: none;
        }

        .home-app-label {
            font-size: 15px;
            font-weight: 520;
            letter-spacing: -0.01em;
        }

        .apps-screen {
            padding-top: 18px;
            padding-bottom: 24px;
        }

        flowbox.apps-icon-grid {
            background: transparent;
        }

        flowbox.apps-icon-grid flowboxchild {
            padding: 0;
            margin: 0;
            min-width: 92px;
        }

        .apps-icon-tile {
            min-width: 88px;
            padding: 4px 2px;
        }

        .apps-icon-shell {
            border-radius: 24px;
            padding: 18px;
        }

        .apps-icon-label {
            font-size: 14px;
            font-weight: 620;
            letter-spacing: -0.01em;
        }

        .launcher-tile {
            min-width: 0;
            min-height: 148px;
            padding: 18px;
            border-radius: 28px;
        }

        window.layout-compact .launcher-tile {
            min-height: 132px;
            padding: 14px;
            border-radius: 24px;
        }

        window.layout-compact .apps-screen {
            padding-top: 10px;
            padding-bottom: 18px;
        }

        window.layout-compact .home-clock-card {
            min-height: 228px;
            padding-top: 6px;
            padding-bottom: 0;
        }

        window.layout-compact .home-clock-value,
        window.layout-compact .home-clock-separator {
            font-size: 144px;
        }

        window.layout-compact .home-clock-date {
            font-size: 26px;
        }

        window.layout-compact flowbox.home-app-grid flowboxchild {
            min-width: 92px;
        }

        window.layout-compact .home-app-label {
            font-size: 13px;
        }

        window.layout-compact .apps-icon-shell {
            border-radius: 22px;
            padding: 16px;
        }

        window.layout-compact .apps-icon-label {
            font-size: 13px;
        }

        window.layout-expanded .launcher-tile {
            min-height: 164px;
            padding: 20px;
        }

        window.layout-expanded .home-clock-card {
            min-height: 286px;
        }

        window.layout-expanded .home-clock-value,
        window.layout-expanded .home-clock-separator {
            font-size: 208px;
        }

        window.layout-expanded .home-clock-date {
            font-size: 34px;
        }

        window.layout-expanded .apps-icon-shell {
            border-radius: 26px;
            padding: 20px;
        }

        flowbox.launcher-flowbox,
        flowbox.metric-flowbox,
        flowbox.adaptive-flowbox,
        flowbox.segmented-flowbox {
            background: transparent;
        }

        flowbox.launcher-flowbox flowboxchild,
        flowbox.metric-flowbox flowboxchild,
        flowbox.adaptive-flowbox flowboxchild,
        flowbox.segmented-flowbox flowboxchild {
            padding: 0;
            margin: 0;
        }

        flowbox.segmented-flowbox flowboxchild {
            min-width: 110px;
        }

        .app-icon-shell {
            border-radius: 18px;
            padding: 12px;
        }

        .app-icon {
            color: inherit;
        }

        .launcher-badge {
            font-size: 12px;
            font-weight: 700;
        }

        .nav-button {
            min-height: 50px;
            min-width: 64px;
            border-radius: 14px;
            font-size: 11px;
            font-weight: 650;
        }

        .nav-label {
            font-size: 11px;
            font-weight: 620;
            letter-spacing: -0.01em;
        }

        .nav-icon {
            min-width: 24px;
            min-height: 24px;
        }

        window.layout-compact .nav-card {
            padding: 8px;
            border-radius: 16px;
        }

        window.layout-compact .nav-button {
            min-height: 46px;
            min-width: 60px;
            border-radius: 12px;
            font-size: 10px;
        }

        window.layout-compact .nav-label {
            font-size: 10px;
        }

        window.layout-expanded .nav-card {
            padding: 8px;
            border-radius: 16px;
        }

        window.layout-expanded .nav-button {
            min-height: 48px;
            min-width: 62px;
            border-radius: 13px;
        }

        .theme-button {
            min-height: 52px;
            border-radius: 18px;
            font-size: 15px;
            font-weight: 650;
        }

        .thin-divider {
            opacity: 0.5;
        }

        .info-pill {
            border-radius: 999px;
            padding: 8px 14px;
            font-size: 13px;
            font-weight: 680;
        }

        .marker-pill {
            padding: 7px 12px;
        }

        .pill-dot {
            border-radius: 999px;
            background: currentColor;
            opacity: 0.9;
        }

        .pill-label {
            font-size: 12px;
            font-weight: 700;
        }

        .status-indicator-row {
            min-height: 18px;
        }

        .status-dot {
            border-radius: 999px;
        }

        .status-dot-online {
            background: #56d68a;
            box-shadow: 0 0 0 3px rgba(86, 214, 138, 0.14);
        }

        .status-dot-offline {
            background: rgba(141, 154, 175, 0.7);
            box-shadow: 0 0 0 3px rgba(141, 154, 175, 0.12);
        }

        .status-network-card,
        .status-battery-card {
            min-width: 0;
        }

        .status-icon {
            color: inherit;
        }

        .status-network-label {
            font-size: 12px;
            letter-spacing: 0.01em;
        }

        .status-network-detail {
            font-size: 11px;
            opacity: 0.78;
        }

        .status-date-label {
            font-size: 11px;
        }

        .signal-bar {
            border-radius: 999px;
            background: rgba(130, 153, 184, 0.22);
        }

        .signal-bar-active {
            background: linear-gradient(180deg, #7dc4ff 0%, #4d8dff 100%);
        }

        .battery-shell {
            padding: 2px;
            border-radius: 5px;
            border: 1px solid rgba(133, 153, 187, 0.42);
            background: rgba(255, 255, 255, 0.24);
        }

        .battery-charge {
            font-size: 13px;
            font-weight: 800;
        }

        .battery-fill-critical {
            background: linear-gradient(90deg, #ff6d7a 0%, #ff4f57 100%);
        }

        .battery-fill {
            min-height: 10px;
            border-radius: 3px;
        }

        .battery-fill-low {
            background: linear-gradient(90deg, #ff9f6f 0%, #ff655e 100%);
        }

        .battery-fill-mid {
            background: linear-gradient(90deg, #ffd86c 0%, #ffb65e 100%);
        }

        .battery-fill-high {
            background: linear-gradient(90deg, #51d7ff 0%, #7d8bff 100%);
        }

        .battery-fill-charging {
            background: linear-gradient(90deg, #69e3a4 0%, #3ccf7d 100%);
        }

        .battery-cap {
            border-radius: 0 2px 2px 0;
            background: rgba(133, 153, 187, 0.52);
        }

        .launcher-badge-cyan,
        .accent-cyan {
            color: #4a8ee5;
        }

        .launcher-badge-violet,
        .accent-violet {
            color: #7f74d6;
        }

        .accent-success {
            color: #39a96b;
        }

        .screen-scroll {
            background: transparent;
            border: none;
        }

        .screen-scroll viewport {
            background: transparent;
        }

        window.theme-light {
            background: #f4f6fb;
            color: #0f1728;
        }

        window.theme-light .shell-root {
            background:
                radial-gradient(circle at top left, rgba(150, 220, 255, 0.55), transparent 28%),
                radial-gradient(circle at top right, rgba(214, 205, 255, 0.44), transparent 26%),
                radial-gradient(circle at bottom center, rgba(255, 255, 255, 0.65), transparent 20%),
                linear-gradient(180deg, #f9fbff 0%, #edf3fa 58%, #e5edf6 100%);
        }

        window.theme-light .status-card,
        window.theme-light .hero-card,
        window.theme-light .nav-card,
        window.theme-light .section-header-card,
        window.theme-light .section-card,
        window.theme-light .theme-segmented,
        window.theme-light preferencesgroup.material-group {
            background: rgba(255, 255, 255, 0.52);
            border: 1px solid rgba(255, 255, 255, 0.72);
        }

        window.theme-light .top-safe-island {
            background: rgba(12, 18, 30, 0.94);
            box-shadow: 0 10px 22px rgba(121, 140, 170, 0.18);
        }

        window.theme-light .home-indicator {
            background: rgba(20, 29, 44, 0.72);
        }

        window.theme-light .eyebrow-label,
        window.theme-light .launcher-badge {
            color: #608fc8;
        }

        window.theme-light .clock-label,
        window.theme-light .hero-title,
        window.theme-light .hero-time,
        window.theme-light .section-card-title,
        window.theme-light .tile-title,
        window.theme-light .metric-value,
        window.theme-light .section-row-title {
            color: #102033;
        }

        window.theme-light .hero-time-separator {
            color: rgba(102, 146, 205, 0.72);
        }

        window.theme-light .hero-body,
        window.theme-light .section-card-subtitle,
        window.theme-light .section-row-subtitle,
        window.theme-light .tile-copy,
        window.theme-light .status-label,
        window.theme-light .detail-label {
            color: rgba(53, 74, 102, 0.82);
        }

        window.theme-light .detail-label-muted {
            color: rgba(98, 116, 142, 0.72);
        }

        window.theme-light .launcher-tile {
            background:
                linear-gradient(180deg, rgba(255, 255, 255, 0.90) 0%, rgba(243, 247, 255, 0.80) 100%);
            border: 1px solid rgba(201, 218, 244, 0.84);
            box-shadow:
                0 14px 28px rgba(162, 183, 210, 0.16),
                inset 0 1px 0 rgba(255, 255, 255, 0.95);
        }

        window.theme-light .nav-button,
        window.theme-light .theme-button {
            background: rgba(255, 255, 255, 0.36);
            border: 1px solid rgba(201, 219, 244, 0.82);
            color: #48678f;
            box-shadow:
                inset 0 1px 0 rgba(255, 255, 255, 0.95),
                0 8px 20px rgba(160, 180, 205, 0.1);
        }

        window.theme-light .nav-button image {
            color: #6e9bd2;
        }

        window.theme-light .nav-button-active,
        window.theme-light .theme-button-active {
            background:
                linear-gradient(180deg, rgba(155, 214, 255, 0.36) 0%, rgba(203, 215, 255, 0.34) 100%);
            border: 1px solid rgba(150, 194, 245, 0.92);
            color: #173252;
            box-shadow:
                0 0 0 1px rgba(166, 205, 249, 0.58),
                0 0 22px rgba(120, 180, 245, 0.28),
                0 12px 28px rgba(120, 170, 228, 0.24),
                inset 0 1px 0 rgba(255, 255, 255, 0.96);
        }

        window.theme-light .nav-button-active image {
            color: #2f73c5;
        }

        window.theme-light .info-pill {
            background: rgba(255, 255, 255, 0.72);
            border: 1px solid rgba(194, 214, 242, 0.88);
            color: #33557f;
        }

        window.theme-light .battery-charge {
            color: #2ea66b;
        }

        window.theme-light .app-icon-shell {
            background: linear-gradient(180deg, rgba(255, 255, 255, 0.94) 0%, rgba(238, 244, 255, 0.82) 100%);
            border: 1px solid rgba(205, 221, 244, 0.9);
            box-shadow:
                inset 0 1px 0 rgba(255, 255, 255, 0.96),
                0 10px 24px rgba(157, 177, 206, 0.16);
        }

        window.theme-light .apps-icon-shell {
            background: linear-gradient(180deg, rgba(255, 255, 255, 0.98) 0%, rgba(243, 248, 255, 0.9) 100%);
            border: 1px solid rgba(198, 217, 244, 0.88);
            box-shadow:
                inset 0 1px 0 rgba(255, 255, 255, 0.98),
                0 12px 24px rgba(168, 188, 214, 0.16);
        }

        window.theme-light .apps-icon-label {
            color: #1f3f67;
        }

        window.theme-light .home-app-label {
            color: #253243;
        }

        window.theme-light .home-divider {
            background:
                radial-gradient(ellipse at center, rgba(137, 195, 255, 0.28) 0%, rgba(137, 195, 255, 0.16) 36%, rgba(137, 195, 255, 0.03) 72%, rgba(137, 195, 255, 0) 100%),
                linear-gradient(90deg, rgba(178, 214, 255, 0.08) 0%, rgba(124, 198, 255, 0.78) 20%, rgba(146, 163, 255, 0.74) 50%, rgba(255, 202, 166, 0.68) 80%, rgba(255, 230, 208, 0.08) 100%);
            box-shadow:
                0 1px 0 rgba(255, 255, 255, 0.88),
                0 2px 10px rgba(121, 176, 236, 0.16),
                0 0 18px rgba(178, 202, 255, 0.12);
        }

        window.theme-light row.material-row {
            background: rgba(255, 255, 255, 0.18);
        }

        window.theme-light .thin-divider {
            color: rgba(149, 174, 206, 0.35);
        }

        window.theme-dark {
            background: #06101a;
            color: #edf4ff;
        }

        window.theme-dark .shell-root {
            background:
                radial-gradient(circle at top left, rgba(78, 157, 255, 0.22), transparent 28%),
                radial-gradient(circle at top right, rgba(140, 118, 255, 0.18), transparent 24%),
                linear-gradient(180deg, #0b1420 0%, #09111a 55%, #060b11 100%);
        }

        window.theme-dark .status-card,
        window.theme-dark .hero-card,
        window.theme-dark .nav-card,
        window.theme-dark .section-header-card,
        window.theme-dark .section-card,
        window.theme-dark .theme-segmented,
        window.theme-dark preferencesgroup.material-group {
            background: rgba(14, 24, 38, 0.66);
            border: 1px solid rgba(126, 157, 201, 0.16);
        }

        window.theme-dark .top-safe-island {
            background: rgba(4, 9, 16, 0.96);
            box-shadow: 0 12px 28px rgba(0, 0, 0, 0.35);
        }

        window.theme-dark .home-indicator {
            background: rgba(231, 239, 249, 0.82);
        }

        window.theme-dark .eyebrow-label,
        window.theme-dark .launcher-badge {
            color: #7dbaff;
        }

        window.theme-dark .clock-label,
        window.theme-dark .hero-title,
        window.theme-dark .hero-time,
        window.theme-dark .section-card-title,
        window.theme-dark .tile-title,
        window.theme-dark .metric-value,
        window.theme-dark .section-row-title {
            color: #f4f8ff;
        }

        window.theme-dark .hero-time-separator {
            color: rgba(143, 191, 255, 0.74);
        }

        window.theme-dark .hero-body,
        window.theme-dark .section-card-subtitle,
        window.theme-dark .section-row-subtitle,
        window.theme-dark .tile-copy,
        window.theme-dark .status-label,
        window.theme-dark .detail-label {
            color: rgba(219, 230, 245, 0.78);
        }

        window.theme-dark .detail-label-muted {
            color: rgba(176, 193, 217, 0.7);
        }

        window.theme-dark .launcher-tile {
            background:
                linear-gradient(180deg, rgba(24, 38, 58, 0.88) 0%, rgba(17, 29, 45, 0.84) 100%);
            border: 1px solid rgba(104, 133, 173, 0.2);
            box-shadow:
                0 16px 30px rgba(0, 0, 0, 0.24),
                inset 0 1px 0 rgba(255, 255, 255, 0.04);
        }

        window.theme-dark .nav-button,
        window.theme-dark .theme-button {
            background: rgba(19, 31, 49, 0.78);
            border: 1px solid rgba(96, 125, 164, 0.24);
            color: #c9dcf8;
        }

        window.theme-dark .nav-button image {
            color: #8ec4ff;
        }

        window.theme-dark .nav-button-active,
        window.theme-dark .theme-button-active {
            background:
                linear-gradient(180deg, rgba(53, 122, 213, 0.42) 0%, rgba(106, 86, 207, 0.34) 100%);
            border: 1px solid rgba(119, 170, 240, 0.48);
            color: #ffffff;
            box-shadow:
                0 0 0 1px rgba(132, 184, 249, 0.38),
                0 0 24px rgba(59, 141, 245, 0.32),
                0 12px 28px rgba(27, 85, 157, 0.42);
        }

        window.theme-dark .nav-button-active image {
            color: #d7ebff;
        }

        window.theme-dark .info-pill {
            background: rgba(28, 43, 64, 0.86);
            border: 1px solid rgba(98, 126, 165, 0.24);
            color: #d0e2ff;
        }

        window.theme-dark .battery-charge {
            color: #8ff0b8;
        }

        window.theme-dark .app-icon-shell {
            background: linear-gradient(180deg, rgba(27, 41, 62, 0.92) 0%, rgba(18, 31, 47, 0.88) 100%);
            border: 1px solid rgba(102, 130, 171, 0.22);
            box-shadow:
                inset 0 1px 0 rgba(255, 255, 255, 0.05),
                0 12px 24px rgba(0, 0, 0, 0.24);
        }

        window.theme-dark .apps-icon-shell {
            background: linear-gradient(180deg, rgba(22, 37, 58, 0.96) 0%, rgba(17, 28, 44, 0.9) 100%);
            border: 1px solid rgba(95, 125, 168, 0.28);
            box-shadow:
                inset 0 1px 0 rgba(255, 255, 255, 0.05),
                0 14px 26px rgba(0, 0, 0, 0.28);
        }

        window.theme-dark .apps-icon-label {
            color: #edf5ff;
        }

        window.theme-dark .home-app-label {
            color: #edf5ff;
        }

        window.theme-dark .home-divider {
            background:
                radial-gradient(ellipse at center, rgba(96, 157, 255, 0.22) 0%, rgba(96, 157, 255, 0.12) 34%, rgba(96, 157, 255, 0.02) 72%, rgba(96, 157, 255, 0) 100%),
                linear-gradient(90deg, rgba(76, 170, 255, 0.06) 0%, rgba(84, 179, 255, 0.7) 20%, rgba(131, 108, 241, 0.54) 54%, rgba(255, 162, 109, 0.4) 82%, rgba(255, 206, 166, 0.04) 100%);
            box-shadow:
                0 1px 0 rgba(255, 255, 255, 0.06),
                0 2px 12px rgba(44, 93, 158, 0.2),
                0 0 18px rgba(89, 122, 205, 0.14);
        }

        window.theme-dark row.material-row {
            background: rgba(255, 255, 255, 0.02);
        }

        window.theme-dark .thin-divider {
            color: rgba(119, 146, 184, 0.22);
        }
        ",
    );

    if let Some(display) = gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}
