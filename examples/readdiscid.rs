use std::env;
use discid::{DiscId, Features};

fn main() {
    // Read the device name from the command line or use the default.
    let device = match env::args().nth(1) {
        Some(d) => Some(d),
        None => None,
    };

    let result = DiscId::read_features(device, Features::ALL);

    match result {
        Ok(disc) => print_disc_info(disc),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        },
    }
}

fn print_disc_info(disc: DiscId) {
    println!("DiscID      : {}", disc.get_id());
    println!("FreeDB ID   : {}", disc.get_freedb_id());
    println!("TOC         : {}", disc.get_toc_string());
    println!("MCN         : {}", disc.get_mcn());
    println!("First track : {}", disc.get_first_track_num());
    println!("Last track  : {}", disc.get_last_track_num());
    println!("Sectors     : {}\n", disc.get_sectors());

    for i in 1..disc.get_last_track_num()+1 {
        println!("Track #{}", i);
        println!("    ISRC   : {}", disc.get_track_isrc(i));
        println!("    Offset : {}", disc.get_track_offset(i));
        println!("    Sectors: {}", disc.get_track_length(i));
    }
}
