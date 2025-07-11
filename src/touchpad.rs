use std::sync::{Arc, Mutex};
use dualsense_rs::DualSense;
use uinput::device::Device;
use uinput::event::relative::Position;
use uinput::event::controller::Mouse;

/// Representation of a DualSense controller mapped to a virtual mouse.
pub struct DualSenseMouse {
    pub name: String,
    #[allow(dead_code)]
    device: Arc<Mutex<Device>>, // keep device alive
    #[allow(dead_code)]
    controller: DualSense,
    #[allow(dead_code)]
    handle: std::thread::JoinHandle<()>,
}

/// Scan for connected DualSense controllers and spawn a uinput mouse for each.
pub fn spawn_dualsense_mice() -> Vec<DualSenseMouse> {
    let mut out = Vec::new();
    let mut index = 0;
    for info in DualSense::list_devices() {
        if info.vendor_id() != 0x054c {
            continue;
        }
        let path = match info.path().to_str() {
            Some(p) => p.to_string(),
            None => continue,
        };
        let mut ds = DualSense::new_path(&path);
        let name = format!("pd-dualsense-mouse-{}", index);
        let ui = uinput::default()
            .and_then(|dev| {
                dev.name(&name)
                    .unwrap()
                    .event(Mouse::Left)
                    .unwrap()
                    .event(Mouse::Right)
                    .unwrap()
                    .event(Position::X)
                    .unwrap()
                    .event(Position::Y)
                    .unwrap()
                    .create()
            })
            .expect("failed to create uinput device");
        let ui = Arc::new(Mutex::new(ui));
        let ui_x = ui.clone();
        let ui_y = ui.clone();
        let pos = Arc::new(Mutex::new((0u16, 0u16)));
        let pos_x = pos.clone();
        let pos_y = pos.clone();
        ds.on_touchpad1_x_changed(&move |x| {
            let mut p = pos_x.lock().unwrap();
            let dx = x as i32 - p.0 as i32;
            p.0 = x;
            let _ = ui_x.lock().unwrap().position(&Position::X, dx);
            let _ = ui_x.lock().unwrap().synchronize();
        });
        ds.on_touchpad1_y_changed(&move |y| {
            let mut p = pos_y.lock().unwrap();
            let dy = y as i32 - p.1 as i32;
            p.1 = y;
            let _ = ui_y.lock().unwrap().position(&Position::Y, dy);
            let _ = ui_y.lock().unwrap().synchronize();
        });
        let handle = ds.run();
        out.push(DualSenseMouse {
            name,
            device: ui,
            controller: ds,
            handle,
        });
        index += 1;
    }
    out
}
