use crate::app_sdk::{AppContext, AppManifest, AppPermission, LauncherApp};
use relm4::gtk;
use relm4::gtk::prelude::*;

pub struct ClockApp;

impl LauncherApp for ClockApp {
    fn manifest(&self) -> AppManifest {
        AppManifest {
            id: "clock",
            title: "Clock",
            icon_name: "clock",
            description: "World clock, alarms, and timer surface.",
        }
    }

    fn permissions(&self) -> &'static [AppPermission] {
        &[AppPermission::Clock]
    }

    fn build_root(&self, context: &AppContext) -> gtk::Widget {
        let root = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .margin_top(24)
            .margin_bottom(24)
            .margin_start(24)
            .margin_end(24)
            .build();

        let time_label = gtk::Label::new(Some(&context.snapshot.time_label));
        time_label.set_halign(gtk::Align::Start);
        time_label.add_css_class("hero-time");

        let date_label = gtk::Label::new(Some(&context.snapshot.date_label));
        date_label.set_halign(gtk::Align::Start);
        date_label.add_css_class("hero-body");

        root.append(&time_label);
        root.append(&date_label);
        root.upcast()
    }
}
