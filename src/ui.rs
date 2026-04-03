use crate::device::NetworkKind;
use relm4::gtk;
use relm4::gtk::glib;
use relm4::gtk::prelude::*;
use relm4::gtk::{Align, Orientation};
use std::path::PathBuf;

pub fn nav_icon(name: &str) -> gtk::Picture {
    let picture = picture_icon(name, 42);
    picture.add_css_class("nav-icon");
    picture
}

pub fn status_icon(name: &str, size: i32) -> gtk::Picture {
    let picture = picture_icon(name, size);
    picture.add_css_class("status-icon");
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
