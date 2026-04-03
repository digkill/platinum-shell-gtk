#![allow(dead_code)]

mod ai;
mod apps_catalog;
mod calendar;
mod call;
mod clock;
mod contacts;
mod message;
mod platinum_one;
mod relay;
mod settings;

use crate::app_sdk::{AppContext, AppRegistry};
use relm4::gtk;
use relm4::gtk::prelude::*;

pub fn app_registry() -> AppRegistry {
    AppRegistry::new(vec![
        Box::new(calendar::CalendarApp),
        Box::new(clock::ClockApp),
        Box::new(contacts::ContactsApp),
        Box::new(apps_catalog::AppsCatalogApp),
        Box::new(ai::AiApp),
        Box::new(call::CallApp),
        Box::new(message::MessageApp),
        Box::new(settings::SettingsApp),
        Box::new(relay::RelayApp),
        Box::new(platinum_one::PlatinumOneApp),
    ])
}

fn placeholder_root(title: &str, description: &str) -> relm4::gtk::Widget {
    let root = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(24)
        .margin_end(24)
        .build();

    let title_label = gtk::Label::new(Some(title));
    title_label.set_halign(gtk::Align::Start);
    title_label.add_css_class("hero-title");

    let description_label = gtk::Label::new(Some(description));
    description_label.set_halign(gtk::Align::Start);
    description_label.set_wrap(true);
    description_label.add_css_class("hero-body");

    root.append(&title_label);
    root.append(&description_label);
    root.upcast()
}

fn app_surface_root() -> gtk::Box {
    gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(18)
        .margin_top(20)
        .margin_bottom(24)
        .margin_start(18)
        .margin_end(18)
        .build()
}

fn app_hero(title: &str, subtitle: &str) -> gtk::Box {
    let hero = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(6)
        .build();
    hero.add_css_class("section-header-card");

    let title_label = gtk::Label::new(Some(title));
    title_label.set_halign(gtk::Align::Start);
    title_label.add_css_class("hero-title");

    let subtitle_label = gtk::Label::new(Some(subtitle));
    subtitle_label.set_halign(gtk::Align::Start);
    subtitle_label.set_wrap(true);
    subtitle_label.add_css_class("hero-body");

    hero.append(&title_label);
    hero.append(&subtitle_label);
    hero
}

fn app_card(title: &str, subtitle: &str) -> gtk::Box {
    let card = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(10)
        .build();
    card.add_css_class("section-card");

    let title_label = gtk::Label::new(Some(title));
    title_label.set_halign(gtk::Align::Start);
    title_label.add_css_class("section-card-title");

    let subtitle_label = gtk::Label::new(Some(subtitle));
    subtitle_label.set_halign(gtk::Align::Start);
    subtitle_label.set_wrap(true);
    subtitle_label.add_css_class("section-card-subtitle");

    card.append(&title_label);
    card.append(&subtitle_label);
    card
}

fn app_row(title: &str, subtitle: &str, trailing: &str) -> gtk::Box {
    let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .build();
    row.add_css_class("section-card");

    let copy = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();

    let title_label = gtk::Label::new(Some(title));
    title_label.set_halign(gtk::Align::Start);
    title_label.add_css_class("section-row-title");

    let subtitle_label = gtk::Label::new(Some(subtitle));
    subtitle_label.set_halign(gtk::Align::Start);
    subtitle_label.set_wrap(true);
    subtitle_label.add_css_class("section-row-subtitle");

    let trailing_label = gtk::Label::new(Some(trailing));
    trailing_label.set_halign(gtk::Align::End);
    trailing_label.set_valign(gtk::Align::Center);
    trailing_label.add_css_class("eyebrow-label");

    copy.append(&title_label);
    copy.append(&subtitle_label);
    row.append(&copy);
    row.append(&trailing_label);
    row
}

fn pill(label: &str) -> gtk::Label {
    let pill = gtk::Label::new(Some(label));
    pill.add_css_class("info-pill");
    pill
}

fn action_button(label: &str) -> gtk::Button {
    let button = gtk::Button::with_label(label);
    button.add_css_class("theme-button");
    button
}

pub fn register_builtins(context: &AppContext) {
    app_registry().register_all(context);
}
