use encdr::{Encdr, EncdrConfig, LedValue};
use std::{thread, time::Duration};

fn main() {
    let mut encdr = Encdr::new(EncdrConfig {
        auto_hotplug: false,
        ..Default::default()
    }).expect("Impossible d'initialiser Encdr");

    let devices = encdr.scan().expect("Erreur de détection USB");
    let Some(&device_id) = devices.first() else {
        eprintln!("Aucun périphérique détecté");
        return;
    };

    let colors = [
        ("ROUGE", 127, 0, 0),
        ("VERT", 0, 127, 0),
        ("BLEU", 0, 0, 127),
    ];

    for (name, r, g, b) in colors {
        println!("Test : {name}");
        encdr.set_led(device_id, "pad_1", LedValue::Rgb { r, g, b });
        thread::sleep(Duration::from_secs(3));
    }

    encdr.set_led(device_id, "pad_1", LedValue::Off);
    thread::sleep(Duration::from_secs(1));
}
