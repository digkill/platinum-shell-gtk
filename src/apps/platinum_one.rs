use crate::app_sdk::{AppContext, AppManifest, AppPermission, LauncherApp};

pub struct PlatinumOneApp;

impl LauncherApp for PlatinumOneApp {
    fn manifest(&self) -> AppManifest {
        AppManifest {
            id: "platinum_one",
            title: "Platinum One",
            icon_name: "platinum-one-home",
            description: "Primary product surface and custom native experience entry point.",
        }
    }

    fn permissions(&self) -> &'static [AppPermission] {
        &[AppPermission::ShellState, AppPermission::Network]
    }

    fn build_root(&self, _context: &AppContext) -> relm4::gtk::Widget {
        super::placeholder_root(
            "Platinum One",
            "Primary native app entry for the handheld experience and future integrated modules.",
        )
    }
}
