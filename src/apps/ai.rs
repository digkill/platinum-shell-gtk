use crate::app_sdk::{AppContext, AppManifest, AppPermission, LauncherApp};
use relm4::gtk;
use relm4::gtk::prelude::*;

pub struct AiApp;

impl LauncherApp for AiApp {
    fn manifest(&self) -> AppManifest {
        AppManifest {
            id: "ai",
            title: "AI",
            icon_name: "ai",
            description: "Assistant workflows and local inference handoff point.",
        }
    }

    fn permissions(&self) -> &'static [AppPermission] {
        &[AppPermission::Network, AppPermission::ShellState]
    }

    fn build_root(&self, context: &AppContext) -> gtk::Widget {
        let root = super::app_surface_root();
        root.append(&super::app_hero(
            "AI Assistant",
            "Shell-aware summaries, notes extraction, and local/remote inference workflows.",
        ));

        let snapshot_card = super::app_card(
            "Context Window",
            "Current device context passed into the assistant surface.",
        );
        let meta = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10)
            .build();
        meta.append(&super::pill(&format!(
            "Time {}",
            context.snapshot.time_label
        )));
        meta.append(&super::pill(&format!(
            "Date {}",
            context.snapshot.date_label
        )));
        meta.append(&super::pill(if context.snapshot.network_is_online {
            "Online"
        } else {
            "Offline"
        }));
        snapshot_card.append(&meta);
        root.append(&snapshot_card);

        let actions_card = super::app_card(
            "Quick Actions",
            "High-frequency assistant jobs prepared for one-handed use.",
        );
        let actions = gtk::FlowBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .row_spacing(10)
            .column_spacing(10)
            .min_children_per_line(2)
            .max_children_per_line(2)
            .build();
        actions.insert(&super::action_button("Summarize Session"), -1);
        actions.insert(&super::action_button("Build Brief"), -1);
        actions.insert(&super::action_button("Extract Tasks"), -1);
        actions.insert(&super::action_button("Draft Reply"), -1);
        actions_card.append(&actions);
        root.append(&actions_card);

        root.append(&super::app_row(
            "Session Summary",
            "Condense recent launcher activity and operator notes into a readable brief.",
            "Ready",
        ));
        root.append(&super::app_row(
            "Field Digest",
            "Generate current device state, battery, network, and contact activity overview.",
            "Queued",
        ));
        root.append(&super::app_row(
            "Notes Pipeline",
            "Extract next actions from future contacts and message surfaces.",
            "Standby",
        ));

        root.upcast()
    }
}
