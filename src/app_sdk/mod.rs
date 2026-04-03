#![allow(dead_code)]

use crate::app_store::LauncherStore;
use crate::device::DeviceSnapshot;
use relm4::gtk;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppPermission {
    Battery,
    Contacts,
    Clock,
    Network,
    ShellState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AppManifest {
    pub id: &'static str,
    pub title: &'static str,
    pub icon_name: &'static str,
    pub description: &'static str,
}

#[derive(Debug, Clone)]
pub struct AppContext {
    pub launch_payload: Option<AppLaunchPayload>,
    pub navigator: AppNavigator,
    pub snapshot: DeviceSnapshot,
    pub store: Rc<RefCell<LauncherStore>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContactPayload {
    pub name: String,
    pub phone: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppLaunchPayload {
    Contact(ContactPayload),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppLaunchRequest {
    pub app_id: &'static str,
    pub payload: Option<AppLaunchPayload>,
}

#[derive(Clone)]
pub struct AppNavigator {
    navigate: Rc<dyn Fn(AppLaunchRequest)>,
}

impl AppNavigator {
    pub fn new(navigate: impl Fn(AppLaunchRequest) + 'static) -> Self {
        Self {
            navigate: Rc::new(navigate),
        }
    }

    pub fn open(&self, app_id: &'static str, payload: Option<AppLaunchPayload>) {
        (self.navigate)(AppLaunchRequest { app_id, payload });
    }
}

impl std::fmt::Debug for AppNavigator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AppNavigator(..)")
    }
}

pub trait LauncherApp {
    fn manifest(&self) -> AppManifest;

    fn permissions(&self) -> &'static [AppPermission] {
        &[]
    }

    fn on_register(&self, _context: &AppContext) {}

    fn on_activate(&self, _context: &AppContext) {}

    fn on_deactivate(&self, _context: &AppContext) {}

    fn build_root(&self, context: &AppContext) -> gtk::Widget;
}

pub struct AppRegistry {
    apps: Vec<Box<dyn LauncherApp>>,
}

impl AppRegistry {
    pub fn new(apps: Vec<Box<dyn LauncherApp>>) -> Self {
        Self { apps }
    }

    pub fn manifests(&self) -> Vec<AppManifest> {
        self.apps.iter().map(|app| app.manifest()).collect()
    }

    pub fn manifest(&self, id: &str) -> Option<AppManifest> {
        self.apps
            .iter()
            .find(|app| app.manifest().id == id)
            .map(|app| app.manifest())
    }

    pub fn build_root(&self, id: &str, context: &AppContext) -> Option<gtk::Widget> {
        let app = self.apps.iter().find(|app| app.manifest().id == id)?;
        app.on_activate(context);
        Some(app.build_root(context))
    }

    pub fn deactivate(&self, id: &str, context: &AppContext) {
        if let Some(app) = self.apps.iter().find(|app| app.manifest().id == id) {
            app.on_deactivate(context);
        }
    }

    pub fn register_all(&self, context: &AppContext) {
        for app in &self.apps {
            app.on_register(context);
        }
    }
}
