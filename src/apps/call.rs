use crate::app_sdk::{AppContext, AppLaunchPayload, AppManifest, AppPermission, LauncherApp};
use relm4::gtk;
use relm4::gtk::glib;
use relm4::gtk::prelude::*;

pub struct CallApp;

impl LauncherApp for CallApp {
    fn manifest(&self) -> AppManifest {
        AppManifest {
            id: "call",
            title: "Call",
            icon_name: "call",
            description: "Dialer and recent voice sessions for the handheld communicator.",
        }
    }

    fn permissions(&self) -> &'static [AppPermission] {
        &[AppPermission::Contacts, AppPermission::Network]
    }

    fn build_root(&self, context: &AppContext) -> gtk::Widget {
        let root = super::app_surface_root();
        root.add_css_class("call-app-root");
        root.set_spacing(12);
        root.append(&super::app_hero(
            "Call",
            "Fast dial, recent sessions, and one-thumb voice actions for the handheld shell.",
        ));

        let top_actions = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10)
            .halign(gtk::Align::Start)
            .build();
        let contacts_button = super::action_button("Contacts");
        let contacts_nav = context.navigator.clone();
        contacts_button.connect_clicked(move |_| {
            contacts_nav.open("contacts", None);
        });
        top_actions.append(&contacts_button);
        root.append(&top_actions);

        let launched_contact = match &context.launch_payload {
            Some(AppLaunchPayload::Contact(contact)) => {
                Some((contact.name.clone(), contact.phone.clone()))
            }
            None => None,
        };

        let dialer_card = super::app_card("Dialer", "Live number pad with quick recent recall.");
        dialer_card.add_css_class("call-dialer-card");
        let display_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .build();

        let number_entry = gtk::Entry::builder()
            .placeholder_text("Enter number")
            .hexpand(true)
            .build();
        gtk::prelude::EditableExt::set_alignment(&number_entry, 0.5);
        number_entry.add_css_class("hero-title");

        if let Some(AppLaunchPayload::Contact(contact)) = &context.launch_payload {
            number_entry.set_text(&contact.phone);
        }

        let initial_status = match &context.launch_payload {
            Some(AppLaunchPayload::Contact(contact)) => format!("Loaded {}", contact.name),
            None if context.snapshot.network_is_online => String::from("Ready to place a call"),
            None => String::from("No network available"),
        };
        let status_label = gtk::Label::new(Some(&initial_status));
        status_label.set_halign(gtk::Align::Center);
        status_label.add_css_class("section-card-subtitle");
        let status_revealer = gtk::Revealer::builder()
            .transition_type(gtk::RevealerTransitionType::Crossfade)
            .transition_duration(160)
            .reveal_child(true)
            .build();
        status_revealer.set_child(Some(&status_label));

        display_box.append(&number_entry);
        display_box.append(&status_revealer);
        dialer_card.append(&display_box);

        let keypad = gtk::Grid::builder()
            .row_spacing(8)
            .column_spacing(8)
            .halign(gtk::Align::Center)
            .build();
        let keys = [
            ("1", 0, 0),
            ("2", 1, 0),
            ("3", 2, 0),
            ("4", 0, 1),
            ("5", 1, 1),
            ("6", 2, 1),
            ("7", 0, 2),
            ("8", 1, 2),
            ("9", 2, 2),
            ("*", 0, 3),
            ("0", 1, 3),
            ("#", 2, 3),
        ];
        for (label, column, row) in keys {
            keypad.attach(
                &dial_key(label, &number_entry, &status_label, &status_revealer),
                column,
                row,
                1,
                1,
            );
        }
        dialer_card.append(&keypad);

        let controls = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk::Align::Center)
            .build();
        controls.append(&erase_button(
            &number_entry,
            &status_label,
            &status_revealer,
        ));
        controls.append(&call_button(
            &number_entry,
            &status_label,
            &status_revealer,
            context.snapshot.network_is_online,
            launched_contact.clone(),
            context.store.clone(),
        ));
        controls.append(&clear_button(
            &number_entry,
            &status_label,
            &status_revealer,
        ));
        dialer_card.append(&controls);
        root.append(&dialer_card);

        let recents_card = super::app_card(
            "Recent Calls",
            "Tap any entry to push the number back into the dialer.",
        );
        recents_card.add_css_class("call-recents-card");
        let recents = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .build();
        for contact in context.store.borrow().recents_snapshot() {
            recents.append(&recent_row(
                &contact.name,
                &contact.phone,
                &contact.note,
                &number_entry,
                &status_label,
                &status_revealer,
            ));
        }
        recents_card.append(&recents);
        root.append(&recents_card);

        root.upcast()
    }
}

fn dial_key(
    label: &str,
    entry: &gtk::Entry,
    status: &gtk::Label,
    revealer: &gtk::Revealer,
) -> gtk::Button {
    let button = super::action_button(label);
    button.set_size_request(64, 50);

    let value = String::from(label);
    let entry = entry.clone();
    let status = status.clone();
    let revealer = revealer.clone();
    button.connect_clicked(move |_| {
        let mut next = entry.text().to_string();
        next.push_str(&value);
        entry.set_text(&next);
        set_call_status(&status, &revealer, "Number updated");
    });

    button
}

fn erase_button(entry: &gtk::Entry, status: &gtk::Label, revealer: &gtk::Revealer) -> gtk::Button {
    let button = super::action_button("Delete");
    let entry = entry.clone();
    let status = status.clone();
    let revealer = revealer.clone();

    button.connect_clicked(move |_| {
        let mut current = entry.text().to_string();
        let _ = current.pop();
        entry.set_text(&current);
        if current.is_empty() {
            set_call_status(&status, &revealer, "Ready to place a call");
        } else {
            set_call_status(&status, &revealer, "Removed last digit");
        }
    });

    button
}

fn clear_button(entry: &gtk::Entry, status: &gtk::Label, revealer: &gtk::Revealer) -> gtk::Button {
    let button = super::action_button("Clear");
    let entry = entry.clone();
    let status = status.clone();
    let revealer = revealer.clone();

    button.connect_clicked(move |_| {
        entry.set_text("");
        set_call_status(&status, &revealer, "Dialer cleared");
    });

    button
}

fn call_button(
    entry: &gtk::Entry,
    status: &gtk::Label,
    revealer: &gtk::Revealer,
    network_online: bool,
    launched_contact: Option<(String, String)>,
    store: std::rc::Rc<std::cell::RefCell<crate::app_store::LauncherStore>>,
) -> gtk::Button {
    let button = super::action_button("Call");
    button.set_sensitive(network_online);

    let entry = entry.clone();
    let status = status.clone();
    let revealer = revealer.clone();
    let launched_contact = launched_contact.clone();
    button.connect_clicked(move |_| {
        let number = entry.text();
        if number.is_empty() {
            set_call_status(&status, &revealer, "Enter a number first");
        } else {
            let phone = number.to_string();
            store.borrow_mut().mark_called(&phone);
            if let Some((name, contact_phone)) = &launched_contact {
                if *contact_phone == phone {
                    set_call_status(&status, &revealer, &format!("Calling {name}"));
                    return;
                }
            }
            set_call_status(&status, &revealer, &format!("Calling {number}"));
        }
    });

    button
}

fn recent_row(
    name: &str,
    number: &str,
    note: &str,
    entry: &gtk::Entry,
    status: &gtk::Label,
    revealer: &gtk::Revealer,
) -> gtk::Button {
    let button = gtk::Button::builder().build();
    button.add_css_class("theme-button");

    let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .build();

    let copy = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();

    let title_label = gtk::Label::new(Some(name));
    title_label.set_halign(gtk::Align::Start);
    title_label.add_css_class("section-row-title");

    let subtitle_label = gtk::Label::new(Some(number));
    subtitle_label.set_halign(gtk::Align::Start);
    subtitle_label.add_css_class("section-row-subtitle");

    let note_label = gtk::Label::new(Some(note));
    note_label.set_halign(gtk::Align::End);
    note_label.add_css_class("eyebrow-label");

    copy.append(&title_label);
    copy.append(&subtitle_label);
    row.append(&copy);
    row.append(&note_label);
    button.set_child(Some(&row));

    let entry = entry.clone();
    let status = status.clone();
    let revealer = revealer.clone();
    let number = String::from(number);
    let name = String::from(name);
    button.connect_clicked(move |_| {
        entry.set_text(&number);
        set_call_status(&status, &revealer, &format!("Loaded {name}"));
    });

    button
}

fn set_call_status(label: &gtk::Label, revealer: &gtk::Revealer, value: &str) {
    revealer.set_reveal_child(false);

    let label = label.clone();
    let revealer = revealer.clone();
    let value = String::from(value);
    glib::timeout_add_local_once(std::time::Duration::from_millis(85), move || {
        label.set_label(&value);
        revealer.set_reveal_child(true);
    });
}
