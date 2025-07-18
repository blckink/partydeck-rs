// Steam Deck Native Input System
// Supports multiple mice, keyboards + Steam Deck built-in controls
// Optimized for 4-player split-screen gaming

use evdev::{Device, InputEventKind, EventType, Key, AbsoluteAxisType};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use serde::{Deserialize, Serialize};
use sysinfo::{System, SystemExt};
use udev::{Enumerator, MonitorBuilder};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SteamDeckPlayer {
    Player1, // Always Steam Deck built-in
    Player2, // First USB mouse + keyboard
    Player3, // Second USB mouse + keyboard  
    Player4, // USB controller or additional input
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteamDeckInputType {
    SteamDeckBuiltIn,    // Steam Deck gamepad + touchpads
    USBMouse(u8),        // USB mouse (1, 2, 3...)
    USBKeyboard(u8),     // USB keyboard (1, 2, 3...)
    USBController(u8),   // External USB controller
    Touchpad,           // Steam Deck touchpads as mouse alternative
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteamDeckButton {
    // Steam Deck native buttons
    A, B, X, Y,
    LeftBumper, RightBumper,
    LeftTrigger, RightTrigger,
    LeftStick, RightStick,
    LeftPad, RightPad,
    Menu, View, Steam,
    
    // Back buttons (Steam Deck specific)
    L4, L5, R4, R5,
    
    // Mouse buttons
    LeftClick, RightClick, MiddleClick,
    
    // Keyboard keys (most important for gaming)
    W, A, S, D, Space, Shift, Ctrl,
    Up, Down, Left, Right,
    Enter, Escape, Tab,
    F1, F2, F3, F4,
}

#[derive(Debug)]
pub struct SteamDeckInputDevice {
    device: Device,
    device_type: SteamDeckInputType,
    player: Option<SteamDeckPlayer>,
    vendor_id: u16,
    product_id: u16,
    path: String,
    name: String,
    is_steam_deck_hardware: bool,
}

impl SteamDeckInputDevice {
    pub fn new(device: Device, path: String) -> Self {
        let name = device.name().unwrap_or("Unknown").to_string();
        let input_id = device.input_id();
        let vendor_id = input_id.vendor();
        let product_id = input_id.product();
        
        // Detect Steam Deck hardware
        let is_steam_deck_hardware = Self::is_steam_deck_device(vendor_id, product_id, &name);
        
        // Classify device type
        let device_type = Self::classify_device(&device, is_steam_deck_hardware);
        
        Self {
            device,
            device_type,
            player: None,
            vendor_id,
            product_id,
            path,
            name,
            is_steam_deck_hardware,
        }
    }
    
    fn is_steam_deck_device(vendor_id: u16, product_id: u16, name: &str) -> bool {
        // Steam Deck Vendor ID: 0x28de (Valve Corporation)
        vendor_id == 0x28de || 
        name.contains("Steam") ||
        name.contains("Valve") ||
        name.contains("Neptune") // Steam Deck internal codename
    }
    
    fn classify_device(device: &Device, is_steam_deck: bool) -> SteamDeckInputType {
        let supported_events = device.supported_events();
        
        if is_steam_deck {
            return SteamDeckInputType::SteamDeckBuiltIn;
        }
        
        // Check if device has mouse capabilities
        if supported_events.contains(EventType::RELATIVE) ||
           supported_events.contains(EventType::KEY) && 
           device.supported_keys().map_or(false, |keys| 
               keys.contains(Key::BTN_LEFT) || keys.contains(Key::BTN_RIGHT)
           ) {
            // Count existing mice to assign number
            let mouse_num = 1; // TODO: Implement proper counting
            return SteamDeckInputType::USBMouse(mouse_num);
        }
        
        // Check if device has keyboard capabilities
        if supported_events.contains(EventType::KEY) &&
           device.supported_keys().map_or(false, |keys|
               keys.contains(Key::KEY_A) || keys.contains(Key::KEY_SPACE)
           ) {
            let keyboard_num = 1; // TODO: Implement proper counting
            return SteamDeckInputType::USBKeyboard(keyboard_num);
        }
        
        // Check if device has gamepad capabilities
        if supported_events.contains(EventType::ABSOLUTE) &&
           device.supported_absolute_axes().map_or(false, |axes|
               axes.contains(AbsoluteAxisType::ABS_X) || axes.contains(AbsoluteAxisType::ABS_Y)
           ) {
            let controller_num = 1; // TODO: Implement proper counting
            return SteamDeckInputType::USBController(controller_num);
        }
        
        SteamDeckInputType::USBController(0) // Fallback
    }
    
    pub fn get_name(&self) -> &str {
        &self.name
    }
    
    pub fn get_player(&self) -> Option<SteamDeckPlayer> {
        self.player
    }
    
    pub fn assign_player(&mut self, player: SteamDeckPlayer) {
        self.player = Some(player);
    }
    
    pub fn get_emoji(&self) -> &str {
        match self.device_type {
            SteamDeckInputType::SteamDeckBuiltIn => "🎮",
            SteamDeckInputType::USBMouse(_) => "🖱️",
            SteamDeckInputType::USBKeyboard(_) => "⌨️",
            SteamDeckInputType::USBController(_) => "🎮",
            SteamDeckInputType::Touchpad => "👆",
        }
    }
    
    pub fn poll(&mut self) -> Vec<(SteamDeckPlayer, SteamDeckButton)> {
        let mut events = Vec::new();
        
        if let Some(player) = self.player {
            // Read events from device
            if let Ok(device_events) = self.device.fetch_events() {
                for event in device_events {
                    if let Some(button) = self.map_event_to_button(&event) {
                        events.push((player, button));
                    }
                }
            }
        }
        
        events
    }
    
    fn map_event_to_button(&self, event: &evdev::InputEvent) -> Option<SteamDeckButton> {
        match event.event_type() {
            EventType::KEY => {
                match event.code() {
                    // Gamepad buttons
                    Key::BTN_SOUTH.code() => Some(SteamDeckButton::A),
                    Key::BTN_EAST.code() => Some(SteamDeckButton::B),
                    Key::BTN_NORTH.code() => Some(SteamDeckButton::X),
                    Key::BTN_WEST.code() => Some(SteamDeckButton::Y),
                    
                    // Mouse buttons
                    Key::BTN_LEFT.code() => Some(SteamDeckButton::LeftClick),
                    Key::BTN_RIGHT.code() => Some(SteamDeckButton::RightClick),
                    Key::BTN_MIDDLE.code() => Some(SteamDeckButton::MiddleClick),
                    
                    // Keyboard keys
                    Key::KEY_W.code() => Some(SteamDeckButton::W),
                    Key::KEY_A.code() => Some(SteamDeckButton::A),
                    Key::KEY_S.code() => Some(SteamDeckButton::S),
                    Key::KEY_D.code() => Some(SteamDeckButton::D),
                    Key::KEY_SPACE.code() => Some(SteamDeckButton::Space),
                    Key::KEY_LEFTSHIFT.code() => Some(SteamDeckButton::Shift),
                    Key::KEY_LEFTCTRL.code() => Some(SteamDeckButton::Ctrl),
                    
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

pub struct SteamDeckInputManager {
    devices: HashMap<String, SteamDeckInputDevice>,
    player_assignments: HashMap<SteamDeckPlayer, Vec<String>>, // Player -> Device paths
    is_steam_deck: bool,
    system_info: System,
}

impl SteamDeckInputManager {
    pub fn new() -> Self {
        let mut system_info = System::new_all();
        system_info.refresh_all();
        
        let is_steam_deck = Self::detect_steam_deck(&system_info);
        
        let mut manager = Self {
            devices: HashMap::new(),
            player_assignments: HashMap::new(),
            is_steam_deck,
            system_info,
        };
        
        // Initialize player assignment maps
        for player in [SteamDeckPlayer::Player1, SteamDeckPlayer::Player2, 
                      SteamDeckPlayer::Player3, SteamDeckPlayer::Player4] {
            manager.player_assignments.insert(player, Vec::new());
        }
        
        manager.scan_devices();
        manager.assign_players_smart();
        
        manager
    }
    
    fn detect_steam_deck(system: &System) -> bool {
        // Check if we're running on a Steam Deck
        system.host_name().unwrap_or("").contains("steamdeck") ||
        system.kernel_version().unwrap_or("").contains("valve") ||
        std::env::var("STEAMDECK").is_ok() ||
        Path::new("/sys/devices/virtual/dmi/id/product_name")
            .exists() && 
            fs::read_to_string("/sys/devices/virtual/dmi/id/product_name")
                .unwrap_or_default()
                .trim() == "Jupiter" // Steam Deck internal name
    }
    
    pub fn scan_devices(&mut self) {
        println!("🔍 Scanning for Steam Deck compatible input devices...");
        
        // Clear existing devices
        self.devices.clear();
        
        // Enumerate input devices
        let mut enumerator = Enumerator::new().unwrap();
        enumerator.match_subsystem("input").unwrap();
        
        for device in enumerator.scan_devices().unwrap() {
            if let Some(devnode) = device.devnode() {
                if let Some(path_str) = devnode.to_str() {
                    if path_str.contains("/dev/input/event") {
                        if let Ok(evdev_device) = Device::open(devnode) {
                            let steam_deck_device = SteamDeckInputDevice::new(
                                evdev_device, 
                                path_str.to_string()
                            );
                            
                            println!("  {} Found: {} - {}", 
                                steam_deck_device.get_emoji(),
                                steam_deck_device.get_name(),
                                match steam_deck_device.device_type {
                                    SteamDeckInputType::SteamDeckBuiltIn => "Steam Deck Built-in",
                                    SteamDeckInputType::USBMouse(n) => &format!("USB Mouse {}", n),
                                    SteamDeckInputType::USBKeyboard(n) => &format!("USB Keyboard {}", n),
                                    SteamDeckInputType::USBController(n) => &format!("USB Controller {}", n),
                                    SteamDeckInputType::Touchpad => "Touchpad",
                                }
                            );
                            
                            self.devices.insert(path_str.to_string(), steam_deck_device);
                        }
                    }
                }
            }
        }
        
        println!("✅ Found {} input devices", self.devices.len());
    }
    
    fn assign_players_smart(&mut self) {
        println!("🎯 Smart player assignment for Steam Deck split-screen...");
        
        // Clear existing assignments
        for assignments in self.player_assignments.values_mut() {
            assignments.clear();
        }
        
        // Priority 1: Steam Deck built-in controller always gets Player 1
        for (path, device) in &mut self.devices {
            if device.is_steam_deck_hardware {
                device.assign_player(SteamDeckPlayer::Player1);
                self.player_assignments.get_mut(&SteamDeckPlayer::Player1)
                    .unwrap().push(path.clone());
                println!("  🎮 Player 1: Steam Deck Built-in Controller");
                break;
            }
        }
        
        // Priority 2: Find pairs of USB mouse + keyboard
        let mut mouse_devices: Vec<&String> = Vec::new();
        let mut keyboard_devices: Vec<&String> = Vec::new();
        let mut controller_devices: Vec<&String> = Vec::new();
        
        for (path, device) in &self.devices {
            match device.device_type {
                SteamDeckInputType::USBMouse(_) => mouse_devices.push(path),
                SteamDeckInputType::USBKeyboard(_) => keyboard_devices.push(path),
                SteamDeckInputType::USBController(_) => controller_devices.push(path),
                _ => {}
            }
        }
        
        // Assign mouse + keyboard pairs
        let mut current_player = SteamDeckPlayer::Player2;
        for i in 0..std::cmp::min(mouse_devices.len(), keyboard_devices.len()) {
            if let (Some(player_assignment), Some(mouse_path), Some(keyboard_path)) = 
                (self.get_next_available_player(current_player), 
                 mouse_devices.get(i), 
                 keyboard_devices.get(i)) {
                
                // Assign mouse
                if let Some(mouse_device) = self.devices.get_mut(*mouse_path) {
                    mouse_device.assign_player(player_assignment);
                    self.player_assignments.get_mut(&player_assignment)
                        .unwrap().push(mouse_path.to_string());
                }
                
                // Assign keyboard
                if let Some(keyboard_device) = self.devices.get_mut(*keyboard_path) {
                    keyboard_device.assign_player(player_assignment);
                    self.player_assignments.get_mut(&player_assignment)
                        .unwrap().push(keyboard_path.to_string());
                }
                
                println!("  🖱️⌨️  Player {}: USB Mouse + Keyboard", 
                    match player_assignment {
                        SteamDeckPlayer::Player2 => 2,
                        SteamDeckPlayer::Player3 => 3,
                        SteamDeckPlayer::Player4 => 4,
                        _ => 0,
                    }
                );
                
                current_player = match current_player {
                    SteamDeckPlayer::Player2 => SteamDeckPlayer::Player3,
                    SteamDeckPlayer::Player3 => SteamDeckPlayer::Player4,
                    _ => break,
                };
            }
        }
        
        // Assign remaining controllers
        for controller_path in controller_devices {
            if let Some(player_assignment) = self.get_next_available_player(current_player) {
                if let Some(controller_device) = self.devices.get_mut(controller_path) {
                    controller_device.assign_player(player_assignment);
                    self.player_assignments.get_mut(&player_assignment)
                        .unwrap().push(controller_path.clone());
                    
                    println!("  🎮 Player {}: USB Controller", 
                        match player_assignment {
                            SteamDeckPlayer::Player2 => 2,
                            SteamDeckPlayer::Player3 => 3,
                            SteamDeckPlayer::Player4 => 4,
                            _ => 0,
                        }
                    );
                    
                    current_player = match current_player {
                        SteamDeckPlayer::Player2 => SteamDeckPlayer::Player3,
                        SteamDeckPlayer::Player3 => SteamDeckPlayer::Player4,
                        _ => break,
                    };
                }
            }
        }
        
        println!("✅ Player assignment complete!");
    }
    
    fn get_next_available_player(&self, start_from: SteamDeckPlayer) -> Option<SteamDeckPlayer> {
        let players = [SteamDeckPlayer::Player2, SteamDeckPlayer::Player3, SteamDeckPlayer::Player4];
        let start_index = match start_from {
            SteamDeckPlayer::Player2 => 0,
            SteamDeckPlayer::Player3 => 1,
            SteamDeckPlayer::Player4 => 2,
            _ => 0,
        };
        
        for &player in &players[start_index..] {
            if self.player_assignments.get(&player).unwrap().is_empty() {
                return Some(player);
            }
        }
        None
    }
    
    pub fn poll_all(&mut self) -> Vec<(SteamDeckPlayer, SteamDeckButton)> {
        let mut all_events = Vec::new();
        
        for device in self.devices.values_mut() {
            let events = device.poll();
            all_events.extend(events);
        }
        
        all_events
    }
    
    pub fn get_player_devices(&self, player: SteamDeckPlayer) -> Vec<String> {
        self.player_assignments.get(&player).cloned().unwrap_or_default()
    }
    
    pub fn get_device_count_by_type(&self) -> (usize, usize, usize, usize) {
        let mut steam_deck_count = 0;
        let mut mouse_count = 0;
        let mut keyboard_count = 0;
        let mut controller_count = 0;
        
        for device in self.devices.values() {
            match device.device_type {
                SteamDeckInputType::SteamDeckBuiltIn => steam_deck_count += 1,
                SteamDeckInputType::USBMouse(_) => mouse_count += 1,
                SteamDeckInputType::USBKeyboard(_) => keyboard_count += 1,
                SteamDeckInputType::USBController(_) => controller_count += 1,
                _ => {}
            }
        }
        
        (steam_deck_count, mouse_count, keyboard_count, controller_count)
    }
    
    pub fn is_running_on_steam_deck(&self) -> bool {
        self.is_steam_deck
    }
}