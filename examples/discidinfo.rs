use discid::DiscId;

fn main() {
    println!("Version       : {}", DiscId::version_string());
    println!("Default device: {}", DiscId::default_device());
}
