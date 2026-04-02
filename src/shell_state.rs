use crate::device::DeviceSnapshot;

#[derive(Debug, Clone)]
pub struct AppModule {
    pub icon_name: &'static str,
    pub title: String,
}

#[derive(Debug, Clone)]
pub struct AiTask {
    pub title: String,
    pub subtitle: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct AppsState {
    pub modules: Vec<AppModule>,
}

#[derive(Debug, Clone)]
pub struct AiState {
    pub headline_badge: String,
    pub model_label: String,
    pub context_label: String,
    pub queue_label: String,
    pub tasks: Vec<AiTask>,
}

#[derive(Debug, Clone)]
pub struct ShellState {
    pub apps: AppsState,
    pub ai: AiState,
}

pub fn load_shell_state(snapshot: &DeviceSnapshot) -> ShellState {
    let apps = AppsState {
        modules: vec![
            app("clock", "Clock"),
            app("contacts", "Contacts"),
            app("apps", "Apps"),
            app("ai", "AI"),
            app("settings", "Settings"),
            app("relay", &snapshot.network_label),
        ],
    };

    let ai = AiState {
        headline_badge: String::from("Ready"),
        model_label: String::from("Local Hybrid"),
        context_label: format!("{} / {}", snapshot.network_label, snapshot.mode_label),
        queue_label: String::from("2 Tasks"),
        tasks: vec![
            task(
                "Summarize Session",
                "Condense latest shell activity and telemetry",
                "Ready",
            ),
            task(
                "Generate Brief",
                "Operator digest with current device context",
                "Queued",
            ),
            task(
                "Extract Follow-ups",
                "Find action items from notes and contacts",
                "Standby",
            ),
        ],
    };

    ShellState { apps, ai }
}

fn app(icon_name: &'static str, title: &str) -> AppModule {
    AppModule {
        icon_name,
        title: String::from(title),
    }
}

fn task(title: &str, subtitle: &str, status: &str) -> AiTask {
    AiTask {
        title: String::from(title),
        subtitle: String::from(subtitle),
        status: String::from(status),
    }
}
