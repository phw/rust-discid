use discid::DiscId;

fn main() {
    println!("Version       : {}", DiscId::get_version_string());
    println!("Default device: {}", DiscId::get_default_device());
}
