use discid::DiscId;

fn main() {
    let offsets = [242457, 150, 44942, 61305, 72755, 96360, 130485, 147315, 164275,
                   190702, 205412, 220437];
    let result = DiscId::put(1, &offsets);

    match result {
        Ok(disc) => print_disc_info(disc),
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        },
    }
}

fn print_disc_info(disc: DiscId) {
    println!("DiscId: {}", disc.get_id());
    println!("TOC   : {}", disc.get_toc_string());
    println!("\nSubmit via {}", disc.get_submission_url());
}
