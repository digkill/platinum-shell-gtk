use crate::app_sdk::{AppContext, AppLaunchPayload, AppManifest, AppPermission, LauncherApp};
use crate::app_store::{MessageRecord, MessageThread};
use relm4::gtk;
use relm4::gtk::glib;
use relm4::gtk::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

pub struct MessageApp;

#[derive(Clone)]
struct ThreadState {
    title: String,
    status: String,
    preview: String,
    messages: Vec<MessageEntry>,
}

#[derive(Clone)]
struct MessageEntry {
    author: String,
    body: String,
    outgoing: bool,
}

impl LauncherApp for MessageApp {
    fn manifest(&self) -> AppManifest {
        AppManifest {
            id: "message",
            title: "Message",
            icon_name: "message",
            description: "Conversations, relay threads, and device message sync.",
        }
    }

    fn permissions(&self) -> &'static [AppPermission] {
        &[
            AppPermission::Contacts,
            AppPermission::Network,
            AppPermission::ShellState,
        ]
    }

    fn build_root(&self, context: &AppContext) -> gtk::Widget {
        let root = super::app_surface_root();
        root.add_css_class("message-app-root");
        root.set_spacing(12);
        root.append(&super::app_hero(
            "Message",
            "Unified thread view for chats, relay links, and direct handheld messaging.",
        ));

        let top_actions = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk::Align::Start)
            .build();
        let contacts_button = super::action_button("Contacts");
        let contacts_nav = context.navigator.clone();
        contacts_button.connect_clicked(move |_| {
            contacts_nav.open("contacts", None);
        });
        top_actions.append(&contacts_button);
        root.append(&top_actions);

        let launch_contact = match &context.launch_payload {
            Some(AppLaunchPayload::Contact(contact)) => {
                Some((contact.name.clone(), contact.phone.clone()))
            }
            None => None,
        };
        let active_index = Rc::new(RefCell::new(if let Some((name, phone)) = &launch_contact {
            context
                .store
                .borrow_mut()
                .ensure_direct_thread(&crate::app_store::ContactRecord {
                    name: name.clone(),
                    phone: phone.clone(),
                    note: String::from("Direct"),
                    created_at: 0,
                    last_called_at: 0,
                    last_message_at: 0,
                    updated_at: 0,
                })
        } else {
            0
        }));

        let layout = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(10)
            .build();

        let thread_card = super::app_card(
            "Threads",
            "Tap a conversation to focus the detail view and composer.",
        );
        thread_card.add_css_class("message-thread-card");
        let thread_list = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .build();
        thread_card.append(&thread_list);

        let detail_card = super::app_card(
            "Conversation",
            "Selected thread, recent history, and inline composer.",
        );
        detail_card.add_css_class("message-detail-card");
        let detail_revealer = gtk::Revealer::builder()
            .transition_type(gtk::RevealerTransitionType::Crossfade)
            .transition_duration(180)
            .reveal_child(true)
            .build();
        let detail_content = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(10)
            .build();
        let detail_header = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .build();

        let detail_copy = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();
        let thread_title = gtk::Label::new(None);
        thread_title.set_halign(gtk::Align::Start);
        thread_title.add_css_class("section-row-title");
        let thread_preview = gtk::Label::new(None);
        thread_preview.set_halign(gtk::Align::Start);
        thread_preview.set_wrap(true);
        thread_preview.add_css_class("section-row-subtitle");
        detail_copy.append(&thread_title);
        detail_copy.append(&thread_preview);

        let thread_status = gtk::Label::new(None);
        thread_status.add_css_class("eyebrow-label");

        detail_header.append(&detail_copy);
        detail_header.append(&thread_status);

        let messages_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(6)
            .build();
        let messages_scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .min_content_height(220)
            .build();
        messages_scroll.set_child(Some(&messages_box));

        let composer = gtk::TextView::builder()
            .wrap_mode(gtk::WrapMode::WordChar)
            .top_margin(12)
            .bottom_margin(12)
            .left_margin(12)
            .right_margin(12)
            .height_request(92)
            .build();
        composer.add_css_class("section-card-subtitle");

        let initial_composer_status = match &launch_contact {
            Some((name, _)) => format!("Composing to {name}"),
            None => String::from("Draft ready"),
        };
        let composer_status = gtk::Label::new(Some(&initial_composer_status));
        composer_status.set_halign(gtk::Align::Start);
        composer_status.add_css_class("section-row-subtitle");

        let composer_actions = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .halign(gtk::Align::End)
            .build();
        let clear_button = super::action_button("Clear");
        let send_button = super::action_button("Send");
        composer_actions.append(&clear_button);
        composer_actions.append(&send_button);

        detail_content.append(&detail_header);
        detail_content.append(&messages_scroll);
        detail_content.append(&composer);
        detail_content.append(&composer_status);
        detail_content.append(&composer_actions);
        detail_revealer.set_child(Some(&detail_content));
        detail_card.append(&detail_revealer);

        layout.append(&thread_card);
        layout.append(&detail_card);
        root.append(&layout);

        rebuild_thread_list(
            &thread_list,
            context.store.clone(),
            active_index.clone(),
            thread_title.clone(),
            thread_preview.clone(),
            thread_status.clone(),
            messages_box.clone(),
            composer_status.clone(),
            detail_revealer.clone(),
        );
        sync_active_thread(
            &context.store.borrow().threads_snapshot(),
            *active_index.borrow(),
            &thread_title,
            &thread_preview,
            &thread_status,
            &messages_box,
        );

        let composer_buffer = composer.buffer();
        {
            let composer_buffer = composer_buffer.clone();
            let composer_status = composer_status.clone();
            clear_button.connect_clicked(move |_| {
                composer_buffer.set_text("");
                composer_status.set_label("Draft cleared");
            });
        }
        {
            let store = context.store.clone();
            let active_index = active_index.clone();
            let composer_buffer = composer_buffer.clone();
            let thread_title = thread_title.clone();
            let thread_preview = thread_preview.clone();
            let thread_status = thread_status.clone();
            let messages_box = messages_box.clone();
            let composer_status = composer_status.clone();
            let thread_list = thread_list.clone();
            send_button.connect_clicked(move |_| {
                let text = composer_buffer
                    .text(
                        &composer_buffer.start_iter(),
                        &composer_buffer.end_iter(),
                        true,
                    )
                    .to_string();
                let trimmed = text.trim();

                if trimmed.is_empty() {
                    composer_status.set_label("Type a message first");
                    return;
                }

                let active = *active_index.borrow();
                store.borrow_mut().append_outgoing_message(active, trimmed);

                composer_buffer.set_text("");
                composer_status.set_label("Message queued locally");
                rebuild_thread_list(
                    &thread_list,
                    store.clone(),
                    active_index.clone(),
                    thread_title.clone(),
                    thread_preview.clone(),
                    thread_status.clone(),
                    messages_box.clone(),
                    composer_status.clone(),
                    detail_revealer.clone(),
                );
                sync_active_thread(
                    &store.borrow().threads_snapshot(),
                    active,
                    &thread_title,
                    &thread_preview,
                    &thread_status,
                    &messages_box,
                );
            });
        }

        root.upcast()
    }
}

fn rebuild_thread_list(
    container: &gtk::Box,
    store: Rc<RefCell<crate::app_store::LauncherStore>>,
    active_index: Rc<RefCell<usize>>,
    thread_title: gtk::Label,
    thread_preview: gtk::Label,
    thread_status: gtk::Label,
    messages_box: gtk::Box,
    composer_status: gtk::Label,
    detail_revealer: gtk::Revealer,
) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    let threads = store.borrow().threads_snapshot();
    for (index, thread) in threads.into_iter().map(thread_from_store).enumerate() {
        let button = gtk::Button::builder().build();
        button.add_css_class("theme-button");
        if index == *active_index.borrow() {
            button.add_css_class("theme-button-active");
        }

        let row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .build();
        let copy = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(2)
            .hexpand(true)
            .build();

        let title = gtk::Label::new(Some(&thread.title));
        title.set_halign(gtk::Align::Start);
        title.add_css_class("section-row-title");

        let subtitle = gtk::Label::new(Some(&thread.preview));
        subtitle.set_halign(gtk::Align::Start);
        subtitle.set_wrap(true);
        subtitle.add_css_class("section-row-subtitle");

        let trailing = gtk::Label::new(Some(&thread.status));
        trailing.set_halign(gtk::Align::End);
        trailing.add_css_class("eyebrow-label");

        copy.append(&title);
        copy.append(&subtitle);
        row.append(&copy);
        row.append(&trailing);
        button.set_child(Some(&row));

        let store = store.clone();
        let active_index = active_index.clone();
        let closure_container = container.clone();
        let thread_title = thread_title.clone();
        let thread_preview = thread_preview.clone();
        let thread_status = thread_status.clone();
        let messages_box = messages_box.clone();
        let composer_status = composer_status.clone();
        let detail_revealer = detail_revealer.clone();
        button.connect_clicked(move |_| {
            *active_index.borrow_mut() = index;
            composer_status.set_label("Conversation selected");
            animate_thread_switch(
                &detail_revealer,
                store.clone(),
                index,
                &thread_title,
                &thread_preview,
                &thread_status,
                &messages_box,
            );
            rebuild_thread_list(
                &closure_container,
                store.clone(),
                active_index.clone(),
                thread_title.clone(),
                thread_preview.clone(),
                thread_status.clone(),
                messages_box.clone(),
                composer_status.clone(),
                detail_revealer.clone(),
            );
        });

        container.append(&button);
    }
}

fn animate_thread_switch(
    detail_revealer: &gtk::Revealer,
    store: Rc<RefCell<crate::app_store::LauncherStore>>,
    index: usize,
    thread_title: &gtk::Label,
    thread_preview: &gtk::Label,
    thread_status: &gtk::Label,
    messages_box: &gtk::Box,
) {
    detail_revealer.set_reveal_child(false);

    let detail_revealer = detail_revealer.clone();
    let thread_title = thread_title.clone();
    let thread_preview = thread_preview.clone();
    let thread_status = thread_status.clone();
    let messages_box = messages_box.clone();
    glib::timeout_add_local_once(std::time::Duration::from_millis(90), move || {
        let threads = store.borrow().threads_snapshot();
        sync_active_thread(
            &threads,
            index,
            &thread_title,
            &thread_preview,
            &thread_status,
            &messages_box,
        );
        detail_revealer.set_reveal_child(true);
    });
}

fn sync_active_thread(
    threads: &[MessageThread],
    active: usize,
    thread_title: &gtk::Label,
    thread_preview: &gtk::Label,
    thread_status: &gtk::Label,
    messages_box: &gtk::Box,
) {
    let Some(thread) = threads.get(active) else {
        return;
    };

    thread_title.set_label(&thread.title);
    thread_preview.set_label(&thread.preview);
    thread_status.set_label(&thread.status);
    let messages = thread
        .messages
        .iter()
        .cloned()
        .map(message_from_store)
        .collect::<Vec<_>>();
    rebuild_messages(messages_box, &messages);
}

fn rebuild_messages(container: &gtk::Box, messages: &[MessageEntry]) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    for message in messages {
        let bubble = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .halign(if message.outgoing {
                gtk::Align::End
            } else {
                gtk::Align::Start
            })
            .build();
        bubble.add_css_class("section-card");
        if message.outgoing {
            bubble.add_css_class("theme-button-active");
        }

        let author = gtk::Label::new(Some(&message.author));
        author.set_halign(gtk::Align::Start);
        author.add_css_class("eyebrow-label");

        let body = gtk::Label::new(Some(&message.body));
        body.set_halign(gtk::Align::Start);
        body.set_wrap(true);
        body.add_css_class("section-card-subtitle");

        bubble.append(&author);
        bubble.append(&body);
        container.append(&bubble);
    }
}

fn thread_from_store(thread: MessageThread) -> ThreadState {
    ThreadState {
        title: thread.title,
        status: thread.status,
        preview: thread.preview,
        messages: thread
            .messages
            .into_iter()
            .map(message_from_store)
            .collect(),
    }
}

fn message_from_store(message: MessageRecord) -> MessageEntry {
    MessageEntry {
        author: message.author,
        body: message.body,
        outgoing: message.outgoing,
    }
}
