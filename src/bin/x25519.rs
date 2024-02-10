use x22519::hex;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: x25519 <m> [<u>]");
        std::process::exit(1);
    }

    let m = hex::decodeScalar25519(&args[1]);

    let u = if args.len() == 3 {
        hex::decodeUCoordinate(&args[2])
    } else {
        hex::decodeUCoordinate(&"0900000000000000000000000000000000000000000000000000000000000000".to_string())
    };
    
    let p = &x22519::elliptic::P.clone();
    let a24:&num_bigint::BigInt = &(x22519::elliptic::A24.clone());

    let result = x22519::elliptic::ladder(&m, &u, p, a24);
    let result = &result.0 * num_bigint::BigInt::modpow(&result.1, &(p-num_bigint::BigInt::from(2)), p) % p;
    //let result = String::from(result.to_radix_be(16).1.to_ascii_lowercase());
    let result = result.to_bytes_be().1;
    for i in (0..result.len()).rev() {
        print!("{:x}", result[i]);
    }
    print!("\n");

}
