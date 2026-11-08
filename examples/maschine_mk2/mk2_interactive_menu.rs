use std::time::Duration;
use encdr::{Encdr, EncdrConfig, Event, PixelFormat};

mod mk2_font;
use mk2_font::draw_text;

fn render_menu(page: usize, parameter: usize) -> (Vec<u8>, Vec<u8>) {
    let mut left = vec![0u8; 2048];
    let mut right = vec![0u8; 2048];

    draw_text(&mut left, 10, 5, "MASCHINE MK2");
    draw_text(&mut left, 10, 20, "MAIN MENU");

    for (i, name) in PAGES.iter().enumerate() {
        let x = 10 + (i % 2) * 115;
        let y = 35 + (i / 2) * 12;
        let prefix = if i == page { ">" } else { " " };
        draw_text(&mut left, x, y, &format!("{}{}", prefix, name));
    }

    draw_text(&mut right, 10, 5, PAGES[page]);
    draw_text(&mut right, 10, 20, "PARAMETER");
    draw_text(&mut right, 10, 35, &format!("{}", parameter + 1));
    draw_text(&mut right, 10, 50, "TOP 5-8 NAVIGATION");

    (left, right)
}

const PAGES: [&str; 4] = ["PADS", "SCREENS", "MIDI", "SETTINGS"];

fn main() {
    let mut encdr = Encdr::new(EncdrConfig {
        auto_hotplug: false,
        ..EncdrConfig::default()
    }).expect("Initialisation Encdr impossible");

    let ids = encdr.scan().expect("Scan USB impossible");

    let device_id = ids.into_iter().find(|&id| {
        encdr.device_descriptor(id)
            .map(|d| d.vendor_id.0 == 0x17cc && d.product_id.0 == 0x1140)
            .unwrap_or(false)
    }).expect("Maschine MK2 introuvable");

    println!("Maschine MK2 connectée !");
    println!("TOP 1-4 : rubriques | TOP 5-6 : navigation");
    println!("TOP 7 : retour | TOP 8 : validation");
    println!("Ctrl+C pour quitter.\n");

    let events = encdr.events().clone();
    let mut page = 0usize;
    let mut parameter = 0usize;

    println!("Page actuelle : {}", PAGES[page]);

    let (left, right) = render_menu(page, parameter);
    encdr.submit_screen_with_format(device_id, "left", &left, PixelFormat::Mono);
    encdr.submit_screen_with_format(device_id, "right", &right, PixelFormat::Mono);

    loop {
        while let Ok(event) = events.try_recv() {
            if let Event::Button { device, name, pressed } = event {
                if device != device_id || !pressed {
                    continue;
                }

                match name {
                    "top_1" => { page = 0; parameter = 0; }
                    "top_2" => { page = 1; parameter = 0; }
                    "top_3" => { page = 2; parameter = 0; }
                    "top_4" => { page = 3; parameter = 0; }
                    "top_5" => { parameter = parameter.saturating_sub(1); }
                    "top_6" => { parameter = (parameter + 1) % 4; }
                    "top_7" => { page = 0; parameter = 0; }
                    "top_8" => {
                        println!("VALIDATION : {} / paramètre {}",
                            PAGES[page], parameter + 1);
                    }
                    _ => continue,
                }

                let (left, right) = render_menu(page, parameter);
                if encdr.screen_backlog(device_id) <= 1 {
                    encdr.submit_screen_with_format(device_id, "left", &left, PixelFormat::Mono);
                    encdr.submit_screen_with_format(device_id, "right", &right, PixelFormat::Mono);
                }

                println!("PAGE : {:8} | PARAMÈTRE : {} | BOUTON : {}",
                    PAGES[page], parameter + 1, name);
            }
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}
