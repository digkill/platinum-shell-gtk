use crate::apps::app_registry;
use crate::device::DeviceSnapshot;

#[derive(Debug, Clone)]
pub struct AppModule {
    pub id: &'static str,
    pub icon_name: &'static str,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct AppsState {
    pub modules: Vec<AppModule>,
}

#[derive(Debug, Clone)]
pub struct ShellState {
    pub apps: AppsState,
}

pub fn load_shell_state(snapshot: &DeviceSnapshot) -> ShellState {
    let registry = app_registry();
    let apps = AppsState {
        modules: registry
            .manifests()
            .into_iter()
            .map(app_from_manifest)
            .collect(),
    };

    let _ = snapshot;
    ShellState { apps }
}

fn app_from_manifest(manifest: crate::app_sdk::AppManifest) -> AppModule {
    AppModule {
        id: manifest.id,
        icon_name: manifest.icon_name,
        title: String::from(manifest.title),
    }
}
