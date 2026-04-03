use crate::app_sdk::AppManifest;
use relm4::gtk::prelude::*;
use relm4::gtk::{self, Align, Orientation};
use relm4::prelude::*;

pub struct AppHost {
    active_app: Option<AppManifest>,
    content_slot: gtk::Box,
}

#[derive(Debug)]
pub enum AppHostInput {
    Show {
        manifest: AppManifest,
        root: gtk::Widget,
    },
    Close,
}

#[derive(Debug)]
pub enum AppHostOutput {
    CloseRequested,
}

#[relm4::component(pub)]
impl SimpleComponent for AppHost {
    type Init = ();
    type Input = AppHostInput;
    type Output = AppHostOutput;

    view! {
        gtk::Box {
            set_orientation: Orientation::Vertical,
            set_spacing: 12,
            add_css_class: "app-host",

            append = &gtk::Box {
                set_orientation: Orientation::Horizontal,
                set_spacing: 12,
                set_valign: Align::Center,
                add_css_class: "app-host-bar",

                append = &gtk::Button {
                    set_label: "Back",
                    add_css_class: "app-host-back",
                    connect_clicked[sender] => move |_| {
                        sender.output(AppHostOutput::CloseRequested).ok();
                    }
                },

                append = &gtk::Label {
                    set_halign: Align::Center,
                    set_hexpand: true,
                    add_css_class: "app-host-title",
                    set_label: "Application",
                }
            },

            #[name(content_slot)]
            gtk::Box {
                set_orientation: Orientation::Vertical,
                set_vexpand: true,
                set_hexpand: true,
                add_css_class: "app-host-content",
            }
        }
    }

    fn init(
        _: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();
        let model = AppHost {
            active_app: None,
            content_slot: widgets.content_slot.clone(),
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppHostInput::Show { manifest, root } => {
                self.active_app = Some(manifest);
                clear_box(&self.content_slot);
                self.content_slot.append(&root);
            }
            AppHostInput::Close => {
                self.active_app = None;
                clear_box(&self.content_slot);
            }
        }
    }
}

fn clear_box(container: &gtk::Box) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }
}
