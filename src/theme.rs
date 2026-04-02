use adw::{ColorScheme, StyleManager};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    Light,
    Dark,
    Auto,
}

impl ThemeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
            Self::Auto => "auto",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value.trim() {
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            "auto" => Some(Self::Auto),
            _ => None,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
            Self::Auto => "Auto",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolvedTheme {
    Light,
    Dark,
}

pub fn resolve_theme(theme_mode: ThemeMode) -> ResolvedTheme {
    match theme_mode {
        ThemeMode::Light => ResolvedTheme::Light,
        ThemeMode::Dark => ResolvedTheme::Dark,
        ThemeMode::Auto => {
            let manager = StyleManager::default();
            if manager.is_dark() {
                ResolvedTheme::Dark
            } else {
                ResolvedTheme::Light
            }
        }
    }
}

pub fn apply_theme(theme_mode: ThemeMode) {
    let manager = StyleManager::default();
    let color_scheme = match theme_mode {
        ThemeMode::Light => ColorScheme::ForceLight,
        ThemeMode::Dark => ColorScheme::ForceDark,
        ThemeMode::Auto => ColorScheme::Default,
    };

    manager.set_color_scheme(color_scheme);
}

pub fn load_theme_mode() -> ThemeMode {
    let path = theme_config_path();
    let contents = fs::read_to_string(path).ok();

    contents
        .as_deref()
        .and_then(ThemeMode::from_str)
        .unwrap_or(ThemeMode::Light)
}

pub fn save_theme_mode(theme_mode: ThemeMode) {
    let path = theme_config_path();

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let _ = fs::write(path, theme_mode.as_str());
}

fn theme_config_path() -> PathBuf {
    let config_root = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));

    config_root
        .join("platinum-shell-gtk")
        .join("theme-mode.txt")
}
