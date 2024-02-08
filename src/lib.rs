extern crate lazy_static;
extern crate num_bigint;

pub mod hex {
    use num_bigint::BigUint;

    // Extract the keys from the hex string
    pub fn decode(hexstr: &[u8]) -> Option<BigUint> {
        let r = BigUint::parse_bytes(hexstr, 16);
        r
    }
}

pub mod elliptic {

    use lazy_static::lazy_static;

    use num_bigint::BigInt;

    lazy_static! {
        static ref P: BigInt = BigInt::from(2).pow(255) - BigInt::from(19);
        static ref A24: BigInt = BigInt::from(486662);
    }

    
    fn x_add(
        (x_P, z_P): (BigInt, BigInt),
        (x_Q, z_Q): (BigInt, BigInt),
        (x_m, z_m): (BigInt, BigInt),
        p: &BigInt,
    ) -> (BigInt, BigInt) {
        let u = (&x_P - &z_P) * (&x_Q + &z_Q) % p;
        let v = (&x_P + &z_P) * (&x_Q - &z_Q) % p;

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

        dbg!(m.bits());
        for i in (0..m.bits()).rev() {
            let bit = bits[i as usize];
            let x_added = x_add(x_0.clone(), x_1.clone(), u.clone(), p);
            let (m0, m1) = conditional_swap(bit, x_0, x_1);
            let x_doubled = x_dbl(m0, &p, &a24);
            
            x_0 = x_doubled;
            x_1 = x_added;

            let (m0, m1) = conditional_swap(bit, x_0, x_1);

            x_0 = m0;
            x_1 = m1;
        }

        x_0
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

        }
    }
}
