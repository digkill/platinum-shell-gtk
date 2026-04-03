use crate::device::{DeviceSnapshot, NetworkKind};
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub struct SystemToggleState {
    pub wifi_enabled: bool,
    pub lte_enabled: bool,
    pub silent_mode: bool,
    pub battery_saver: bool,
}

#[derive(Debug, Clone, Copy)]
pub enum SystemToggle {
    Wifi,
    Lte,
    Silent,
    BatterySaver,
}

pub fn load_system_toggle_state(snapshot: &DeviceSnapshot) -> SystemToggleState {
    SystemToggleState {
        wifi_enabled: detect_wifi_enabled().unwrap_or(
            snapshot.network_is_online && matches!(snapshot.network_kind, NetworkKind::Wifi),
        ),
        lte_enabled: detect_lte_enabled().unwrap_or(
            snapshot.network_is_online && matches!(snapshot.network_kind, NetworkKind::Lte),
        ),
        silent_mode: detect_silent_mode().unwrap_or(false),
        battery_saver: detect_battery_saver().unwrap_or(snapshot.battery_level <= 20),
    }
}

pub fn set_system_toggle(toggle: SystemToggle, enabled: bool) -> Result<(), String> {
    match toggle {
        SystemToggle::Wifi => set_wifi_enabled(enabled),
        SystemToggle::Lte => set_lte_enabled(enabled),
        SystemToggle::Silent => set_silent_mode(enabled),
        SystemToggle::BatterySaver => set_battery_saver(enabled),
    }
}

fn detect_wifi_enabled() -> Option<bool> {
    #[cfg(target_os = "macos")]
    {
        let device = macos_wifi_device()?;
        let output = Command::new("networksetup")
            .args(["-getairportpower", &device])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8(output.stdout).ok()?;
        return Some(stdout.to_ascii_lowercase().contains(" on"));
    }

    #[cfg(not(target_os = "macos"))]
    {
        let output = Command::new("nmcli")
            .args(["radio", "wifi"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8(output.stdout).ok()?;
        Some(stdout.trim().eq_ignore_ascii_case("enabled"))
    }
}

fn detect_lte_enabled() -> Option<bool> {
    #[cfg(target_os = "macos")]
    {
        None
    }

    #[cfg(not(target_os = "macos"))]
    {
        let output = Command::new("nmcli")
            .args(["radio", "wwan"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8(output.stdout).ok()?;
        Some(stdout.trim().eq_ignore_ascii_case("enabled"))
    }
}

fn detect_silent_mode() -> Option<bool> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("osascript")
            .args(["-e", "output muted of (get volume settings)"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8(output.stdout).ok()?;
        return Some(stdout.trim().eq_ignore_ascii_case("true"));
    }

    #[cfg(not(target_os = "macos"))]
    {
        let output = Command::new("wpctl")
            .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
            .output()
            .ok()?;
        if output.status.success() {
            let stdout = String::from_utf8(output.stdout).ok()?;
            return Some(stdout.to_ascii_lowercase().contains("[muted]"));
        }

        let output = Command::new("amixer")
            .args(["get", "Master"])
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8(output.stdout).ok()?;
        Some(stdout.to_ascii_lowercase().contains("[off]"))
    }
}

fn detect_battery_saver() -> Option<bool> {
    #[cfg(target_os = "macos")]
    {
        let output = Command::new("pmset").arg("-g").output().ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8(output.stdout).ok()?;
        return stdout
            .lines()
            .find(|line| line.contains("lowpowermode"))
            .and_then(|line| line.split_whitespace().last())
            .map(|value| value == "1");
    }

    #[cfg(not(target_os = "macos"))]
    {
        let output = Command::new("powerprofilesctl").arg("get").output().ok()?;
        if !output.status.success() {
            return None;
        }
        let stdout = String::from_utf8(output.stdout).ok()?;
        Some(stdout.trim().eq_ignore_ascii_case("power-saver"))
    }
}

fn set_wifi_enabled(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let Some(device) = macos_wifi_device() else {
            return Err(String::from("Wi-Fi device not found"));
        };
        command_result(
            "networksetup",
            &[
                "-setairportpower",
                &device,
                if enabled { "on" } else { "off" },
            ],
        )
    }

    #[cfg(not(target_os = "macos"))]
    {
        command_result(
            "nmcli",
            &["radio", "wifi", if enabled { "on" } else { "off" }],
        )
    }
}

fn set_lte_enabled(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let _ = enabled;
        Err(String::from("LTE toggle is not supported on this platform"))
    }

    #[cfg(not(target_os = "macos"))]
    {
        command_result(
            "nmcli",
            &["radio", "wwan", if enabled { "on" } else { "off" }],
        )
    }
}

fn set_silent_mode(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        command_result(
            "osascript",
            &[
                "-e",
                if enabled {
                    "set volume with output muted"
                } else {
                    "set volume without output muted"
                },
            ],
        )
    }

    #[cfg(not(target_os = "macos"))]
    {
        command_result(
            "wpctl",
            &[
                "set-mute",
                "@DEFAULT_AUDIO_SINK@",
                if enabled { "1" } else { "0" },
            ],
        )
        .or_else(|_| {
            command_result(
                "amixer",
                &["sset", "Master", if enabled { "mute" } else { "unmute" }],
            )
        })
    }
}

fn set_battery_saver(enabled: bool) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        command_result(
            "pmset",
            &["-a", "lowpowermode", if enabled { "1" } else { "0" }],
        )
    }

    #[cfg(not(target_os = "macos"))]
    {
        command_result(
            "powerprofilesctl",
            &["set", if enabled { "power-saver" } else { "balanced" }],
        )
    }
}

fn command_result(program: &str, args: &[&str]) -> Result<(), String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("{program}: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.is_empty() {
            Err(format!("{program} failed"))
        } else {
            Err(stderr)
        }
    }
}

#[cfg(target_os = "macos")]
fn macos_wifi_device() -> Option<String> {
    let output = Command::new("networksetup")
        .arg("-listallhardwareports")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let mut last_was_wifi = false;
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Hardware Port: Wi-Fi")
            || trimmed.starts_with("Hardware Port: AirPort")
        {
            last_was_wifi = true;
            continue;
        }
        if last_was_wifi && trimmed.starts_with("Device: ") {
            return trimmed
                .split_once(':')
                .map(|(_, value)| value.trim().to_string());
        }
    }
    None
}
