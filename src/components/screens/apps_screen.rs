use crate::shell_state::AppsState;
use crate::ui::app_icon;
use relm4::gtk::prelude::*;
use relm4::gtk::{self, Align, Orientation};
use relm4::prelude::*;

pub struct AppsScreen {
    state: AppsState,
}

#[derive(Debug)]
pub enum AppsScreenInput {
    SetState(AppsState),
}

#[relm4::component(pub)]
impl SimpleComponent for AppsScreen {
    type Init = AppsState;
    type Input = AppsScreenInput;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: Orientation::Vertical,
            set_spacing: 0,
            add_css_class: "apps-screen",

            append = &app_grid(&model.state),
        }
    }

    fn init(
        state: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AppsScreen { state };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AppsScreenInput::SetState(state) => {
                self.state = state;
            }
        }
    }
}

fn app_grid(state: &AppsState) -> gtk::FlowBox {
    let flow = gtk::FlowBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .row_spacing(20)
        .column_spacing(18)
        .halign(Align::Center)
        .min_children_per_line(3)
        .max_children_per_line(4)
        .activate_on_single_click(false)
        .build();
    flow.add_css_class("apps-icon-grid");

    for module in &state.modules {
        flow.insert(&app_tile(module.icon_name, &module.title), -1);
    }

    flow
}

fn app_tile(icon_name: &str, title: &str) -> gtk::Box {
    let tile = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .halign(Align::Center)
        .valign(Align::Start)
        .build();
    tile.add_css_class("apps-icon-tile");

    let icon_shell = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .halign(Align::Center)
        .valign(Align::Center)
        .width_request(76)
        .height_request(76)
        .build();
    icon_shell.add_css_class("apps-icon-shell");
    icon_shell.append(&app_icon(icon_name));

    let label = gtk::Label::new(Some(title));
    label.set_halign(Align::Center);
    label.set_wrap(true);
    label.set_wrap_mode(relm4::gtk::pango::WrapMode::WordChar);
    label.set_justify(relm4::gtk::Justification::Center);
    label.add_css_class("apps-icon-label");

    tile.append(&icon_shell);
    tile.append(&label);
    tile
}
