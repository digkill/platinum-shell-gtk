use crate::shell_state::AiState;
use crate::ui::{action_row, marker_pill, preference_group, surface_header, titled_card};
use adw::prelude::PreferencesGroupExt;
use relm4::gtk::prelude::*;
use relm4::gtk::{self, Align, Orientation};
use relm4::prelude::*;

pub struct AiScreen {
    state: AiState,
}

#[derive(Debug)]
pub enum AiScreenInput {
    SetState(AiState),
}

#[relm4::component(pub)]
impl SimpleComponent for AiScreen {
    type Init = AiState;
    type Input = AiScreenInput;
    type Output = ();

    view! {
        gtk::Box {
            set_orientation: Orientation::Vertical,
            set_spacing: 16,

            append = &surface_header("AI", "Assistant workflows prepared for local and remote inference.", &model.state.headline_badge),

            append = &summary_card(
                "Assistant State",
                "Context-aware shell handoff point for notes, summaries, and operator support.",
                &model.state,
            ),

            append = &tasks_group(
                "Queued Actions",
                "Fast actions that map to live shell state instead of hardcoded copy.",
                &model.state,
            ),
        }
    }

    fn init(
        state: Self::Init,
        root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = AiScreen { state };
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            AiScreenInput::SetState(state) => {
                self.state = state;
            }
        }
    }
}

fn summary_card(title: &str, subtitle: &str, state: &AiState) -> gtk::Box {
    let card = titled_card(title, subtitle);
    let row = gtk::FlowBox::builder()
        .selection_mode(gtk::SelectionMode::None)
        .row_spacing(12)
        .column_spacing(12)
        .min_children_per_line(1)
        .max_children_per_line(3)
        .activate_on_single_click(false)
        .build();
    row.add_css_class("metric-flowbox");

    row.insert(
        &metric_column("Model", &state.model_label, "Ready", "accent-success"),
        -1,
    );
    row.insert(
        &metric_column("Context", &state.context_label, "Live", "accent-cyan"),
        -1,
    );
    row.insert(
        &metric_column("Queue", &state.queue_label, "Hot", "accent-violet"),
        -1,
    );

    card.append(&crate::ui::divider());
    card.append(&row);
    card
}

fn tasks_group(title: &str, subtitle: &str, state: &AiState) -> gtk::Box {
    let container = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(10)
        .build();

    let group = preference_group(title, subtitle);
    group.add_css_class("material-group");

    for task in &state.tasks {
        group.add(&action_row(&task.title, &task.subtitle, &task.status));
    }

    container.append(&group);
    container
}

fn metric_column(title: &str, value: &str, marker: &str, marker_class: &str) -> gtk::Box {
    let column = gtk::Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(4)
        .hexpand(true)
        .build();

    let title_label = gtk::Label::new(Some(title));
    title_label.set_halign(Align::Start);
    title_label.add_css_class("eyebrow-label");

    let value_label = gtk::Label::new(Some(value));
    value_label.set_halign(Align::Start);
    value_label.add_css_class("section-row-title");

    column.append(&title_label);
    column.append(&value_label);
    column.append(&marker_pill(marker, marker_class));
    column
}
