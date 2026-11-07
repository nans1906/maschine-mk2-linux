use encdr::{Encdr, EncdrConfig, Event, LedValue};
use midir::os::unix::VirtualOutput;
use midir::{MidiOutput, MidiOutputConnection};
use std::thread;
use std::time::{Duration, Instant};

fn main() {
    let midi = MidiOutput::new("Maschine MK2").expect("Impossible d'initialiser MIDI");

    let mut connexion: MidiOutputConnection = midi
        .create_virtual("Maschine MK2 MIDI")
        .expect("Impossible de creer le port MIDI");

    let mut encdr = Encdr::new(EncdrConfig {
        auto_hotplug: false,
        ..EncdrConfig::default()
    })
    .expect("Impossible d'initialiser Encdr");

    let ids = encdr.scan().expect("Erreur de detection USB");

    let device_id = ids
        .into_iter()
        .find(|&id| {
            encdr
                .device_descriptor(id)
                .map(|d| d.vendor_id.0 == 0x17cc && d.product_id.0 == 0x1140)
                .unwrap_or(false)
        })
        .expect("Maschine MK2 introuvable");

    let events = encdr.events().clone();

    // Allumer les 16 pads en bleu au démarrage
    for i in 1..=16 {
        let pad_name = format!("pad_{}", i);

        encdr.set_led(device_id, &pad_name, LedValue::Rgb { r: 255, g: 0, b: 0 });
    }

    println!("Maschine MK2 connectee !");
    println!("Port MIDI virtuel cree.");
    println!("Appuyez sur les pads. Ctrl+C pour quitter.");
    let mut frappes: [Option<(Instant, f32)>; 16] = [None; 16];
    let mut notes_actives = [false; 16];
    // LED TRANSPORT : PLAY vert, REC rouge
    encdr.set_led(device_id, "grid", LedValue::Single(127));
    encdr.set_led(device_id, "play", LedValue::Single(127));

    let mut valeurs_cc = [64i32; 8];
    let mut volume_midi = 100i32;
    let mut last_led_update = Instant::now();

    loop {
        if last_led_update.elapsed() >= Duration::from_millis(30) {
            last_led_update = Instant::now();

            for i in 1..=16 {
                let pad_name = format!("pad_{}", i);

                encdr.set_led(device_id, &pad_name, LedValue::Rgb { r: 255, g: 0, b: 0 });
            }
        }

        while let Ok(event) = events.try_recv() {
            if let Event::Grid { name, pressure, .. } = &event {
                if let Some(numero) = name.strip_prefix("pad_") {
                    if let Ok(pad) = numero.parse::<usize>() {
                        if (1..=16).contains(&pad) && pad != 14 {
                            let i = pad - 1;
                            if let Some((_, maximum)) = frappes[i].as_mut() {
                                *maximum = (*maximum).max(*pressure);
                            }
                        }
                    }
                }
            }

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
                                    let message = [0xB0, 20 + i as u8, valeurs_cc[i] as u8];

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
                // PLAY physique : commander LMMS
                if name == "grid" && pressed {
                    match std::process::Command::new("xdotool")
                        .args(["key", "--clearmodifiers", "space"])
                        .status()
                    {
                        Ok(status) if status.success() => println!("LMMS : PLAY/STOP"),
                        Ok(status) => eprintln!("xdotool : {}", status),
                        Err(e) => eprintln!("Erreur xdotool : {}", e),
                    }
                }

                // CC TRANSPORT
                if name == "grid" || name == "step_right" {
                    let cc = if name == "grid" { 110 } else { 111 };
                    let valeur = if pressed { 127 } else { 0 };

                    if let Err(e) = connexion.send(&[0xB0, cc, valeur]) {
                        eprintln!("Erreur MIDI transport : {}", e);
                    }

                    println!(
                        "TRANSPORT {} : CC {} = {}",
                        name, cc, valeur
                    );
                }

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
                            let i = (pad - 1) as usize;
                            let note = 35 + pad;

                            if pressed {
                                if frappes[i].is_none() && !notes_actives[i] {
                                    frappes[i] = Some((Instant::now(), 0.0));
                                }
                            } else {
                                if !notes_actives[i] {
                                    if let Some((_, maximum)) = frappes[i] {
                                        let velocite =
                                            (maximum * 300.0).round().clamp(1.0, 127.0) as u8;
                                        if let Err(e) = connexion.send(&[0x90, note, velocite]) {
                                            eprintln!("Erreur MIDI : {}", e);
                                        }
                                    }
                                }

                                if frappes[i].is_some() || notes_actives[i] {
                                    if let Err(e) = connexion.send(&[0x80, note, 0]) {
                                        eprintln!("Erreur MIDI : {}", e);
                                    }
                                }
                                frappes[i] = None;
                                notes_actives[i] = false;
                            }
                        }
                    }
                }
            }
        }

        // Declencher les notes apres 8 ms de mesure
        for i in 0..16 {
            if i == 13 || notes_actives[i] {
                continue;
            }

            if let Some((debut, maximum)) = frappes[i] {
                if debut.elapsed() >= Duration::from_millis(8) {
                    let velocite = (maximum * 300.0).round().clamp(1.0, 127.0) as u8;
                    let note = 36 + i as u8;

                    if let Err(e) = connexion.send(&[0x90, note, velocite]) {
                        eprintln!("Erreur MIDI : {}", e);
                    } else {
                        notes_actives[i] = true;
                        println!("Pad {} : note {} velocite {}", i + 1, note, velocite);
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(2));
    }
}
