use std::time::Duration;
use encdr::{Encdr, EncdrConfig, LedValue};

fn main() {
    let led = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "play".to_string());

    let mut encdr = Encdr::new(EncdrConfig {
        auto_hotplug: false,
        ..EncdrConfig::default()
    }).expect("Encdr initialization failed");

    let ids = encdr.scan().expect("Scan failed");

    let device_id = ids.into_iter().find(|&id| {
        encdr.device_descriptor(id)
            .map(|d| d.vendor_id.0 == 0x17cc && d.product_id.0 == 0x1140)
            .unwrap_or(false)
    }).expect("Maschine MK2 not found");

    println!("Maschine MK2 connectée.");

    // Éteindre toutes les LEDs connues.
    for i in 1..=16 {
        encdr.set_led(device_id, &format!("pad_{}", i), LedValue::Off);
    }

    for group in ["a", "b", "c", "d", "e", "f", "g", "h"] {
        encdr.set_led(device_id, &format!("group_{}", group), LedValue::Off);
    }

    let buttons = [
        "control", "step", "browser", "sampling",
        "arrow_left", "arrow_right", "all", "auto",
        "top_1", "top_2", "top_3", "top_4",
        "top_5", "top_6", "top_7", "top_8",
        "note_repeat", "enter", "nav_right", "nav_left",
        "tempo", "swing", "volume",
        "mute", "solo", "select", "duplicate",
        "navigate", "pad_mode", "pattern", "scene",
        "step_left", "step_right", "grid",
        "play", "rec", "shift", "erase"
    ];

    for button in buttons {
        encdr.set_led(device_id, button, LedValue::Off);
    }

    println!("Extinction des LEDs...");
    std::thread::sleep(Duration::from_secs(1));

    println!("Allumage de la LED {} pendant 5 secondes.", led);

    encdr.set_led(device_id, &led, LedValue::Single(127));

    std::thread::sleep(Duration::from_secs(5));

    encdr.set_led(device_id, &led, LedValue::Off);

    println!("Test terminé.");
}
