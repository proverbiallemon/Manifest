fn main() {
    let path = std::env::args().nth(1).expect("usage: list_real <file.otr>");
    let assets = manifest_core::formats::list_mpq_assets(std::path::Path::new(&path)).unwrap();
    println!("{}", assets.len());
    for a in assets.iter().take(5) { println!("{a}"); }
}
