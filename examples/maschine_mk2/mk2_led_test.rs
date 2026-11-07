use encdr::{Encdr, EncdrConfig, LedValue};
use std::{thread, time::Duration};

fn main() {
    let mut encdr = Encdr::new(EncdrConfig {
        auto_hotplug: false,
        ..Default::default()
    }).expect("Initialisation impossible");

    let ids = encdr.scan().expect("Scan impossible");
    let device_id = ids.into_iter().find(|&id| {
        encdr.device_descriptor(id)
            .map(|d| d.vendor_id.0 == 0x17cc && d.product_id.0 == 0x1140)
            .unwrap_or(false)
    }).expect("Maschine MK2 introuvable");

    let leds = [
        "restart", "step_left", "step_right", "grid",
        "play", "rec", "shift", "erase"
    ];

    for name in leds {
        for led in leds {
            encdr.set_led(device_id, led, LedValue::Single(0));
        }
        thread::sleep(Duration::from_millis(300));

        println!("TEST LED : {}", name);
        encdr.set_led(device_id, name, LedValue::Single(127));
        thread::sleep(Duration::from_secs(3));
    }

    for led in leds {
        encdr.set_led(device_id, led, LedValue::Single(0));
    }
    thread::sleep(Duration::from_millis(500));
    println!("Test termine.");
}
