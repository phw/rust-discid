use discid::DiscId;

fn main() {
    let toc = "1 11 242457 150 44942 61305 72755 96360 130485 147315 164275 190702 205412 220437";
    let result = DiscId::parse(toc);

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
    println!("First track : {}", disc.first_track_num());
    println!("Last track  : {}", disc.last_track_num());
    println!("Sectors     : {}\n", disc.sectors());

    for track in disc.tracks() {
        println!("Track #{}", track.number);
        println!("    Offset  : {}", track.offset);
        println!("    Sectors : {}", track.sectors);
    }
}
