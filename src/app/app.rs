use crate::app::config::*;
use crate::game::{Game::*, *};
use crate::input::*;
use crate::launch::launch_game;
use crate::paths::*;
use crate::util::*;

use std::collections::HashMap;

use dialog::DialogBox;
use eframe::egui::RichText;
use eframe::egui::{self, Key, Ui};

#[derive(Eq, PartialEq)]
pub enum MenuPage {
    Home,
    Settings,
    Profiles,
    Game,
    Instances,
}

#[derive(Eq, PartialEq)]
pub enum SettingsPage {
    General,
    Gamescope,
}

pub struct PartyApp {
    pub needs_update: bool,
    pub options: PartyConfig,
    pub cur_page: MenuPage,
    pub settings_page: SettingsPage,
    pub infotext: String,

    pub input_devices: Vec<InputDevice>,
    pub steam_map: HashMap<usize, Vec<usize>>,
    pub instances: Vec<Instance>,
    pub instance_add_dev: Option<usize>,
    pub games: Vec<Game>,
    pub selected_game: usize,
    pub profiles: Vec<String>,

    pub loading_msg: Option<String>,
    pub loading_since: Option<std::time::Instant>,
    pub testing_device: Option<usize>,
    pub testing_last_btn: Option<String>,
    pub testing_start: Option<std::time::Instant>,
    #[allow(dead_code)]
    pub task: Option<std::thread::JoinHandle<()>>,
}

macro_rules! cur_game {
    ($self:expr) => {
        &$self.games[$self.selected_game]
    };
}

impl Default for PartyApp {
    fn default() -> Self {
        let options = load_cfg();
        let mut input_devices = scan_input_devices(&options.pad_filter_type);
        fill_steam_names(&mut input_devices);
        let steam_map = map_steam_inputs(&input_devices);
        Self {
            needs_update: check_for_partydeck_update(),
            options,
            cur_page: MenuPage::Home,
            settings_page: SettingsPage::General,
            infotext: String::new(),
            input_devices,
            steam_map,
            instances: Vec::new(),
            instance_add_dev: None,
            games: scan_all_games(),
            selected_game: 0,
            profiles: Vec::new(),
            loading_msg: None,
            loading_since: None,
            testing_device: None,
            testing_last_btn: None,
            testing_start: None,
            task: None,
        }
    }
}

impl eframe::App for PartyApp {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, raw_input: &mut egui::RawInput) {
        if !raw_input.focused {
            return;
        }
        match self.cur_page {
            MenuPage::Instances => self.handle_devices_instance_menu(),
            _ => self.handle_gamepad_gui(raw_input),
        }
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // TODO: We shouldn't run this every frame
        self.check_dependencies();

        egui::TopBottomPanel::top("menu_nav_panel").show(ctx, |ui| {
            if self.task.is_some() {
                ui.disable();
            }
            self.display_panel_top(ui);
        });

        egui::SidePanel::left("games_panel")
            .resizable(false)
            .exact_width(200.0)
            .show(ctx, |ui| {
                if self.task.is_some() {
                    ui.disable();
                }
                self.display_panel_left(ui);
            });

        if self.cur_page == MenuPage::Instances {
            egui::SidePanel::right("devices_panel")
                .resizable(false)
                .exact_width(180.0)
                .show(ctx, |ui| {
                    if self.task.is_some() {
                        ui.disable();
                    }
                    self.display_panel_right(ui);
                });
        }

        if (self.cur_page != MenuPage::Home) && (self.cur_page != MenuPage::Instances) {
            self.display_panel_bottom(ctx);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.task.is_some() {
                ui.disable();
            }
            match self.cur_page {
                MenuPage::Home => self.display_page_main(ui),
                MenuPage::Settings => self.display_page_settings(ui),
                MenuPage::Profiles => self.display_page_profiles(ui),
                MenuPage::Game => self.display_page_game(ui),
                MenuPage::Instances => self.display_page_instances(ui),
            }
        });

        if let Some(handle) = self.task.take() {
            if handle.is_finished() {
                let _ = handle.join();
                self.loading_since = None;
                self.loading_msg = None;
            } else {
                self.task = Some(handle);
            }
        }
        if let Some(start) = self.loading_since {
            if start.elapsed() > std::time::Duration::from_secs(60) {
                // Give up waiting after one minute
                self.loading_msg = Some("Operation timed out".to_string());
            }
        }
        if let Some(msg) = &self.loading_msg {
            egui::Area::new("loading".into())
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .interactable(false)
                .show(ctx, |ui| {
                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 192))
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::symmetric(16, 12))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.add(egui::widgets::Spinner::new().size(40.0));
                                ui.add_space(8.0);
                                ui.label(msg);
                            });
                        });
                });
        }
        if let Some(dev_idx) = self.testing_device {
            if let Some(btn) = self.input_devices[dev_idx].poll() {
                self.testing_last_btn = Some(format!("{:?}", btn));
                self.testing_start = Some(std::time::Instant::now());
            }
            if let Some(start) = self.testing_start {
                if start.elapsed() > std::time::Duration::from_secs(5) {
                    self.testing_device = None;
                }
            }
            egui::Area::new("testing".into())
                .anchor(egui::Align2::CENTER_TOP, egui::Vec2::new(0.0, 20.0))
                .show(ctx, |ui| {
                    egui::Frame::NONE
                        .fill(egui::Color32::from_rgba_premultiplied(0, 0, 0, 192))
                        .corner_radius(6.0)
                        .inner_margin(egui::Margin::symmetric(16, 12))
                        .show(ui, |ui| {
                            ui.vertical_centered(|ui| {
                                ui.label(format!("Testing {}", self.input_devices[dev_idx].fancyname()));
                                if let Some(b) = &self.testing_last_btn {
                                    ui.label(format!("Last: {b}"));
                                } else {
                                    ui.label("Press any button...");
                                }
                            });
                        });
                });
        }
        if ctx.input(|input| input.focused) {
            ctx.request_repaint_after(std::time::Duration::from_millis(33)); // 30 fps
        }
    }
}

impl PartyApp {
    fn spawn_task<F>(&mut self, msg: &str, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.loading_msg = Some(msg.to_string());
        self.loading_since = Some(std::time::Instant::now());
        self.task = Some(std::thread::spawn(f));
    }

    fn check_dependencies(&mut self) {
        if self.task.is_some() {
            return;
        }

        if !PATH_RES.join("umu-run").exists() {
            self.spawn_task("Downloading UMU Launcher...", || {
                if let Err(e) = update_umu_launcher() {
                    println!("Failed to download UMU Launcher: {}", e);
                    msg("Error", &format!("Failed to download UMU Launcher: {}", e));
                    let _ = std::fs::remove_file(PATH_RES.join("umu-run"));
                }
            });
        } else if !PATH_RES.join("goldberg_linux").exists()
            || !PATH_RES.join("goldberg_win").exists()
        {
            self.spawn_task("Downloading Goldberg Steam Emu...", || {
                if let Err(e) = update_goldberg_emu() {
                    println!("Failed to download Goldberg: {}", e);
                    msg("Error", &format!("Failed to download Goldberg: {}", e));
                    let _ = std::fs::remove_dir_all(PATH_RES.join("goldberg_linux"));
                    let _ = std::fs::remove_dir_all(PATH_RES.join("goldberg_win"));
                }
            });
        }
    }

    fn handle_gamepad_gui(&mut self, raw_input: &mut egui::RawInput) {
        let mut key: Option<egui::Key> = None;
        for pad in &mut self.input_devices {
            if !pad.enabled() {
                continue;
            }
            match pad.poll() {
                Some(PadButton::ABtn) => key = Some(Key::Enter),
                Some(PadButton::BBtn) => self.cur_page = MenuPage::Home,
                Some(PadButton::XBtn) => {
                    self.profiles = scan_profiles(false);
                    self.cur_page = MenuPage::Profiles;
                }
                Some(PadButton::YBtn) => self.cur_page = MenuPage::Settings,
                Some(PadButton::SelectBtn) => key = Some(Key::Tab),
                Some(PadButton::StartBtn) => {
                    if self.cur_page == MenuPage::Game {
                        self.instances.clear();
                        self.profiles = scan_profiles(true);
                        self.instance_add_dev = None;
                        self.cur_page = MenuPage::Instances;
                    }
                }
                Some(PadButton::Up) => key = Some(Key::ArrowUp),
                Some(PadButton::Down) => key = Some(Key::ArrowDown),
                Some(PadButton::Left) => key = Some(Key::ArrowLeft),
                Some(PadButton::Right) => key = Some(Key::ArrowRight),
                Some(_) => {}
                None => {}
            }
        }

        if let Some(key) = key {
            raw_input.events.push(egui::Event::Key {
                key,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::default(),
            });
        }
    }

    fn handle_devices_instance_menu(&mut self) {
        let mut i = 0;
        while i < self.input_devices.len() {
            if !self.input_devices[i].enabled() {
                i += 1;
                continue;
            }
            match self.input_devices[i].poll() {
                Some(PadButton::ABtn) | Some(PadButton::ZKey) | Some(PadButton::RightClick) => {
                    if self.input_devices[i].device_type() != DeviceType::Gamepad
                        && !self.options.kbm_support
                    {
                        continue;
                    }
                    if self.is_device_in_any_instance(i) {
                        continue;
                    }

                    match self.instance_add_dev {
                        Some(inst) => {
                            self.instance_add_dev = None;
                            self.instances[inst].devices.push(i);
                        }
                        None => {
                            self.instances.push(Instance {
                                devices: vec![i],
                                profname: String::new(),
                                profselection: 0,
                            });
                        }
                    }
                }
                Some(PadButton::BBtn) | Some(PadButton::XKey) => {
                    if self.instance_add_dev != None {
                        self.instance_add_dev = None;
                    } else if self.is_device_in_any_instance(i) {
                        self.remove_device(i);
                    } else if self.instances.len() < 1 {
                        self.cur_page = MenuPage::Game;
                    }
                }
                Some(PadButton::YBtn) | Some(PadButton::AKey) => {
                    if self.instance_add_dev == None {
                        if let Some((instance, _)) = self.find_device_in_instance(i) {
                            self.instance_add_dev = Some(instance);
                        }
                    }
                }
                Some(PadButton::StartBtn) => {
                    if self.instances.len() > 0 && self.is_device_in_any_instance(i) {
                        self.prepare_game_launch();
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    fn is_device_in_any_instance(&mut self, dev: usize) -> bool {
        for instance in &self.instances {
            if instance.devices.contains(&dev) {
                return true;
            }
        }
        false
    }

    fn find_device_in_instance(&mut self, dev: usize) -> Option<(usize, usize)> {
        for (i, instance) in self.instances.iter().enumerate() {
            for (d, device) in instance.devices.iter().enumerate() {
                if device == &dev {
                    return Some((i, d));
                }
            }
        }
        None
    }

    fn remove_device(&mut self, dev: usize) {
        if let Some((instance_index, device_index)) = self.find_device_in_instance(dev) {
            self.instances[instance_index].devices.remove(device_index);
            if self.instances[instance_index].devices.is_empty() {
                self.instances.remove(instance_index);
            }
        }
    }

    fn prepare_game_launch(&mut self) {
        let game = cur_game!(self).to_owned();
        let mut instances = self.instances.clone();
        let mut guests = GUEST_NAMES.to_vec();

        for instance in &mut instances {
            if instance.profselection == 0 {
                let i = fastrand::usize(..guests.len());
                instance.profname = format!(".{}", guests[i]);
                guests.swap_remove(i);
            } else {
                instance.profname = self.profiles[instance.profselection].to_owned();
            }
        }

        let dev_infos: Vec<DeviceInfo> = self
            .input_devices
            .iter()
            .map(|p| DeviceInfo {
                path: p.path().to_string(),
                vendor: p.vendor(),
                enabled: p.enabled(),
                device_type: p.device_type(),
            })
            .collect();

        let cfg = self.options.clone();
        let _ = save_cfg(&cfg);

        self.cur_page = MenuPage::Home;
        self.spawn_task("Launching...", move || {
            if let Err(err) = launch_game(&game, &dev_infos, &instances, &cfg) {
                println!("{}", err);
                msg("Launch Error", &format!("{err}"));
            }
        });
    }
}

impl PartyApp {
    fn display_page_main(&mut self, ui: &mut Ui) {
        ui.heading("Welcome to PartyDeck");
        ui.separator();
        ui.label("Press SELECT/BACK or Tab to unlock gamepad navigation.");
        ui.label("PartyDeck is in the very early stages of development; as such, you will likely encounter bugs, issues, and strange design decisions.");
        ui.label("For debugging purposes, it's recommended to read terminal output (stdout) for further information on errors.");
        ui.label("If you have found this software useful, consider donating to support further development!");
        ui.hyperlink_to("Ko-fi", "https://ko-fi.com/wunner");
        ui.label("If you've encountered issues or want to suggest improvements, criticism and feedback are always appreciated!");
        ui.hyperlink_to("GitHub", "https://github.com/wunnr/partydeck-rs");
    }

    fn display_page_settings(&mut self, ui: &mut Ui) {
        self.infotext.clear();
        ui.horizontal(|ui| {
            ui.heading("Settings");
            ui.selectable_value(&mut self.settings_page, SettingsPage::General, "General");
            ui.selectable_value(
                &mut self.settings_page,
                SettingsPage::Gamescope,
                "Gamescope",
            );
        });
        ui.separator();

        match self.settings_page {
            SettingsPage::General => self.display_settings_general(ui),
            SettingsPage::Gamescope => self.display_settings_gamescope(ui),
        }

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.horizontal(|ui| {
                if ui.button("Save Settings").clicked() {
                    if let Err(e) = save_cfg(&self.options) {
                        msg("Error", &format!("Couldn't save settings: {}", e));
                    }
                }
                if ui.button("Restore Defaults").clicked() {
                    self.options = PartyConfig {
                        force_sdl: false,
                        render_scale: 100,
                        enable_kwin_script: true,
                        gamescope_sdl_backend: true,
                        kbm_support: true,
                        proton_version: "".to_string(),
                        vertical_two_player: false,
                        pad_filter_type: PadFilterType::NoSteamInput,
                    };
                    self.input_devices = scan_input_devices(&self.options.pad_filter_type);
                    fill_steam_names(&mut self.input_devices);
                    self.steam_map = map_steam_inputs(&self.input_devices);
                }
            });
            ui.separator();
        });
    }

    fn display_page_profiles(&mut self, ui: &mut Ui) {
        ui.heading("Profiles");
        ui.separator();
        egui::ScrollArea::vertical()
            .max_height(ui.available_height() - 16.0)
            .auto_shrink(false)
            .show(ui, |ui| {
                for profile in &self.profiles {
                    if ui.selectable_value(&mut 0, 0, profile).clicked() {
                        if let Err(_) = std::process::Command::new("sh")
                            .arg("-c")
                            .arg(format!(
                                "xdg-open {}/profiles/{}",
                                PATH_PARTY.display(),
                                profile
                            ))
                            .status()
                        {
                            msg("Error", "Couldn't open profile directory!");
                        }
                    };
                }
            });
        if ui.button("New").clicked() {
            if let Some(name) = dialog::Input::new("Enter name (must be alphanumeric):")
                .title("New Profile")
                .show()
                .expect("Could not display dialog box")
            {
                if !name.is_empty() && name.chars().all(char::is_alphanumeric) {
                    create_profile(&name).unwrap();
                } else {
                    msg("Error", "Invalid name");
                }
            }
            self.profiles = scan_profiles(false);
        }
    }

    fn display_page_game(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.image(cur_game!(self).icon());
            ui.heading(cur_game!(self).name());
        });

        ui.separator();

        ui.horizontal(|ui| {
            ui.add(
                egui::Image::new(egui::include_image!("../../res/BTN_START.png")).max_height(16.0),
            );
            ui.add(
                egui::Image::new(egui::include_image!("../../res/BTN_START_PS5.png"))
                    .max_height(16.0),
            );
            if ui.button("Play").clicked() {
                self.instances.clear();
                self.profiles = scan_profiles(true);
                self.instance_add_dev = None;
                self.cur_page = MenuPage::Instances;
            }
            if let HandlerRef(h) = cur_game!(self) {
                ui.add(egui::Separator::default().vertical());
                if h.win {
                    ui.label(" Proton");
                } else {
                    ui.label("🐧 Native");
                }
                ui.add(egui::Separator::default().vertical());
                ui.label(format!("Author: {}", h.author));
                ui.add(egui::Separator::default().vertical());
                ui.label(format!("Version: {}", h.version));
            }
        });

        if let HandlerRef(h) = cur_game!(self) {
            egui::ScrollArea::horizontal()
                .max_width(f32::INFINITY)
                .show(ui, |ui| {
                    let available_height = ui.available_height();
                    ui.horizontal(|ui| {
                        for img in h.img_paths.iter() {
                            ui.add(
                                egui::Image::new(format!("file://{}", img.display()))
                                    .fit_to_exact_size(egui::vec2(
                                        available_height * 1.77,
                                        available_height,
                                    ))
                                    .maintain_aspect_ratio(true),
                            );
                        }
                    });
                });
        }
    }

    fn display_page_instances(&mut self, ui: &mut Ui) {
        ui.heading("Instances");
        ui.separator();

        ui.horizontal(|ui| {
            ui.add(
                egui::Image::new(egui::include_image!("../../res/BTN_SOUTH.png")).max_height(12.0),
            );
            ui.label("[Z]");
            ui.add(
                egui::Image::new(egui::include_image!("../../res/MOUSE_RIGHT.png"))
                    .max_height(12.0),
            );
            let add_text = match self.instance_add_dev {
                None => "New Instance",
                Some(i) => &format!("Add to Instance {}", i + 1),
            };
            ui.label(add_text);

            ui.label("      ");

            ui.add(
                egui::Image::new(egui::include_image!("../../res/BTN_EAST.png")).max_height(12.0),
            );
            ui.label("[X]");
            let remove_text = match self.instance_add_dev {
                None => "Remove",
                Some(_) => "Cancel",
            };
            ui.label(remove_text);
        });

        ui.separator();

        let mut devices_to_remove = Vec::new();
        for (i, instance) in &mut self.instances.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("Instance {}", i + 1));

                if let HandlerRef(_) = cur_game!(self) {
                    ui.label("👤");
                    egui::ComboBox::from_id_salt(format!("{i}")).show_index(
                        ui,
                        &mut instance.profselection,
                        self.profiles.len(),
                        |i| self.profiles[i].clone(),
                    );
                }

            });
            for (d_index, dev) in instance.devices.iter_mut().enumerate() {
                let mut dev_text = RichText::new(format!(
                    "{} {}",
                    self.input_devices[*dev].emoji(),
                    self.input_devices[*dev].fancyname()
                ));

                if self.input_devices[*dev].has_button_held() {
                    dev_text = dev_text.strong();
                }

            ui.horizontal(|ui| {
                ui.label("  ");
                // determine currently selected real device
                let mut real_sel = if let Some((r, _)) = self
                    .steam_map
                    .iter()
                    .find(|(_, v)| v.contains(dev))
                {
                    *r
                } else {
                    *dev
                };
                egui::ComboBox::from_id_salt(format!("real{}_{}", i, d_index))
                    .selected_text(format!("{} {}", self.input_devices[real_sel].emoji(), self.input_devices[real_sel].fancyname()))
                    .show_ui(ui, |ui| {
                        for (idx, p) in self.input_devices.iter().enumerate() {
                            if !p.is_steam_input() {
                                ui.selectable_value(&mut real_sel, idx, format!("{} {}", p.emoji(), p.fancyname()));
                            }
                        }
                    });

                let steam_list = self.steam_map.get(&real_sel).cloned().unwrap_or_default();
                let mut mode_idx = if let Some(pos) = steam_list.iter().position(|s| s == dev) {
                    pos + 1
                } else {
                    0
                };

                egui::ComboBox::from_id_salt(format!("mode{}_{}", i, d_index))
                    .selected_text(match mode_idx {
                        0 => "Native".to_string(),
                        n => format!("{}", self.input_devices[steam_list[n - 1]].fancyname()),
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut mode_idx, 0, "Native");
                        for (idx, sid) in steam_list.iter().enumerate() {
                            ui.selectable_value(&mut mode_idx, idx + 1, self.input_devices[*sid].fancyname());
                        }
                    });

                *dev = if mode_idx == 0 { real_sel } else { steam_list[mode_idx - 1] };

                if ui.button("🗑").clicked() {
                    devices_to_remove.push(*dev);
                }
            });
            }
        }

        for d in devices_to_remove {
            self.remove_device(d);
        }

        if self.instances.len() > 0 {
            ui.separator();
            ui.horizontal(|ui| {
                ui.add(
                    egui::Image::new(egui::include_image!("../../res/BTN_START.png"))
                        .max_height(16.0),
                );
                ui.add(
                    egui::Image::new(egui::include_image!("../../res/BTN_START_PS5.png"))
                        .max_height(16.0),
                );
                if ui.button("Start").clicked() {
                    self.prepare_game_launch();
                }
            });
        }
    }

    fn display_settings_general(&mut self, ui: &mut Ui) {
        let force_sdl2_check = ui.checkbox(&mut self.options.force_sdl, "Force Steam Runtime SDL2");

        let enable_kwin_script_check = ui.checkbox(
            &mut self.options.enable_kwin_script,
            "Automatically resize/reposition instances",
        );

        let vertical_two_player_check = ui.checkbox(
            &mut self.options.vertical_two_player,
            "Vertical split for 2 players",
        );

        if force_sdl2_check.hovered() {
            self.infotext = "Forces games to use the version of SDL2 included in the Steam Runtime. Only works on native Linux games, may fix problematic game controller support (incorrect mappings) in some games, may break others. If unsure, leave this unchecked.".to_string();
        }

        if enable_kwin_script_check.hovered() {
            self.infotext = "Resizes/repositions instances to fit the screen using a KWin script. If unsure, leave this checked. If using a desktop environment or window manager other than KDE Plasma, uncheck this; note that you will need to manually resize and reposition the windows.".to_string();
        }

        if vertical_two_player_check.hovered() {
            self.infotext =
                "Splits two-player games vertically (side by side) instead of horizontally."
                    .to_string();
        }

        ui.horizontal(|ui| {
        let proton_ver_label = ui.label("Proton version");
        let proton_ver_editbox = ui.add(
            egui::TextEdit::singleline(&mut self.options.proton_version)
                .hint_text("GE-Proton"),
        );
        if proton_ver_label.hovered() || proton_ver_editbox.hovered() {
            self.infotext = "Specify a Proton version. This can be a path, e.g. \"/path/to/proton\" or just a name, e.g. \"GE-Proton\" for the latest version of Proton-GE. If left blank, this will default to \"GE-Proton\". If unsure, leave this blank.".to_string();
        }
        });

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Update/Redownload Dependencies");
            if ui.button("Goldberg Steam Emu").clicked() {
                self.spawn_task("Updating Goldberg Steam Emu...", || {
                    if let Err(err) = update_goldberg_emu() {
                        msg("Error", &format!("Couldn't update: {}", err));
                    }
                });
            }
            if ui.button("UMU Launcher").clicked() {
                self.spawn_task("Updating UMU Launcher...", || {
                    if let Err(err) = update_umu_launcher() {
                        msg("Error", &format!("Couldn't update: {}", err));
                    }
                });
            }
        });

        ui.horizontal(|ui| {
        if ui.button("Erase Proton Prefix").clicked() {
            if yesno("Erase Prefix?", "This will erase the Wine prefix used by PartyDeck. This shouldn't erase profile/game-specific data, but exercise caution. Are you sure?") && PATH_PARTY.join("gamesyms").exists() {
                if let Err(err) = std::fs::remove_dir_all(PATH_PARTY.join("pfx")) {
                    msg("Error", &format!("Couldn't erase pfx data: {}", err));
                }
                else if let Err(err) = std::fs::create_dir_all(PATH_PARTY.join("pfx")) {
                    msg("Error", &format!("Couldn't re-create pfx directory: {}", err));
                }
                else {
                    msg("Data Erased", "Proton prefix data successfully erased.");
                }
            }
        }

        if ui.button("Erase Symlink Data").clicked() {
            if yesno("Erase Symlink Data?", "This will erase all game symlink data. This shouldn't erase profile/game-specific data, but exercise caution. Are you sure?") && PATH_PARTY.join("gamesyms").exists() {
                if let Err(err) = std::fs::remove_dir_all(PATH_PARTY.join("gamesyms")) {
                    msg("Error", &format!("Couldn't erase symlink data: {}", err));
                }
                else if let Err(err) = std::fs::create_dir_all(PATH_PARTY.join("gamesyms")) {
                    msg("Error", &format!("Couldn't re-create symlink directory: {}", err));
                }
                else {
                    msg("Data Erased", "Game symlink data successfully erased.");
                }
            }
        }
        });

        ui.horizontal(|ui| {
            if ui.button("Open PartyDeck Data Folder").clicked() {
                if let Err(_) = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("xdg-open {}/", PATH_PARTY.display()))
                    .status()
                {
                    msg("Error", "Couldn't open PartyDeck Data Folder!");
                }
            }
            if ui.button("Edit game paths").clicked() {
                if let Err(_) = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!("xdg-open {}/paths.json", PATH_PARTY.display(),))
                    .status()
                {
                    msg("Error", "Couldn't open paths.json!");
                }
            }
        });
    }

    fn display_settings_gamescope(&mut self, ui: &mut Ui) {
        let render_scale_slider = ui.add(
            egui::Slider::new(&mut self.options.render_scale, 35..=200)
                .text("Instance resolution scale"),
        );
        let gamescope_sdl_backend_check = ui.checkbox(
            &mut self.options.gamescope_sdl_backend,
            "Use SDL backend for Gamescope",
        );
        let kbm_support_check = ui.checkbox(
            &mut self.options.kbm_support,
            "Enable keyboard and mouse support through custom Gamescope",
        );

        if render_scale_slider.hovered() {
            self.infotext = "PartyDeck divides each instance by a base resolution. 100% render scale = your monitor's native resolution. Lower this value to increase performance, but may cause graphical issues or even break some games. If you're using a small screen like the Steam Deck's handheld screen, increase this to 150% or higher.".to_string();
        }
        if gamescope_sdl_backend_check.hovered() {
            self.infotext = "Runs gamescope sessions using the SDL backend. If unsure, leave this checked. If gamescope sessions only show a black screen or give an error (especially on Nvidia + Wayland), try disabling this.".to_string();
        }
        if kbm_support_check.hovered() {
            self.infotext = "Runs a custom Gamescope build with support for holding keyboards and mice. If you want to use your own Gamescope installation, uncheck this.".to_string();
        }
    }
}

impl PartyApp {
    fn display_panel_top(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.add(
                egui::Image::new(egui::include_image!("../../res/BTN_EAST.png")).max_height(12.0),
            );
            ui.selectable_value(&mut self.cur_page, MenuPage::Home, "Home");
            ui.add(
                egui::Image::new(egui::include_image!("../../res/BTN_NORTH.png")).max_height(12.0),
            );
            ui.selectable_value(&mut self.cur_page, MenuPage::Settings, "Settings");
            ui.add(
                egui::Image::new(egui::include_image!("../../res/BTN_WEST.png")).max_height(12.0),
            );
            if ui
                .selectable_value(&mut self.cur_page, MenuPage::Profiles, "Profiles")
                .clicked()
            {
                self.profiles = scan_profiles(false);
                self.cur_page = MenuPage::Profiles;
            }

            if ui.button("🎮 Rescan").clicked() {
                self.instances.clear();
                self.input_devices = scan_input_devices(&self.options.pad_filter_type);
                fill_steam_names(&mut self.input_devices);
                self.steam_map = map_steam_inputs(&self.input_devices);
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❌ Quit").clicked() {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
                let version_label = match self.needs_update {
                    true => format!("v{} (Update Available)", env!("CARGO_PKG_VERSION")),
                    false => format!("v{}", env!("CARGO_PKG_VERSION")),
                };
                ui.hyperlink_to(
                    version_label,
                    "https://github.com/wunnr/partydeck-rs/releases",
                );
                ui.add(egui::Separator::default().vertical());
                ui.hyperlink_to(
                    "Open Source Licenses",
                    "https://github.com/wunnr/partydeck-rs/tree/main?tab=License-2-ov-file",
                );
            });
        });
    }

    fn display_panel_left(&mut self, ui: &mut Ui) {
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.heading("Games");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("➕").clicked() {
                    if let Err(err) = add_game() {
                        println!("Couldn't add game: {err}");
                        msg("Error", &format!("Couldn't add game: {err}"));
                    }
                    let dir_tmp = PATH_PARTY.join("tmp");
                    if dir_tmp.exists() {
                        std::fs::remove_dir_all(&dir_tmp).unwrap();
                    }
                    self.games = crate::game::scan_all_games();
                }
                if ui.button("🔄").clicked() {
                    self.games = crate::game::scan_all_games();
                }
            });
        });
        ui.separator();
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.panel_left_game_list(ui);
        });
    }

    fn display_panel_bottom(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("info_panel")
            .exact_height(100.0)
            .show(ctx, |ui| {
                if self.task.is_some() {
                    ui.disable();
                }
                match self.cur_page {
                    MenuPage::Game => {
                        match cur_game!(self){
                            Game::Executable { path, .. } =>
                                self.infotext = format!("{}", path.display()),
                            Game::HandlerRef(h) =>
                                self.infotext = h.info.to_owned(),
                        }
                    }
                    MenuPage::Profiles =>
                        self.infotext = "Create profiles to persistently store game save data, settings, and stats.".to_string(),
                    _ => {}
                }
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.label(&self.infotext);
                });
            });
    }

    fn display_panel_right(&mut self, ui: &mut Ui) {
        ui.add_space(6.0);

        ui.heading("Devices");
        ui.separator();

        let mut deck_indices: Vec<usize> = Vec::new();
        for (i, pad) in self.input_devices.iter().enumerate() {
            if pad.is_steam_input() {
                continue;
            }
            if pad.vendor() == 0x28de {
                deck_indices.push(i);
                continue;
            }

            let mut dev_text = RichText::new(format!(
                "{} {} ({})",
                pad.emoji(),
                pad.name(),
                pad.path()
            ))
            .small();

            if !pad.enabled() {
                dev_text = dev_text.weak();
            } else if pad.has_button_held() {
                dev_text = dev_text.strong();
            }

            ui.label(dev_text);

            if let Some(stds) = self.steam_map.get(&i) {
                for sid in stds {
                    let sdev = &self.input_devices[*sid];
                    ui.label(RichText::new(format!("  ↳ {} {}", sdev.emoji(), sdev.fancyname())).small());
                }
            }
        }

        if !deck_indices.is_empty() {
            ui.label(RichText::new("Steam Deck").strong());
            for i in deck_indices {
                let pad = &self.input_devices[i];
                let mut dev_text = RichText::new(format!("  {} {} ({})", pad.emoji(), pad.name(), pad.path())).small();
                if !pad.enabled() {
                    dev_text = dev_text.weak();
                } else if pad.has_button_held() {
                    dev_text = dev_text.strong();
                }
                ui.label(dev_text);
                if let Some(stds) = self.steam_map.get(&i) {
                    for sid in stds {
                        let sdev = &self.input_devices[*sid];
                        ui.label(RichText::new(format!("    ↳ {} {}", sdev.emoji(), sdev.fancyname())).small());
                    }
                }
            }
        }
    }

    fn panel_left_game_list(&mut self, ui: &mut Ui) {
        let mut refresh_games = false;

        for (i, game) in self.games.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.add(
                    egui::Image::new(game.icon())
                        .max_width(16.0)
                        .corner_radius(2),
                );
                let btn = ui.selectable_value(&mut self.selected_game, i, game.name());
                if btn.has_focus() {
                    btn.scroll_to_me(None);
                }
                if btn.clicked() {
                    self.cur_page = MenuPage::Game;
                };

                let popup_id = ui.make_persistent_id(format!("gamectx{}", i));

                egui::popup::popup_below_widget(
                    ui,
                    popup_id,
                    &btn,
                    egui::popup::PopupCloseBehavior::CloseOnClick,
                    |ui| {
                        if ui.button("Remove").clicked() {
                            if yesno(
                                "Remove game?",
                                &format!("Are you sure you want to remove {}?", game.name()),
                            ) {
                                if let Err(err) = remove_game(&self.games[i]) {
                                    println!("Failed to remove game: {}", err);
                                    msg("Error", &format!("Failed to remove game: {}", err));
                                }
                            }
                            refresh_games = true;
                        }
                        if let HandlerRef(h) = game {
                            if ui.button("Open Handler Folder").clicked() {
                                if let Err(_) = std::process::Command::new("sh")
                                    .arg("-c")
                                    .arg(format!("xdg-open {}", h.path_handler.display()))
                                    .status()
                                {
                                    msg("Error", "Couldn't open handler folder!");
                                }
                            }
                        }
                    },
                );

                if btn.secondary_clicked() {
                    ui.memory_mut(|mem| mem.toggle_popup(popup_id));
                }
            });
        }
        // Hacky workaround to avoid borrowing conflicts from inside the loop
        if refresh_games {
            self.games = scan_all_games();
        }
    }
}

static GUEST_NAMES: [&str; 21] = [
    "Blinky", "Pinky", "Inky", "Clyde", "Beatrice", "Battler", "Ellie", "Joel", "Leon", "Ada",
    "Madeline", "Theo", "Yokatta", "Wyrm", "Brodiee", "Supreme", "Conk", "Gort", "Lich", "Smores",
    "Canary",
];
