use x22519::hex;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: x25519 <m> [<u>]");
        std::process::exit(1);
    }
    dbg!(args[1].as_bytes());
    let m = hex::decode(args[1].as_bytes()).expect("Invalid hex-encoded string for m");
    dbg!(m.clone());

    let u = if args.len() == 3 {
        hex::decode(args[2].as_bytes()).expect("Invalid hex-encoded string for u")
    } else {
        hex::decode("09".as_bytes()).expect("Invalid hex-encoded string for base point")
    };

    /*
    let secret = StaticSecret::from(m);
    let public = PublicKey::from(&secret);

    let shared_secret = if u.is_empty() {
        secret.diffie_hellman(&PublicKey::from([9u8; 32]))
    } else {
        secret.diffie_hellman(&PublicKey::from(u))
    };
    */
    println!("Decoding functional");
    //println!("{}", hex::encode(shared_secret.to_bytes()));
}
