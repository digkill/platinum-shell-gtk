use crate::app_sdk::{AppContext, AppManifest, AppPermission, LauncherApp};

pub struct CalendarApp;

impl LauncherApp for CalendarApp {
    fn manifest(&self) -> AppManifest {
        AppManifest {
            id: "calendar",
            title: "Calendar",
            icon_name: "calendar-home",
            description: "Native calendar surface for upcoming events and schedule blocks.",
        }
    }

    fn permissions(&self) -> &'static [AppPermission] {
        &[AppPermission::Clock]
    }

    fn build_root(&self, _context: &AppContext) -> relm4::gtk::Widget {
        super::placeholder_root(
            "Calendar",
            "Native calendar app surface for schedule, events, and day planning.",
        )
    }
}
