use crate::app::PadFilterType;
use steamworks;

#[derive(Clone)]
pub struct Instance {
    pub devices: Vec<usize>,
    pub profname: String,
    pub profselection: usize,
}

use evdev::*;

#[derive(Clone, PartialEq, Copy)]
pub enum DeviceType {
    Gamepad,
    Keyboard,
    Mouse,
    Other,
}

#[derive(Debug)]
pub enum PadButton {
    Left,
    Right,
    Up,
    Down,
    ABtn,
    BBtn,
    XBtn,
    YBtn,
    StartBtn,
    SelectBtn,

    AKey,
    RKey,
    XKey,
    ZKey,

    RightClick,
}

#[derive(Clone)]
pub struct DeviceInfo {
    pub path: String,
    pub vendor: u16,
    pub enabled: bool,
    pub device_type: DeviceType,
}

pub struct InputDevice {
    path: String,
    dev: Device,
    phys: String,
    uniq: String,
    steam_name: Option<String>,
    enabled: bool,
    device_type: DeviceType,
    has_button_held: bool,
}
impl InputDevice {
    pub fn name(&self) -> &str {
        self.dev.name().unwrap_or_else(|| "")
    }
    pub fn emoji(&self) -> &str {
        match self.device_type() {
            DeviceType::Gamepad => "🎮",
            DeviceType::Keyboard => "🖮",
            DeviceType::Mouse => "🖱",
            DeviceType::Other => "",
        }
    }
    pub fn fancyname(&self) -> &str {
        if let Some(name) = &self.steam_name {
            return name;
        }
        match self.dev.input_id().vendor() {
            0x045e => "Xbox Controller",
            0x054c => "PS Controller",
            0x057e => "NT Pro Controller",
            0x28de => "Steam Input",
            _ => self.name(),
        }
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn phys(&self) -> &str {
        &self.phys
    }
    pub fn uniq(&self) -> &str {
        &self.uniq
    }
    pub fn vendor(&self) -> u16 {
        self.dev.input_id().vendor()
    }
    pub fn is_steam_input(&self) -> bool {
        self.vendor() == 0x28de
    }
    pub fn enabled(&self) -> bool {
        self.enabled
    }
    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }
    pub fn has_button_held(&self) -> bool {
        self.has_button_held
    }
    pub fn poll(&mut self) -> Option<PadButton> {
        let mut btn: Option<PadButton> = None;
        if let Ok(events) = self.dev.fetch_events() {
            for event in events {
                let summary = event.destructure();

                match summary {
                    EventSummary::Key(_, _, 1) => {
                        self.has_button_held = true;
                    }
                    EventSummary::Key(_, _, 0) => {
                        self.has_button_held = false;
                    }
                    _ => {}
                }

                btn = match summary {
                    EventSummary::Key(_, KeyCode::BTN_SOUTH, 1) => Some(PadButton::ABtn),
                    EventSummary::Key(_, KeyCode::BTN_EAST, 1) => Some(PadButton::BBtn),
                    EventSummary::Key(_, KeyCode::BTN_NORTH, 1) => Some(PadButton::XBtn),
                    EventSummary::Key(_, KeyCode::BTN_WEST, 1) => Some(PadButton::YBtn),
                    EventSummary::Key(_, KeyCode::BTN_START, 1) => Some(PadButton::StartBtn),
                    EventSummary::Key(_, KeyCode::BTN_SELECT, 1) => Some(PadButton::SelectBtn),
                    EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_HAT0X, -1) => {
                        Some(PadButton::Left)
                    }
                    EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_HAT0X, 1) => {
                        Some(PadButton::Right)
                    }
                    EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_HAT0Y, -1) => {
                        Some(PadButton::Up)
                    }
                    EventSummary::AbsoluteAxis(_, AbsoluteAxisCode::ABS_HAT0Y, 1) => {
                        Some(PadButton::Down)
                    }
                    //keyboard
                    EventSummary::Key(_, KeyCode::KEY_A, 1) => Some(PadButton::AKey),
                    EventSummary::Key(_, KeyCode::KEY_R, 1) => Some(PadButton::RKey),
                    EventSummary::Key(_, KeyCode::KEY_X, 1) => Some(PadButton::XKey),
                    EventSummary::Key(_, KeyCode::KEY_Z, 1) => Some(PadButton::ZKey),
                    //mouse
                    EventSummary::Key(_, KeyCode::BTN_RIGHT, 1) => Some(PadButton::RightClick),
                    _ => btn,
                };
            }
        }
        btn
    }
}

pub fn scan_input_devices(filter: &PadFilterType) -> Vec<InputDevice> {
    let mut pads: Vec<InputDevice> = Vec::new();
    for dev in evdev::enumerate() {
        let enabled = match filter {
            PadFilterType::All => true,
            PadFilterType::NoSteamInput => dev.1.input_id().vendor() != 0x28de,
            PadFilterType::OnlySteamInput => dev.1.input_id().vendor() == 0x28de,
        };

        let device_type = if dev
            .1
            .supported_keys()
            .map_or(false, |keys| keys.contains(KeyCode::BTN_SOUTH))
        {
            DeviceType::Gamepad
        } else if dev
            .1
            .supported_keys()
            .map_or(false, |keys| keys.contains(KeyCode::BTN_LEFT))
        {
            DeviceType::Mouse
        } else if dev
            .1
            .supported_keys()
            .map_or(false, |keys| keys.contains(KeyCode::KEY_SPACE))
        {
            DeviceType::Keyboard
        } else {
            DeviceType::Other
        };

        if device_type != DeviceType::Other {
            if dev.1.set_nonblocking(true).is_err() {
                println!("Failed to set non-blocking mode for {}", dev.0.display());
                continue;
            }
            let phys = dev.1.physical_path().unwrap_or_default().to_string();
            let uniq = dev.1.unique_name().unwrap_or_default().to_string();
            let device = dev.1;
            pads.push(InputDevice {
                path: dev.0.to_str().unwrap().to_string(),
                dev: device,
                phys,
                uniq,
                steam_name: None,
                enabled,
                device_type,
                has_button_held: false,
            });
        }
    }
    pads.sort_by_key(|pad| pad.path().to_string());
    pads
}

pub fn map_steam_inputs(devs: &[InputDevice]) -> std::collections::HashMap<usize, Vec<usize>> {
    use std::collections::HashMap;
    let mut map: HashMap<usize, Vec<usize>> = HashMap::new();
    for (i, dev) in devs.iter().enumerate() {
        if dev.is_steam_input() {
            if let Some((real_idx, _)) = devs
                .iter()
                .enumerate()
                .find(|(_, d)| !d.is_steam_input() && d.uniq() == dev.uniq() && !dev.uniq().is_empty())
            {
                map.entry(real_idx).or_default().push(i);
            } else if let Some((real_idx, _)) = devs
                .iter()
                .enumerate()
                .find(|(_, d)| !d.is_steam_input() && d.phys() == dev.phys() && !dev.phys().is_empty())
            {
                map.entry(real_idx).or_default().push(i);
            }
        }
    }
    map
}

pub fn fill_steam_names(devs: &mut [InputDevice]) {
    if let Ok(client) = steamworks::Client::init() {
        let input = client.input();
        input.run_frame();
        let handles = input.get_connected_controllers();
        for (sid, handle) in handles.iter().enumerate() {
            let name = match input.get_input_type_for_handle(*handle) {
                steamworks::InputType::PS4Controller => "PS4 Controller",
                steamworks::InputType::PS5Controller => "PS5 Controller",
                steamworks::InputType::XBoxOneController => "Xbox Controller",
                steamworks::InputType::XBox360Controller => "Xbox Controller",
                steamworks::InputType::SwitchProController => "Switch Pro",
                steamworks::InputType::SteamDeckController => "Steam Deck",
                _ => "Steam Input",
            };
            if let Some(dev) = devs.iter_mut().filter(|d| d.is_steam_input()).nth(sid) {
                dev.steam_name = Some(name.to_string());
            }
        }
    }
}
