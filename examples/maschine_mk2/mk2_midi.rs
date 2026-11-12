// Maschine MK2 Linux MIDI Controller
//
// Based on Encdr, developed by Rufus White.
// Original project: https://github.com/RufusIbiza/Encdr
//
// Encdr Copyright (C) 2026 Rufus White
// Licensed under GNU GPL v3.0 or later (GPL-3.0-or-later).
//
// MIDI adaptation for the Native Instruments Maschine MK2.

use encdr::{Encdr, EncdrConfig, Event, LedValue};
use midir::{MidiOutput, MidiOutputConnection};
use midir::os::unix::VirtualOutput;
use std::thread;
use std::time::{Duration, Instant};

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
    
    // Allumer les 16 pads en bleu au démarrage
	for i in 1..=16 {
    let pad_name = format!("pad_{}", i);

    encdr.set_led(
        device_id,
        &pad_name,
        LedValue::Rgb {
		r: 255,
		g: 0,
		b: 0,
},
    );
}

    println!("Maschine MK2 connectee !");
    println!("Port MIDI virtuel cree.");
    println!("Appuyez sur les pads. Ctrl+C pour quitter."); 
    let mut frappes: [Option<(Instant, f32)>; 16] = [None; 16];
    let mut notes_actives = [false; 16];

    // Groupes A a H : decalages en demi-tons
    let decalages_groupes: [i16; 8] = [
        0, 12, 24, 36, -12, -24, -36, 48
    ];

    // Groupe A selectionne au demarrage
    let mut groupe_actif: usize = 0;

    // Memoriser la note MIDI jouee par chaque pad
    let mut notes_jouees: [Option<u8>; 16] = [None; 16];

    let mut valeurs_cc = [64i32; 8];
    let mut volume_midi = 100i32;
	let mut last_led_update = Instant::now();
    
    loop {
        
        if last_led_update.elapsed() >= Duration::from_millis(30) {
    last_led_update = Instant::now();

		for i in 1..=16 {
        let pad_name = format!("pad_{}", i);

        encdr.set_led(
            device_id,
            &pad_name,
            LedValue::Rgb {
			r: 255,
			g: 0,
			b: 0,
},
        );
    }
}
        
        while let Ok(event) = events.try_recv() {

            // Mesurer la pression maximale pendant chaque frappe
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


            // Mesurer la pression maximale pendant chaque frappe
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
                if delta.abs() >= 1.0 {
                    let pas = delta.signum() as i32;

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

                // Selection des groupes A a H
                if pressed {
                    if let Some(lettre) = name.strip_prefix("group_") {
                        if let Some(index) = "abcdefgh".find(lettre) {
                            groupe_actif = index;

                            println!(
                                "Groupe {} selectionne : transposition {} demi-tons",
                                lettre.to_uppercase(),
                                decalages_groupes[groupe_actif]
                            );
                        }
                    }
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

                           if pressed {
                               if frappes[i].is_none() && !notes_actives[i] {
                                   frappes[i] = Some((Instant::now(), 0.0));
                               }
                           } else {
                               if let Some(note) = notes_jouees[i] {
                                   if let Err(e) = connexion.send(&[0x80, note, 0]) {
                                       eprintln!("Erreur MIDI : {}", e);
                                   }
                               }

                               frappes[i] = None;
                               notes_actives[i] = false;
                               notes_jouees[i] = None;
                           }
                            }
                        }
                    }
            }
        }

        // Declencher les notes apres 8 ms de mesure de pression
        for i in 0..16 {
            // Le pad 14 est physiquement defectueux
            if i == 13 || notes_actives[i] {
                continue;
            }

            if let Some((debut, maximum)) = frappes[i] {
                if debut.elapsed() >= Duration::from_millis(8) {
                    let velocite =
                        (maximum * 300.0).round().clamp(1.0, 127.0) as u8;

                    let note = (36i16 + i as i16
                        + decalages_groupes[groupe_actif]) as u8;

                    if let Err(e) = connexion.send(&[0x90, note, velocite]) {
                        eprintln!("Erreur MIDI : {}", e);
                    } else {
                        notes_actives[i] = true;
                        notes_jouees[i] = Some(note);
                        println!(
                            "Pad {} : note {} velocite {}",
                            i + 1,
                            note,
                            velocite
                        );
                    }
                }
            }
        }

        thread::sleep(Duration::from_millis(2));
    }
}
