use std::time::Duration;
use encdr::{Encdr, EncdrConfig, PixelFormat};

fn set_pixel(buf: &mut [u8], x: usize, y: usize) {
    let index = y * 32 + x / 8;
    buf[index] |= 0x80 >> (x % 8);
}

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
    }).expect("Maschine Mk2 introuvable");

    println!("Maschine Mk2 détectée.");

    let mut left = vec![0u8; 2048];
    let mut right = vec![0u8; 2048];

    for y in 0..64 {
        for x in 0..256 {
            if (x / 16) % 2 == 0 {
                set_pixel(&mut left, x, y);
            }
            if (y / 8) % 2 == 0 {
                set_pixel(&mut right, x, y);
            }
        }
    }

println!("left: {:02X?} | right: {:02X?}", &left[..16], &right[..16]);
    encdr.submit_screen_with_format(device_id, "left", &left, PixelFormat::Mono);
    encdr.submit_screen_with_format(device_id, "right", &right, PixelFormat::Mono);

    println!("Deux images envoyées à Encdr. Capture en cours...");
    println!("Arrêt avec Ctrl+C.");

    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}
