use discid::{DiscId, Features};
use std::env;

fn main() {
    // Read the device name from the command line or use the default.
    let arg1 = env::args().nth(1).unwrap_or(String::new());
    let device = if !arg1.is_empty() {
        Some(&arg1[..])
    } else {
        None
    };
    let result = DiscId::read_features(device, Features::ALL);

    match result {
        Ok(disc) => print_disc_info(disc),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}

fn print_disc_info(disc: DiscId) {
    println!("DiscID      : {}", disc.id());
    println!("FreeDB ID   : {}", disc.freedb_id());
    println!("TOC         : {}", disc.toc_string());
    println!("MCN         : {}", disc.mcn());
    println!("First track : {}", disc.first_track_num());
    println!("Last track  : {}", disc.last_track_num());
    println!("Sectors     : {}\n", disc.sectors());

    for i in 1..disc.last_track_num() + 1 {
        println!("Track #{}", i);
        println!("    ISRC   : {}", disc.track_isrc(i));
        println!("    Offset : {}", disc.track_offset(i));
        println!("    Sectors: {}", disc.track_length(i));
    }
}
