use crate::app_sdk::{AppContext, AppManifest, AppPermission, LauncherApp};

pub struct RelayApp;

impl LauncherApp for RelayApp {
    fn manifest(&self) -> AppManifest {
        AppManifest {
            id: "relay",
            title: "Relay",
            icon_name: "relay",
            description: "Local bridge and device-to-device connectivity surface.",
        }
    }

    fn permissions(&self) -> &'static [AppPermission] {
        &[AppPermission::Network]
    }

    fn build_root(&self, _context: &AppContext) -> relm4::gtk::Widget {
        super::placeholder_root(
            "Relay",
            "Connectivity bridge for nearby devices and future radio-backed routing.",
        )
    }
}
