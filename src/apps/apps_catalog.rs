use crate::app_sdk::{AppContext, AppManifest, AppPermission, LauncherApp};

pub struct AppsCatalogApp;

impl LauncherApp for AppsCatalogApp {
    fn manifest(&self) -> AppManifest {
        AppManifest {
            id: "apps",
            title: "Apps",
            icon_name: "apps",
            description: "Installed launcher applications and native module entry points.",
        }
    }

    fn permissions(&self) -> &'static [AppPermission] {
        &[AppPermission::ShellState]
    }

    fn build_root(&self, _context: &AppContext) -> relm4::gtk::Widget {
        super::placeholder_root(
            "Apps",
            "Registry-backed launcher catalog for built-in and future third-party native apps.",
        )
    }
}
