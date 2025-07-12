use dualsense_rs::DualSense;
use std::sync::{Arc, Mutex};
use uinput::device::Device;
use uinput::event::controller::Mouse;
use uinput::event::relative::Position;

/// Representation of a DualSense controller mapped to a virtual mouse.
pub struct DualSenseMouse {
    pub name: String,
    #[allow(dead_code)]
    device: Arc<Mutex<Device>>, // keep device alive
    #[allow(dead_code)]
    controller: DualSense,
    #[allow(dead_code)]
    handle: std::thread::JoinHandle<()>,
    #[allow(dead_code)]
    _cbx: Arc<Box<dyn Fn(u16) + Send + Sync>>,
    #[allow(dead_code)]
    _cby: Arc<Box<dyn Fn(u16) + Send + Sync>>,
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
            Ok(p) => p.to_string(),
            Err(_) => continue,
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
        let cbx: Arc<Box<dyn Fn(u16) + Send + Sync>> = Arc::new(Box::new(move |x| {
            let mut p = pos_x.lock().unwrap();
            let dx = x as i32 - p.0 as i32;
            p.0 = x;
            let _ = ui_x.lock().unwrap().position(&Position::X, dx);
            let _ = ui_x.lock().unwrap().synchronize();
        }));
        // Safety: the Arc lives as long as DualSenseMouse
        ds.on_touchpad1_x_changed(unsafe {
            &*(Arc::as_ptr(&cbx) as *const Box<dyn Fn(u16) + Send + Sync>)
        });

        let cby: Arc<Box<dyn Fn(u16) + Send + Sync>> = Arc::new(Box::new(move |y| {
            let mut p = pos_y.lock().unwrap();
            let dy = y as i32 - p.1 as i32;
            p.1 = y;
            let _ = ui_y.lock().unwrap().position(&Position::Y, dy);
            let _ = ui_y.lock().unwrap().synchronize();
        }));
        ds.on_touchpad1_y_changed(unsafe {
            &*(Arc::as_ptr(&cby) as *const Box<dyn Fn(u16) + Send + Sync>)
        });
        let handle = ds.run();
        out.push(DualSenseMouse {
            name,
            device: ui,
            controller: ds,
            handle,
            _cbx: cbx,
            _cby: cby,
        });
        index += 1;
    }
    out
}
