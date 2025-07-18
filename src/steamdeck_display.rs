// Steam Deck Native Display Management
// Optimized for 1280x800 handheld + docked mode split-screen

use crate::steamdeck_input::{SteamDeckPlayer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SteamDeckDisplayMode {
    Handheld,    // 1280x800 built-in display
    Docked,      // External display via USB-C
    DualScreen,  // Both displays active
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SteamDeckResolution {
    pub width: u32,
    pub height: u32,
    pub refresh_rate: u32,
}

impl SteamDeckResolution {
    pub const HANDHELD: Self = Self { width: 1280, height: 800, refresh_rate: 60 };
    pub const DOCKED_1080P: Self = Self { width: 1920, height: 1080, refresh_rate: 60 };
    pub const DOCKED_1440P: Self = Self { width: 2560, height: 1440, refresh_rate: 60 };
    pub const DOCKED_4K: Self = Self { width: 3840, height: 2160, refresh_rate: 60 };
}

#[derive(Debug, Clone, Copy)]
pub struct SteamDeckViewport {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub player: SteamDeckPlayer,
}

impl SteamDeckViewport {
    pub fn center_x(&self) -> u32 {
        self.x + self.width / 2
    }
    
    pub fn center_y(&self) -> u32 {
        self.y + self.height / 2
    }
    
    pub fn contains_point(&self, x: u32, y: u32) -> bool {
        x >= self.x && x < self.x + self.width &&
        y >= self.y && y < self.y + self.height
    }
}

#[derive(Debug)]
pub struct SteamDeckDisplayManager {
    current_mode: SteamDeckDisplayMode,
    primary_resolution: SteamDeckResolution,
    secondary_resolution: Option<SteamDeckResolution>,
    viewports: HashMap<SteamDeckPlayer, SteamDeckViewport>,
    is_gamescope_active: bool,
    scaling_factor: f32,
}

impl SteamDeckDisplayManager {
    pub fn new() -> Self {
        let mut manager = Self {
            current_mode: SteamDeckDisplayMode::Handheld,
            primary_resolution: SteamDeckResolution::HANDHELD,
            secondary_resolution: None,
            viewports: HashMap::new(),
            is_gamescope_active: Self::detect_gamescope(),
            scaling_factor: 1.0,
        };
        
        manager.detect_display_mode();
        manager.setup_split_screen_viewports();
        
        manager
    }
    
    fn detect_gamescope() -> bool {
        // Check if we're running under gamescope
        std::env::var("GAMESCOPE_WAYLAND_DISPLAY").is_ok() ||
        std::env::var("GAMESCOPE").is_ok() ||
        std::process::Command::new("pgrep")
            .arg("gamescope")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    
    fn detect_display_mode(&mut self) {
        println!("🖥️  Detecting Steam Deck display configuration...");
        
        // Check for external displays
        let external_displays = self.scan_external_displays();
        
        match external_displays.len() {
            0 => {
                self.current_mode = SteamDeckDisplayMode::Handheld;
                self.primary_resolution = SteamDeckResolution::HANDHELD;
                println!("  📱 Handheld mode: 1280x800");
            }
            1 => {
                self.current_mode = SteamDeckDisplayMode::Docked;
                self.primary_resolution = external_displays[0];
                self.secondary_resolution = Some(SteamDeckResolution::HANDHELD);
                println!("  🖥️  Docked mode: {}x{}", 
                    self.primary_resolution.width, 
                    self.primary_resolution.height
                );
            }
            _ => {
                self.current_mode = SteamDeckDisplayMode::DualScreen;
                self.primary_resolution = external_displays[0];
                self.secondary_resolution = Some(external_displays[1]);
                println!("  🖥️📱 Dual screen mode");
            }
        }
        
        // Set scaling factor based on resolution
        self.scaling_factor = match self.primary_resolution {
            SteamDeckResolution::HANDHELD => 1.0,
            SteamDeckResolution::DOCKED_1080P => 1.5,
            SteamDeckResolution::DOCKED_1440P => 2.0,
            SteamDeckResolution::DOCKED_4K => 3.0,
            _ => 1.0,
        };
    }
    
    fn scan_external_displays(&self) -> Vec<SteamDeckResolution> {
        let mut displays = Vec::new();
        
        // Try to detect displays via drm/wayland
        if let Ok(output) = std::process::Command::new("wlr-randr")
            .output() {
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            for line in output_str.lines() {
                if line.contains("x") && line.contains("@") {
                    if let Some(resolution) = self.parse_resolution_string(line) {
                        displays.push(resolution);
                    }
                }
            }
        }
        
        // Fallback: Check via xrandr if available
        if displays.is_empty() {
            if let Ok(output) = std::process::Command::new("xrandr")
                .arg("--listmonitors")
                .output() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                
                for line in output_str.lines() {
                    if line.contains("/") && line.contains("x") {
                        if let Some(resolution) = self.parse_xrandr_line(line) {
                            displays.push(resolution);
                        }
                    }
                }
            }
        }
        
        displays
    }
    
    fn parse_resolution_string(&self, line: &str) -> Option<SteamDeckResolution> {
        // Parse "1920x1080@60Hz" format
        if let Some(res_part) = line.split('@').next() {
            if let Some((width_str, height_str)) = res_part.split_once('x') {
                if let (Ok(width), Ok(height)) = (width_str.parse(), height_str.parse()) {
                    return Some(SteamDeckResolution {
                        width,
                        height,
                        refresh_rate: 60, // Default
                    });
                }
            }
        }
        None
    }
    
    fn parse_xrandr_line(&self, line: &str) -> Option<SteamDeckResolution> {
        // Parse xrandr output format
        let parts: Vec<&str> = line.split_whitespace().collect();
        for part in parts {
            if part.contains("x") && part.contains("/") {
                if let Some((res_part, _)) = part.split_once("/") {
                    if let Some((width_str, height_str)) = res_part.split_once('x') {
                        if let (Ok(width), Ok(height)) = (width_str.parse(), height_str.parse()) {
                            return Some(SteamDeckResolution {
                                width,
                                height,
                                refresh_rate: 60,
                            });
                        }
                    }
                }
            }
        }
        None
    }
    
    fn setup_split_screen_viewports(&mut self) {
        println!("🔲 Setting up 4-player split-screen viewports...");
        
        let (base_width, base_height) = (self.primary_resolution.width, self.primary_resolution.height);
        
        // Calculate individual viewport dimensions
        let viewport_width = base_width / 2;
        let viewport_height = base_height / 2;
        
        // Create 2x2 grid layout
        let viewports = [
            // Top-left: Player 1 (Steam Deck built-in)
            SteamDeckViewport {
                x: 0,
                y: 0,
                width: viewport_width,
                height: viewport_height,
                player: SteamDeckPlayer::Player1,
            },
            // Top-right: Player 2
            SteamDeckViewport {
                x: viewport_width,
                y: 0,
                width: viewport_width,
                height: viewport_height,
                player: SteamDeckPlayer::Player2,
            },
            // Bottom-left: Player 3
            SteamDeckViewport {
                x: 0,
                y: viewport_height,
                width: viewport_width,
                height: viewport_height,
                player: SteamDeckPlayer::Player3,
            },
            // Bottom-right: Player 4
            SteamDeckViewport {
                x: viewport_width,
                y: viewport_height,
                width: viewport_width,
                height: viewport_height,
                player: SteamDeckPlayer::Player4,
            },
        ];
        
        // Store viewports in HashMap
        for viewport in viewports {
            self.viewports.insert(viewport.player, viewport);
            println!("  {} Player {}: {}x{} at ({}, {})",
                match viewport.player {
                    SteamDeckPlayer::Player1 => "🎮",
                    SteamDeckPlayer::Player2 => "🖱️",
                    SteamDeckPlayer::Player3 => "⌨️",
                    SteamDeckPlayer::Player4 => "🎮",
                },
                match viewport.player {
                    SteamDeckPlayer::Player1 => 1,
                    SteamDeckPlayer::Player2 => 2,
                    SteamDeckPlayer::Player3 => 3,
                    SteamDeckPlayer::Player4 => 4,
                },
                viewport.width,
                viewport.height,
                viewport.x,
                viewport.y
            );
        }
        
        println!("✅ Split-screen layout configured for {} mode", 
            match self.current_mode {
                SteamDeckDisplayMode::Handheld => "handheld",
                SteamDeckDisplayMode::Docked => "docked",
                SteamDeckDisplayMode::DualScreen => "dual-screen",
            }
        );
    }
    
    pub fn get_viewport(&self, player: SteamDeckPlayer) -> Option<&SteamDeckViewport> {
        self.viewports.get(&player)
    }
    
    pub fn get_all_viewports(&self) -> &HashMap<SteamDeckPlayer, SteamDeckViewport> {
        &self.viewports
    }
    
    pub fn get_display_mode(&self) -> SteamDeckDisplayMode {
        self.current_mode
    }
    
    pub fn get_primary_resolution(&self) -> SteamDeckResolution {
        self.primary_resolution
    }
    
    pub fn get_scaling_factor(&self) -> f32 {
        self.scaling_factor
    }
    
    pub fn is_handheld_mode(&self) -> bool {
        matches!(self.current_mode, SteamDeckDisplayMode::Handheld)
    }
    
    pub fn is_docked_mode(&self) -> bool {
        matches!(self.current_mode, SteamDeckDisplayMode::Docked | SteamDeckDisplayMode::DualScreen)
    }
    
    pub fn translate_mouse_coordinates(&self, x: u32, y: u32, player: SteamDeckPlayer) -> Option<(u32, u32)> {
        if let Some(viewport) = self.get_viewport(player) {
            // Translate global coordinates to viewport-relative coordinates
            if viewport.contains_point(x, y) {
                let relative_x = x - viewport.x;
                let relative_y = y - viewport.y;
                Some((relative_x, relative_y))
            } else {
                None
            }
        } else {
            None
        }
    }
    
    pub fn get_optimal_game_resolution(&self, player: SteamDeckPlayer) -> Option<(u32, u32)> {
        if let Some(viewport) = self.get_viewport(player) {
            // Return resolution optimized for the viewport
            Some((viewport.width, viewport.height))
        } else {
            None
        }
    }
    
    pub fn should_use_fsr(&self) -> bool {
        // Use FSR (FidelityFX Super Resolution) for performance boost
        // especially in docked mode with higher resolutions
        match self.current_mode {
            SteamDeckDisplayMode::Handheld => false, // Native resolution
            SteamDeckDisplayMode::Docked => {
                // Use FSR if docked resolution is significantly higher than native
                self.primary_resolution.width > 1280 || self.primary_resolution.height > 800
            }
            SteamDeckDisplayMode::DualScreen => true, // Always beneficial with dual screen
        }
    }
    
    pub fn get_recommended_graphics_settings(&self) -> SteamDeckGraphicsSettings {
        match self.current_mode {
            SteamDeckDisplayMode::Handheld => SteamDeckGraphicsSettings {
                resolution_scale: 1.0,
                texture_quality: TextureQuality::High,
                shadows: ShadowQuality::Medium,
                anti_aliasing: AntiAliasing::FXAA,
                use_fsr: false,
                framerate_target: 60,
            },
            SteamDeckDisplayMode::Docked => SteamDeckGraphicsSettings {
                resolution_scale: 0.8, // Render at lower res, upscale
                texture_quality: TextureQuality::Medium,
                shadows: ShadowQuality::Low,
                anti_aliasing: AntiAliasing::None,
                use_fsr: true,
                framerate_target: 30, // Prioritize stability over framerate in split-screen
            },
            SteamDeckDisplayMode::DualScreen => SteamDeckGraphicsSettings {
                resolution_scale: 0.7,
                texture_quality: TextureQuality::Low,
                shadows: ShadowQuality::Off,
                anti_aliasing: AntiAliasing::None,
                use_fsr: true,
                framerate_target: 30,
            },
        }
    }
    
    pub fn refresh_display_state(&mut self) {
        // Re-detect display configuration (for hotplug support)
        self.detect_display_mode();
        self.setup_split_screen_viewports();
    }
    
    pub fn get_performance_info(&self) -> SteamDeckPerformanceInfo {
        // TODO: Integrate with Steam Deck performance monitoring
        SteamDeckPerformanceInfo {
            cpu_usage: 0.0,
            gpu_usage: 0.0,
            ram_usage: 0.0,
            temperature: 0.0,
            battery_level: 100,
            power_draw: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SteamDeckGraphicsSettings {
    pub resolution_scale: f32,
    pub texture_quality: TextureQuality,
    pub shadows: ShadowQuality,
    pub anti_aliasing: AntiAliasing,
    pub use_fsr: bool,
    pub framerate_target: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum TextureQuality {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy)]
pub enum ShadowQuality {
    Off,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy)]
pub enum AntiAliasing {
    None,
    FXAA,
    MSAA2x,
    MSAA4x,
}

#[derive(Debug, Clone)]
pub struct SteamDeckPerformanceInfo {
    pub cpu_usage: f32,      // 0.0 - 100.0
    pub gpu_usage: f32,      // 0.0 - 100.0
    pub ram_usage: f32,      // 0.0 - 100.0
    pub temperature: f32,    // Celsius
    pub battery_level: u8,   // 0 - 100
    pub power_draw: f32,     // Watts
}