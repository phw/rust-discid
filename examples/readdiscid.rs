use std::env;
use discid::{DiscId, Features};

fn main() {
    // Read the device name from the command line or use the default.
    let device = match env::args().nth(1) {
        Some(d) => Some(d),
        None => None,
    };

    let result = DiscId::read_features(device, Features::READ | Features::MCN);

    match result {
        Ok(disc) => print_disc_info(disc),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        },
    }
}

fn print_disc_info(disc: DiscId) {
    println!("DiscID   : {}", disc.get_id());
    println!("FreeDB ID: {}", disc.get_freedb_id());
    println!("TOC      : {}", disc.get_toc_string());
    println!("MCN      : {}", disc.get_mcn());
}
