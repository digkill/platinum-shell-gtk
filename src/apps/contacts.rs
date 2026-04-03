use crate::app_sdk::{
    AppContext, AppLaunchPayload, AppManifest, AppPermission, ContactPayload, LauncherApp,
};
use crate::app_store::ContactRecord;
use relm4::gtk;
use relm4::gtk::prelude::*;

pub struct ContactsApp;

impl LauncherApp for ContactsApp {
    fn manifest(&self) -> AppManifest {
        AppManifest {
            id: "contacts",
            title: "Contacts",
            icon_name: "contacts",
            description: "People, favorites, and recent communication entries.",
        }
    }

    fn permissions(&self) -> &'static [AppPermission] {
        &[AppPermission::Contacts]
    }

    fn build_root(&self, context: &AppContext) -> gtk::Widget {
        let root = super::app_surface_root();
        root.append(&super::app_hero(
            "Contacts",
            "Favorite people, quick actions, and direct handoff into calling or messaging.",
        ));

        let favorites_card = super::app_card(
            "Favorites",
            "Tap Call or Message to open the target app with this contact prefilled.",
        );
        let favorites = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(10)
            .build();

        for contact in context.store.borrow().contacts_snapshot() {
            favorites.append(&contact_row(context, contact.clone()));
        }

        favorites_card.append(&favorites);
        root.append(&favorites_card);

        root.upcast()
    }
}

fn contact_row(context: &AppContext, record: ContactRecord) -> gtk::Box {
    let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(10)
        .build();
    row.add_css_class("section-card");

    let header = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .build();

    let copy = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .hexpand(true)
        .build();

    let title = gtk::Label::new(Some(&record.name));
    title.set_halign(gtk::Align::Start);
    title.add_css_class("section-row-title");

    let subtitle = gtk::Label::new(Some(&record.phone));
    subtitle.set_halign(gtk::Align::Start);
    subtitle.add_css_class("section-row-subtitle");

    let note = gtk::Label::new(Some(&record.note));
    note.set_halign(gtk::Align::End);
    note.add_css_class("eyebrow-label");

    copy.append(&title);
    copy.append(&subtitle);
    header.append(&copy);
    header.append(&note);

    let actions = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(10)
        .build();

    let payload = AppLaunchPayload::Contact(ContactPayload {
        name: record.name.clone(),
        phone: record.phone.clone(),
    });

    let call_button = super::action_button("Call");
    let call_nav = context.navigator.clone();
    let call_payload = payload.clone();
    call_button.connect_clicked(move |_| {
        call_nav.open("call", Some(call_payload.clone()));
    });

    let message_button = super::action_button("Message");
    let message_nav = context.navigator.clone();
    message_button.connect_clicked(move |_| {
        message_nav.open("message", Some(payload.clone()));
    });

    actions.append(&call_button);
    actions.append(&message_button);

    row.append(&header);
    row.append(&actions);
    row
}
