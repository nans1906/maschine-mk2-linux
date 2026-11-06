use encdr::{Encdr, EncdrConfig, Event};
use midir::{MidiOutput, MidiOutputConnection};
use midir::os::unix::VirtualOutput;
use std::thread;
use std::time::Duration;

fn main() {
    let midi = MidiOutput::new("Maschine MK2")
        .expect("Impossible d'initialiser MIDI");

    let mut connexion: MidiOutputConnection = midi
        .create_virtual("Maschine MK2 MIDI")
        .expect("Impossible de creer le port MIDI");

    let mut encdr = Encdr::new(EncdrConfig {
        auto_hotplug: false,
        ..EncdrConfig::default()
    }).expect("Impossible d'initialiser Encdr");

    let ids = encdr.scan().expect("Erreur de detection USB");

    let device_id = ids.into_iter().find(|&id| {
        encdr.device_descriptor(id)
            .map(|d| d.vendor_id.0 == 0x17cc && d.product_id.0 == 0x1140)
            .unwrap_or(false)
    }).expect("Maschine MK2 introuvable");

    let events = encdr.events().clone();

    println!("Maschine MK2 connectee !");
    println!("Port MIDI virtuel cree.");
    println!("Appuyez sur les pads. Ctrl+C pour quitter."); 
    let mut valeurs_cc = [64i32; 8];
    let mut volume_midi = 100i32;

    
    loop {
        while let Ok(event) = events.try_recv() {
            
            if let Event::EncoderFine { name, delta, .. } = &event {
			// Gros potentiometre VOLUME : variations de +1 ou -1
if *name == "screen_encoder_1" && delta.abs() == 1.0 {
    volume_midi = (volume_midi + *delta as i32).clamp(0, 127);

    let message = [0xB0, 7, volume_midi as u8];

    if let Err(e) = connexion.send(&message) {
        eprintln!("Erreur MIDI volume : {}", e);
    }

    println!("VOLUME : CC 7 = {}", volume_midi);
}
    
    if let Some(numero) = name.strip_prefix("screen_encoder_") {
        if let Ok(index) = numero.parse::<usize>() {
            if (1..=8).contains(&index) {
                let i = index - 1;

// Ignore les petites variations parasites
                if delta.abs() >= 256.0 {
                    let pas = (*delta / 1024.0).round() as i32;

                    if pas != 0 {
                        valeurs_cc[i] = (valeurs_cc[i] + pas).clamp(0, 127);

                        // CC 20 a 27, canal MIDI 1
                        let message = [
                            0xB0,
                            20 + i as u8,
                            valeurs_cc[i] as u8,
                        ];

                        if let Err(e) = connexion.send(&message) {
                            eprintln!("Erreur MIDI CC : {}", e);
                        }

                        println!(
                            "Potentiometre {} : CC {} = {}",
                            index,
                            20 + i,
                            valeurs_cc[i]
                        );
                    }
                }
            }
        }
    }
}
            
            if let Event::Button { name, pressed, .. } = event {
				if name == "volume" {
				let valeur = if pressed { 127 } else { 0 };
				let message = [0xB0, 64, valeur];

				if let Err(e) = connexion.send(&message) {
				eprintln!("Erreur MIDI sustain : {}", e);
    }

					println!("SUSTAIN : CC64 = {}", valeur);
}
                if let Some(numero) = name.strip_prefix("pad_") {
                    if let Ok(pad) = numero.parse::<u8>() {
                        if (1..=16).contains(&pad) && pad != 14 {
                            let note = 35 + pad;
                            let message = if pressed {
                                [0x90, note, 100]
                            } else {
                                [0x80, note, 0]
                            };

                            if let Err(e) = connexion.send(&message) {
                                eprintln!("Erreur MIDI : {}", e);
                            }

                            println!(
                                "Pad {} : note {} {}",
                                pad,
                                note,
                                if pressed { "ON" } else { "OFF" }
                            );
                        }
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(2));
    }
}
