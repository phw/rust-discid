use discid::{DiscId, Features};

fn main() {
    let result = DiscId::read_features(None, Features::ISRC);

    match result {
        Ok(disc) => {
            println!("Disc ID: {}", disc.id());

            for track in disc.tracks() {
                println!("Track #{} ISRC: {}", track.number, track.isrc);
            }
        }
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
