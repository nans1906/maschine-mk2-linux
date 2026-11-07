use encdr::{Encdr, EncdrConfig, LedValue};
use std::io::{self, BufRead};
use std::{thread, time::Duration};

fn main() {
    let mut encdr = Encdr::new(EncdrConfig {
        auto_hotplug: false,
        ..Default::default()
    }).expect("Impossible d'initialiser Encdr");

    let devices = encdr.scan().expect("Erreur de détection USB");

    let Some(&device_id) = devices.first() else {
        eprintln!("Aucune Maschine MK2 détectée");
        return;
    };

    println!("READY");

    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };

        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.len() != 4 {
            eprintln!("Commande invalide : {line}");
            continue;
        }

        let name = parts[0];

        let (Ok(r), Ok(g), Ok(b)) = (
            parts[1].parse::<u8>(),
            parts[2].parse::<u8>(),
            parts[3].parse::<u8>()
        ) else {
            eprintln!("Valeurs RGB invalides");
            continue;
        };

        encdr.set_led(
            device_id,
            name,
            LedValue::Rgb { r, g, b }
        );

        thread::sleep(Duration::from_millis(10));
    }
}
