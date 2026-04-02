use crate::device::NetworkKind;
use adw::prelude::*;
use relm4::gtk;
use relm4::gtk::glib;
use relm4::gtk::{Align, Orientation};
use std::path::PathBuf;

pub fn surface_card() -> gtk::Box {
    let card = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(14)
        .build();
    card.add_css_class("section-card");
    card
}

pub fn surface_header(title: &str, subtitle: &str, badge: &str) -> gtk::Box {
    let header = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(12)
        .build();
    header.add_css_class("section-header-card");

    let copy = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .hexpand(true)
        .build();

    let title_label = gtk::Label::new(Some(title));
    title_label.set_halign(Align::Start);
    title_label.add_css_class("hero-title");

    let subtitle_label = gtk::Label::new(Some(subtitle));
    subtitle_label.set_halign(Align::Start);
    subtitle_label.set_wrap(true);
    subtitle_label.add_css_class("hero-body");

    let pill = info_pill(badge);

    copy.append(&title_label);
    copy.append(&subtitle_label);
    header.append(&copy);
    header.append(&pill);
    header
}

pub fn titled_card(title: &str, subtitle: &str) -> gtk::Box {
    let card = surface_card();

    let title_label = gtk::Label::new(Some(title));
    title_label.set_halign(Align::Start);
    title_label.add_css_class("section-card-title");

    let subtitle_label = gtk::Label::new(Some(subtitle));
    subtitle_label.set_halign(Align::Start);
    subtitle_label.set_wrap(true);
    subtitle_label.add_css_class("section-card-subtitle");

    card.append(&title_label);
    card.append(&subtitle_label);
    card
}

pub fn divider() -> gtk::Separator {
    let divider = gtk::Separator::new(Orientation::Horizontal);
    divider.add_css_class("thin-divider");
    divider
}

pub fn info_pill(label: &str) -> gtk::Label {
    let pill = gtk::Label::new(Some(label));
    pill.add_css_class("info-pill");
    pill
}

pub fn marker_pill(label: &str, tone_class: &str) -> gtk::Box {
    let pill = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(8)
        .halign(Align::Start)
        .valign(Align::Center)
        .build();
    pill.add_css_class("info-pill");
    pill.add_css_class("marker-pill");
    pill.add_css_class(tone_class);

    let dot = gtk::Box::builder()
        .width_request(8)
        .height_request(8)
        .valign(Align::Center)
        .build();
    dot.add_css_class("pill-dot");

    let label_widget = gtk::Label::new(Some(label));
    label_widget.add_css_class("pill-label");

    pill.append(&dot);
    pill.append(&label_widget);
    pill
}

pub fn preference_group(title: &str, description: &str) -> adw::PreferencesGroup {
    adw::PreferencesGroup::builder()
        .title(title)
        .description(description)
        .build()
}

pub fn action_row(title: &str, subtitle: &str, trailing: &str) -> adw::ActionRow {
    let row = adw::ActionRow::builder()
        .title(title)
        .subtitle(subtitle)
        .build();
    row.add_css_class("material-row");
    row.add_suffix(&marker_pill(trailing, tone_class_for_label(trailing)));
    row
}

pub fn nav_icon(name: &str) -> gtk::Picture {
    let picture = picture_icon(name, 24);
    picture.add_css_class("nav-icon");
    picture
}

pub fn status_icon(name: &str, size: i32) -> gtk::Picture {
    let picture = picture_icon(name, size);
    picture.add_css_class("status-icon");
    picture
}

pub fn app_icon(name: &str) -> gtk::Picture {
    let picture = picture_icon(name, 24);
    picture.add_css_class("app-icon");
    picture
}

pub fn picture_icon(name: &str, size: i32) -> gtk::Picture {
    let picture = gtk::Picture::for_filename(asset_path(&format!("assets/icons/{name}.svg")));
    picture.set_size_request(size, size);
    picture.set_can_shrink(true);
    picture
}

pub fn network_kind_icon(kind: NetworkKind) -> gtk::Picture {
    let icon_name = match kind {
        NetworkKind::Wifi => "status-wifi",
        NetworkKind::Ethernet => "status-ethernet",
        NetworkKind::Lte => "status-lte",
        NetworkKind::Offline | NetworkKind::Unknown => "status-offline",
    };
    status_icon(icon_name, 16)
}

pub fn signal_indicator(level: u8) -> gtk::Box {
    let clamped_level = level.min(5);
    let row = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(3)
        .valign(Align::Center)
        .build();
    row.add_css_class("signal-indicator");

    for index in 0..5 {
        let bar = gtk::Box::builder()
            .width_request(4)
            .height_request(6 + (index * 3))
            .valign(Align::End)
            .build();
        bar.add_css_class("signal-bar");

        if index < i32::from(clamped_level) {
            bar.add_css_class("signal-bar-active");
        }

        row.append(&bar);
    }

    row
}

pub fn signal_status_dot(is_online: bool) -> gtk::Box {
    let dot = gtk::Box::builder()
        .width_request(8)
        .height_request(8)
        .valign(Align::Center)
        .build();
    dot.add_css_class("status-dot");
    if is_online {
        dot.add_css_class("status-dot-online");
    } else {
        dot.add_css_class("status-dot-offline");
    }
    dot
}

pub fn update_signal_status_dot(dot: &gtk::Box, is_online: bool) {
    dot.remove_css_class("status-dot-online");
    dot.remove_css_class("status-dot-offline");
    if is_online {
        dot.add_css_class("status-dot-online");
    } else {
        dot.add_css_class("status-dot-offline");
    }
}

pub fn update_signal_indicator(row: &gtk::Box, level: u8) {
    let clamped_level = level.min(5);
    let mut index = 0_u8;
    let mut child = row.first_child();

    while let Some(widget) = child {
        let next = widget.next_sibling();

        if let Ok(bar) = widget.downcast::<gtk::Box>() {
            if index < clamped_level {
                bar.add_css_class("signal-bar-active");
            } else {
                bar.remove_css_class("signal-bar-active");
            }
        }

        index = index.saturating_add(1);
        child = next;
    }
}

pub fn battery_indicator(level: u8, charging: bool) -> gtk::Box {
    let shell = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(4)
        .valign(Align::Center)
        .build();

    let overlay = gtk::Overlay::builder()
        .width_request(34)
        .height_request(16)
        .build();

    let battery = gtk::Box::builder()
        .orientation(Orientation::Horizontal)
        .spacing(0)
        .width_request(34)
        .height_request(16)
        .build();
    battery.add_css_class("battery-shell");

    let fill = gtk::Box::builder().build();
    fill.add_css_class("battery-fill");
    fill.set_size_request(((level.clamp(0, 100) as i32 * 26) / 100).max(4), 10);

    let charge = gtk::Label::new(Some("⚡"));
    charge.add_css_class("battery-charge");
    charge.set_halign(Align::Center);
    charge.set_valign(Align::Center);
    charge.set_visible(charging);

    let cap = gtk::Box::builder()
        .width_request(3)
        .height_request(7)
        .valign(Align::Center)
        .build();
    cap.add_css_class("battery-cap");

    battery.append(&fill);
    overlay.set_child(Some(&battery));
    overlay.add_overlay(&charge);

    shell.append(&overlay);
    shell.append(&cap);

    update_battery_indicator(&shell, level, charging);
    shell
}

pub fn update_battery_indicator(shell: &gtk::Box, level: u8, charging: bool) {
    let clamped_level = level.min(100);
    let fill_width = ((i32::from(clamped_level) * 26) / 100).max(4);
    let fill_class = battery_fill_class(clamped_level, charging);

    if let Some(overlay) = shell.first_child().and_downcast::<gtk::Overlay>() {
        if let Some(charge) = overlay.last_child().and_downcast::<gtk::Label>() {
            charge.set_visible(charging);
        }
    }

    if let Some(battery) = shell
        .first_child()
        .and_downcast::<gtk::Overlay>()
        .and_then(|overlay| overlay.child())
        .and_downcast::<gtk::Box>()
    {
        if let Some(fill) = battery.first_child().and_downcast::<gtk::Box>() {
            fill.set_size_request(fill_width, 10);
            fill.remove_css_class("battery-fill-critical");
            fill.remove_css_class("battery-fill-low");
            fill.remove_css_class("battery-fill-mid");
            fill.remove_css_class("battery-fill-high");
            fill.remove_css_class("battery-fill-charging");
            fill.add_css_class(fill_class);
        }
    }
}

pub fn animate_battery_indicator(shell: &gtk::Box, level: u8, charging: bool) {
    let target_level = level.min(100);
    let target_width = ((i32::from(target_level) * 26) / 100).max(4);
    let shell = shell.clone();

    glib::timeout_add_local(std::time::Duration::from_millis(16), move || {
        if let Some(overlay) = shell.first_child().and_downcast::<gtk::Overlay>() {
            if let Some(charge) = overlay.last_child().and_downcast::<gtk::Label>() {
                charge.set_visible(charging);
            }
        }

        let Some(battery) = shell
            .first_child()
            .and_downcast::<gtk::Overlay>()
            .and_then(|overlay| overlay.child())
            .and_downcast::<gtk::Box>()
        else {
            return glib::ControlFlow::Break;
        };
        let Some(fill) = battery.first_child().and_downcast::<gtk::Box>() else {
            return glib::ControlFlow::Break;
        };

        let current_width = fill.width_request().max(4);
        if current_width == target_width {
            update_battery_indicator(&shell, target_level, charging);
            return glib::ControlFlow::Break;
        }

        let step = if current_width < target_width { 1 } else { -1 };
        let next_width = current_width + step;
        fill.set_size_request(next_width, 10);

        if next_width == target_width {
            update_battery_indicator(&shell, target_level, charging);
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}

fn asset_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn tone_class_for_label(label: &str) -> &'static str {
    let normalized = label.to_ascii_lowercase();
    if normalized.contains("ready")
        || normalized.contains("on")
        || normalized.contains("live")
        || normalized.contains("online")
        || normalized.contains("synced")
    {
        "accent-success"
    } else if normalized.contains("queue")
        || normalized.contains("standby")
        || normalized.contains("off")
    {
        "accent-violet"
    } else {
        "accent-cyan"
    }
}

fn battery_fill_class(level: u8, charging: bool) -> &'static str {
    if charging {
        "battery-fill-charging"
    } else {
        match level {
            0..=10 => "battery-fill-critical",
            11..=20 => "battery-fill-low",
            21..=55 => "battery-fill-mid",
            _ => "battery-fill-high",
        }
    }
}
