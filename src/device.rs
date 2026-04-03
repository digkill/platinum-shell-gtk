use chrono::Local;
use std::fs;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct DeviceSnapshot {
    pub battery_level: u8,
    pub battery_charging: bool,
    pub date_label: String,
    pub network_kind: NetworkKind,
    pub network_is_online: bool,
    pub signal_level: u8,
    pub time_label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkKind {
    Offline,
    Wifi,
    Ethernet,
    Lte,
    Unknown,
}

struct NetworkSnapshot {
    kind: NetworkKind,
    is_online: bool,
    signal_level: u8,
}

pub fn load_device_snapshot() -> DeviceSnapshot {
    let now = Local::now();
    let (battery_level, battery_charging) = read_linux_battery_snapshot().unwrap_or((82, false));
    let network = read_network_snapshot().unwrap_or_else(NetworkSnapshot::offline);

    DeviceSnapshot {
        battery_level,
        battery_charging,
        date_label: now.format("%a, %d %b").to_string(),
        network_kind: network.kind,
        network_is_online: network.is_online,
        signal_level: network.signal_level,
        time_label: now.format("%H:%M").to_string(),
    }
}

fn read_linux_battery_snapshot() -> Option<(u8, bool)> {
    let power_supply_dir = fs::read_dir("/sys/class/power_supply").ok()?;

    for entry in power_supply_dir.flatten() {
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if !file_name.starts_with("BAT") {
            continue;
        }

        let capacity_path = entry.path().join("capacity");
        let capacity_contents = fs::read_to_string(capacity_path).ok()?;
        let battery_level = capacity_contents.trim().parse::<u8>().ok()?;
        let status_path = entry.path().join("status");
        let battery_charging = fs::read_to_string(status_path)
            .ok()
            .map(|status| is_charging_status(&status))
            .unwrap_or(false);

        return Some((battery_level, battery_charging));
    }

    let output = Command::new("upower").args(["-e"]).output().ok()?;
    let stdout = String::from_utf8(output.stdout).ok()?;
    let battery_device = stdout.lines().find(|line| line.contains("battery"))?;

    let battery_output = Command::new("upower")
        .args(["-i", battery_device])
        .output()
        .ok()?;
    let battery_stdout = String::from_utf8(battery_output.stdout).ok()?;

    let battery_level = parse_percentage_from_text(&battery_stdout)?;
    let battery_charging = battery_stdout
        .lines()
        .find(|line| line.trim_start().starts_with("state:"))
        .map(is_charging_status)
        .unwrap_or(false);

    Some((battery_level, battery_charging))
}

fn parse_percentage_from_text(text: &str) -> Option<u8> {
    text.split_whitespace().find_map(|segment| {
        let trimmed_segment = segment.trim_end_matches([';', ',']);
        let numeric_part = trimmed_segment.strip_suffix('%')?;

        numeric_part.parse::<u8>().ok()
    })
}

fn is_charging_status(text: &str) -> bool {
    let normalized = text.trim().to_ascii_lowercase();
    normalized.contains("charging") || normalized.contains("fully-charged")
}

impl NetworkSnapshot {
    fn offline() -> Self {
        Self {
            kind: NetworkKind::Offline,
            is_online: false,
            signal_level: 0,
        }
    }
}

fn read_network_snapshot() -> Option<NetworkSnapshot> {
    read_linux_network_snapshot()
        .or_else(read_linux_modem_snapshot)
        .or_else(read_macos_network_snapshot)
}

fn read_linux_network_snapshot() -> Option<NetworkSnapshot> {
    let output = Command::new("nmcli")
        .args(["-t", "-f", "TYPE,ACTIVE,SIGNAL,CONNECTION", "dev"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let line = stdout.lines().find(|line| line.contains(":connected:"))?;
    let parts = line.split(':').collect::<Vec<_>>();
    let kind = parts.first().copied().unwrap_or("wifi");
    let signal_percent = parts
        .get(2)
        .and_then(|value| value.parse::<u8>().ok())
        .unwrap_or(if kind == "ethernet" { 100 } else { 0 });

    Some(NetworkSnapshot {
        kind: network_kind(kind),
        is_online: true,
        signal_level: signal_percent_to_level(signal_percent),
    })
}

fn read_macos_network_snapshot() -> Option<NetworkSnapshot> {
    let airport =
        "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport";
    let output = Command::new(airport).arg("-I").output().ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let rssi = stdout.lines().find_map(|line| {
        line.split_once(" agrCtlRSSI: ")
            .and_then(|(_, value)| value.trim().parse::<i32>().ok())
    })?;

    Some(NetworkSnapshot {
        kind: NetworkKind::Wifi,
        is_online: true,
        signal_level: signal_percent_to_level(rssi_to_percent(rssi)),
    })
}

fn read_linux_modem_snapshot() -> Option<NetworkSnapshot> {
    let modems_output = Command::new("mmcli").arg("-L").output().ok()?;
    if !modems_output.status.success() {
        return None;
    }

    let modems_stdout = String::from_utf8(modems_output.stdout).ok()?;
    let modem_path = modems_stdout.lines().find_map(|line| {
        line.split_whitespace()
            .find(|part| part.starts_with("/org/freedesktop/ModemManager1/Modem/"))
    })?;

    let modem_output = Command::new("mmcli")
        .args(["-m", modem_path])
        .output()
        .ok()?;
    if !modem_output.status.success() {
        return None;
    }

    let modem_stdout = String::from_utf8(modem_output.stdout).ok()?;
    let state = find_mmcli_value(&modem_stdout, "state")?;
    if !state.to_ascii_lowercase().contains("connected")
        && !state.to_ascii_lowercase().contains("registered")
    {
        return None;
    }

    let signal_percent = find_mmcli_value(&modem_stdout, "signal quality")
        .and_then(|value| value.split_whitespace().next().map(String::from))
        .and_then(|value| value.parse::<u8>().ok())
        .unwrap_or(60);

    Some(NetworkSnapshot {
        kind: NetworkKind::Lte,
        is_online: true,
        signal_level: signal_percent_to_level(signal_percent),
    })
}

fn signal_percent_to_level(signal_percent: u8) -> u8 {
    ((u16::from(signal_percent.min(100)) * 5) / 100).clamp(0, 5) as u8
}

fn rssi_to_percent(rssi: i32) -> u8 {
    let clamped = rssi.clamp(-90, -40);
    (((clamped + 90) * 100) / 50) as u8
}

fn network_kind(kind: &str) -> NetworkKind {
    match kind {
        "wifi" | "wireless" => NetworkKind::Wifi,
        "ethernet" => NetworkKind::Ethernet,
        "gsm" | "wwan" => NetworkKind::Lte,
        "offline" => NetworkKind::Offline,
        _ => NetworkKind::Unknown,
    }
}

fn find_mmcli_value(text: &str, key: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let normalized = line.trim();
        if !normalized.contains(key) {
            return None;
        }
        normalized
            .split_once(':')
            .map(|(_, value)| value.trim().trim_matches('\'').to_string())
            .filter(|value| !value.is_empty() && value != "--")
    })
}
