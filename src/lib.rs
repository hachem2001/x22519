extern crate lazy_static;
extern crate num_bigint;

pub mod hex {

    use num_bigint::BigInt;
    use num_traits::Num;

    // Extract the keys from the hex string
    pub fn decode(hexstr: &str) -> Result<BigInt, num_bigint::ParseBigIntError>{
        let r = BigInt::from_str_radix(hexstr, 16);
        r
    }

    pub fn decodeLittleEndian(b: &[u8]) -> BigInt {
        BigInt::from_bytes_le(num_bigint::Sign::Plus, &b[0..32])
    }

    pub fn decodeScalar25519(k: &str) -> BigInt{
        assert!(k.len() == 64);
        let mut k_list: [u8;32] = [0; 32];
        for i in 0..32 {
            k_list[i] = u8::from_str_radix(&k[2*i .. 2*(i+1)], 16).expect("Error parsing hexidecimal string");
        }
        
        k_list[0] &= 248;
        k_list[31] &= 127;
        k_list[31] |= 64;

        decodeLittleEndian(k_list.as_slice())
    }

    
    pub fn decodeUCoordinate(k: &str) -> BigInt{
        assert!(k.len() == 64);
        let mut u_list: [u8;32] = [0; 32];
        for i in 0..32 {
            u_list[i] = u8::from_str_radix(&k[2*i .. 2*(i+1)], 16).expect("Error parsing hexidecimal string");
        }

        decodeLittleEndian(u_list.as_slice())
    }

    #[cfg(test)]
    mod tests {

        use super::*;

        #[test]
        fn test_ladder() {
            let k = "a546e36bf0527c9d3b16154b82465edd62144c0ac1fc5a18506a2244ba449ac4".to_string();
            let u = "e6db6867583030db3594c1a424b15f7c726624ec26b3353b10a903a6d0ab1c4c".to_string();

            let k = decodeScalar25519(&k);
            let u = decodeUCoordinate(&u);
            dbg!(k, u);
        }
    }

}

pub mod elliptic {

    use lazy_static::lazy_static;

    use num_bigint::BigInt;

    lazy_static! {
        pub static ref P: BigInt = BigInt::from(2).pow(255) - BigInt::from(19);
        pub static ref A24: BigInt = BigInt::from(121666);
    }

    
    fn x_add(
        (x_p, z_p): (BigInt, BigInt),
        (x_q, z_q): (BigInt, BigInt),
        (x_m, z_m): (BigInt, BigInt),
        p: &BigInt,
    ) -> (BigInt, BigInt) {
        let u = (&x_p - &z_p) * (&x_q + &z_q) % p;
        let v = (&x_p + &z_p) * (&x_q - &z_q) % p;

        let upv2 = (&u + &v).pow(2);
        let umv2 = (&u - &v).pow(2);

        let x_p = (&z_m * upv2) % p;
        let z_p = (&x_m * umv2) % p;

        (x_p, z_p)
    }
    
    fn x_dbl((x, z): (BigInt, BigInt), p: &BigInt, a24: &BigInt) -> (BigInt, BigInt) {

        let q = (&x + &z) % p;
        let q = (q.pow(2)) % p;

        //let R = (X - Z) % p;
        let r = (&x*&x + &z*&z - BigInt::from(2)*&x*&z) % p;

        let s = (BigInt::from(4) * &x * &z) % p;

        let x_3 = (&q * &r) % p;
        let z_3 = (&s * (&r + (a24 * &s))) % p;

        (x_3, z_3)
    }


    
    fn conditional_swap(
        swap: u8,
        (x_1, z_1): (BigInt, BigInt),
        (x_2, z_2): (BigInt, BigInt),
    ) -> ((BigInt, BigInt), (BigInt, BigInt)) {
        let swap = BigInt::from(swap);
        let onemswap = BigInt::from(1) - &swap;
        (
            (&x_1 * &onemswap + &x_2 * &swap, &z_1 * &onemswap + &z_2 * &swap),
            (&x_1 * &swap + &x_2 * &onemswap, &z_1 * &swap + &z_2 * &onemswap),
        )
    }
    

    
    pub fn ladder(m: &BigInt, x: &BigInt, p: &BigInt, a24: &BigInt) -> (BigInt, BigInt) {
        let u = (x.clone(), BigInt::from(1));
        let mut x_0 = (BigInt::from(1), BigInt::from(0));
        let mut x_1 = u.clone();

        let mut bits: [u8; 256] = [0; 256];
        for i in 0..255 {
            bits[i] = m.bit(i as u64) as u8;
        } // Bits are read in one constant go.

        for i in (0..m.bits()).rev() {
            let bit = bits[i as usize];
            let x_added = x_add(x_0.clone(), x_1.clone(), u.clone(), p);
            let (m0, _) = conditional_swap(bit, x_0, x_1);
            let x_doubled = x_dbl(m0, &p, &a24);
            
            x_0 = x_doubled;
            x_1 = x_added;

            let (m0, m1) = conditional_swap(bit, x_0, x_1);

            x_0 = m0;
            x_1 = m1;
        }

        x_0
    }
    
    pub fn slightly_different_x22519(m: &BigInt, x: &BigInt, p: &BigInt, a24: &BigInt) -> BigInt {
        let u = (x.clone(), BigInt::from(1));
        let mut x_2 = (BigInt::from(1), BigInt::from(0));
        let mut x_3 = u.clone();

        let mut bits: [u8; 256] = [0; 256];
        for i in 0..255 {
            bits[i] = m.bit(i as u64) as u8;
        } // Bits are read in one constant go.
        let mut swap= 0;
        for i in (0..m.bits()).rev() {
            let bit = bits[i as usize];
            swap ^= bit;

            (x_2, x_3) = conditional_swap(swap, x_2, x_3);

            swap = bit;
            let xx_2 = &x_2.0;
            let xz_2 = &x_2.1;

            let xx_3 = &x_3.0;
            let xz_3 = &x_3.1;

            let A = xx_2 + xz_2;
            let AA = (&A).pow(2);

            let B = xx_2 - xz_2;
            let BB = (&B).pow(2);

            let E = &AA - &BB;
            let C = xx_3 + xz_3;
            let D = xx_3 - xz_3;
            let DA = D * &A;
            let CB = C * &B;

            let xx_3 = (&DA + &CB).pow(2) % p;
            let xz_3 = x * (&DA - &CB ).pow(2) % p;
            let xx_2 = (&AA * BB) % p;
            let xz_2 = &E * (AA + a24 * &E) % p;

            x_2 = (xx_2, xz_2);
            x_3 = (xx_3, xz_3);

        }
        (x_2, _) = conditional_swap(swap, x_2, x_3);

        x_2.0 * (x_2.1.modpow(&(p-2), p)) % p
    }


    #[cfg(test)]
    mod tests {

        use super::*;

        #[test]
        fn test_ladder() {
            let p = BigInt::from(101);
            let a24 = BigInt::from(38);


            let (x_1, z_1) = x_dbl((BigInt::from(2),BigInt::from(1)), &p, &a24);
            let x_0 = (x_1.clone() * BigInt::modpow(&z_1, &(&p-BigInt::from(2)), &p)) % &p;
            assert_eq!(x_0, BigInt::from(70), "2[P]");


            let (x_1, z_1) = ladder(&BigInt::from(3), &BigInt::from(2), &p, &a24);
            let x_0 = (x_1.clone() * BigInt::modpow(&z_1, &(&p-BigInt::from(2)), &p)) % &p;
            assert_eq!(x_0, BigInt::from(59), "3[P]");


            let (x_1, z_1) = ladder(&BigInt::from(77), &BigInt::from(2), &p, &a24);
            let x_0 = (x_1.clone() * BigInt::modpow(&z_1, &(&p-BigInt::from(2)), &p)) % &p;
            assert_eq!(x_0, BigInt::from(8), "77[P]");


            // Tests with p = 1009 and A=682
            let p = BigInt::from(1009);
            let a24 = BigInt::from(171);

            let x_p = BigInt::from(7);

            let ms = [2, 3, 5, 34, 104, 947];
            let asserts = [284, 759, 1000, 286, 810, 755].map(|v| BigInt::from(v));
            for i in 0..ms.len() {
                let (x_1, z_1) = ladder( &BigInt::from(ms[i]), &x_p, &p, &a24);
                let x_0 = (x_1.clone() * BigInt::modpow(&z_1, &(&p-BigInt::from(2)), &p)) % &p;
                assert_eq!(x_0, asserts[i], "{i}[P]");
    
            }

            
            // Tests for Curve25519

            let p = P.clone();
            let a24 = A24.clone();

            let x_p = BigInt::from(9);

            let (x_1, z_1) = ladder( &BigInt::from(7), &x_p, &p, &a24);
            let x_0 = (x_1.clone() * BigInt::modpow(&z_1, &(&p-BigInt::from(2)), &p)) % &p;
            dbg!(x_0.to_str_radix(10));


        }
    }
}
