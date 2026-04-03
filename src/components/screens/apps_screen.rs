use crate::shell_state::AppsState;
use crate::ui::picture_icon;
use relm4::gtk::prelude::*;
use relm4::gtk::{self, Align, Orientation};
use relm4::prelude::*;

pub struct AppsScreen {
    root_box: gtk::Box,
    state: AppsState,
}

#[derive(Debug)]
pub enum AppsScreenInput {
    SetState(AppsState),
    OpenApp(&'static str),
}

#[derive(Debug, Clone, Copy)]
pub enum AppsScreenOutput {
    OpenApp(&'static str),
}

#[relm4::component(pub)]
impl SimpleComponent for AppsScreen {
    type Init = AppsState;
    type Input = AppsScreenInput;
    type Output = AppsScreenOutput;

    view! {
        #[name(root_box)]
        gtk::Box {
            set_orientation: Orientation::Vertical,
            set_spacing: 0,
            add_css_class: "apps-screen",
        }
    }

    fn init(
        state: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();
        rebuild_grid(&widgets.root_box, &state, sender.input_sender().clone());
        let model = AppsScreen {
            root_box: widgets.root_box.clone(),
            state,
        };

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            AppsScreenInput::SetState(state) => {
                self.state = state;
                rebuild_grid(&self.root_box, &self.state, sender.input_sender().clone());
            }
            AppsScreenInput::OpenApp(app_id) => {
                sender.output(AppsScreenOutput::OpenApp(app_id)).ok();
            }
        }
    }
}

fn rebuild_grid(
    root_box: &gtk::Box,
    state: &AppsState,
    input_sender: relm4::Sender<AppsScreenInput>,
) {
    while let Some(child) = root_box.first_child() {
        root_box.remove(&child);
    }

    root_box.append(&app_grid(state, input_sender));
}

fn app_grid(state: &AppsState, input_sender: relm4::Sender<AppsScreenInput>) -> gtk::FlowBox {
    let flow = gtk::FlowBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .row_spacing(18)
        .column_spacing(12)
        .halign(Align::Center)
        .min_children_per_line(4)
        .max_children_per_line(4)
        .activate_on_single_click(false)
        .build();
    flow.add_css_class("apps-icon-grid");

    for module in &state.modules {
        flow.insert(
            &app_tile(
                module.id,
                module.icon_name,
                &module.title,
                input_sender.clone(),
            ),
            -1,
        );
    }

    flow
}

fn app_tile(
    app_id: &'static str,
    icon_name: &str,
    title: &str,
    input_sender: relm4::Sender<AppsScreenInput>,
) -> gtk::Button {
    let tile = gtk::Button::builder()
        .halign(Align::Center)
        .valign(Align::Start)
        .focus_on_click(false)
        .has_frame(false)
        .build();
    tile.set_focusable(false);
    tile.add_css_class("apps-icon-button");

    let content = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(8)
        .halign(Align::Center)
        .valign(Align::Start)
        .build();
    content.add_css_class("apps-icon-tile");

    let icon_shell = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .halign(Align::Center)
        .valign(Align::Center)
        .width_request(74)
        .height_request(74)
        .build();
    icon_shell.add_css_class("apps-icon-shell");
    icon_shell.append(&picture_icon(apps_icon_name(app_id, icon_name), 74));

    let label = gtk::Label::new(Some(title));
    label.set_halign(Align::Center);
    label.set_wrap(true);
    label.set_wrap_mode(relm4::gtk::pango::WrapMode::WordChar);
    label.set_justify(relm4::gtk::Justification::Center);
    label.add_css_class("apps-icon-label");

    content.append(&icon_shell);
    content.append(&label);
    tile.set_child(Some(&content));
    tile.connect_clicked(move |_| {
        let _ = input_sender.send(AppsScreenInput::OpenApp(app_id));
    });
    tile
}

fn apps_icon_name<'a>(app_id: &'static str, fallback: &'a str) -> &'a str {
    match app_id {
        "calendar" => "calendar-home",
        "clock" => "clock-home",
        "contacts" => "contacts-home",
        "platinum_one" => "platinum-one-home",
        "ai" => "ai-home",
        "settings" => "settings-home",
        _ => fallback,
    }
}
