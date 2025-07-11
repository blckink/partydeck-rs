use dialog::{Choice, DialogBox};
use std::error::Error;
use std::path::PathBuf;

use crate::input::Player;
use crate::launch::MouseInfo;
use x11rb::connection::Connection;

pub fn msg(title: &str, contents: &str) {
    let _ = dialog::Message::new(contents).title(title).show();
}

pub fn yesno(title: &str, contents: &str) -> bool {
    if let Ok(prompt) = dialog::Question::new(contents).title(title).show() {
        if prompt == Choice::Yes {
            return true;
        }
    }
    false
}

pub fn get_screen_resolution() -> (u32, u32) {
    if let Ok(conn) = x11rb::connect(None) {
        let screen = &conn.0.setup().roots[0];
        println!(
            "Got screen resolution: {}x{}",
            screen.width_in_pixels, screen.height_in_pixels
        );
        return (
            screen.width_in_pixels as u32,
            screen.height_in_pixels as u32,
        );
    }
    // Fallback to a common resolution if detection fails
    println!("Failed to detect screen resolution, using fallback 1920x1080");
    (1920, 1080)
}

// Gets the resolution for a specific instance based on the number of instances
pub fn get_instance_resolution(
    playercount: usize,
    i: usize,
    basewidth: u32,
    baseheight: u32,
    two_player_vertical: bool,
) -> (u32, u32) {
    let (w, h) = match playercount {
        1 => (basewidth, baseheight),
        2 => {
            if two_player_vertical {
                (basewidth / 2, baseheight)
            } else {
                (basewidth, baseheight / 2)
            }
        }
        3 => {
            if i == 0 {
                (basewidth, baseheight / 2)
            } else {
                (basewidth / 2, baseheight / 2)
            }
        }
        4 => (basewidth / 2, baseheight / 2),
        // 5 => {
        //     if i < 2 {
        //         (basewidth / 2, baseheight / 2)
        //     } else {
        //         (basewidth / 3, baseheight / 2)
        //     }
        // }
        // 6 => (basewidth / 3, baseheight / 2),
        // 7 => {
        //     if i < 2 || i > 4 {
        //         (basewidth / 2, baseheight / 3)
        //     } else {
        //         (basewidth / 3, baseheight / 3)
        //     }
        // }
        // 8 => (basewidth / 2, baseheight / 4),
        _ => (basewidth, baseheight),
    };
    println!("Resolution for instance {}/{playercount}: {w}x{h}", i + 1);
    return (w, h);
}

// Sends the splitscreen script to the active KWin session through DBus
pub fn kwin_dbus_start_script(file: PathBuf) -> Result<(), Box<dyn Error>> {
    println!("Loading script {}...", file.display());
    if !file.exists() {
        return Err("Script file doesn't exist!".into());
    }

    let conn = zbus::blocking::Connection::session()?;
    let proxy = zbus::blocking::Proxy::new(
        &conn,
        "org.kde.KWin",
        "/Scripting",
        "org.kde.kwin.Scripting",
    )?;

    let _: i32 = proxy.call("loadScript", &(file.to_string_lossy(), "splitscreen"))?;
    println!("Script loaded. Starting...");
    let _: () = proxy.call("start", &())?;

    println!("KWin script started.");
    Ok(())
}

pub fn kwin_dbus_unload_script() -> Result<(), Box<dyn Error>> {
    println!("Unloading splitscreen script...");
    let conn = zbus::blocking::Connection::session()?;
    let proxy = zbus::blocking::Proxy::new(
        &conn,
        "org.kde.KWin",
        "/Scripting",
        "org.kde.kwin.Scripting",
    )?;

    let _: bool = proxy.call("unloadScript", &("splitscreen"))?;

    println!("Script unloaded.");
    Ok(())
}

pub fn assign_pointer(win_id: &str, device: &str) -> Result<(), Box<dyn Error>> {
    let name = format!("pd-{device}");
    let check = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("xinput list | grep -q '{name} pointer'"))
        .status()?;
    if !check.success() {
        let _ = std::process::Command::new("xinput")
            .arg("create-master")
            .arg(&name)
            .status();
    }

    let id_out = std::process::Command::new("sh")
        .arg("-c")
        .arg(format!("xinput list --id-only \"{device}\" 2>/dev/null"))
        .output()?;
    let id = String::from_utf8_lossy(&id_out.stdout).trim().to_string();
    if !id.is_empty() {
        let _ = std::process::Command::new("xinput")
            .args(["reattach", &id, &format!("{name} pointer")])
            .status();
        if !win_id.is_empty() {
            let _ = std::process::Command::new("xinput")
                .args(["set-cp", win_id, &format!("{name} pointer")])
                .status();
        }
    }
    Ok(())
}

pub fn auto_assign_mice(players: Vec<Player>, mice: Vec<MouseInfo>) {
    std::thread::spawn(move || {
        use std::time::Duration;
        for _ in 0..20 {
            let out = std::process::Command::new("xdotool")
                .args(["search", "--class", "gamescope"])
                .output();
            if let Ok(out) = out {
                let mut wins: Vec<String> =
                    String::from_utf8_lossy(&out.stdout).lines().map(|s| s.to_string()).collect();
                wins.sort();
                if wins.len() >= players.len() {
                    for (i, player) in players.iter().enumerate() {
                        if let Some(idx) = player.mouse_index {
                            if let Some(m) = mice.get(idx) {
                                let _ = assign_pointer(&wins[i], &m.name);
                            }
                        }
                    }
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(500));
        }
    });
}
