pub mod approx_funcs;

extern crate rand;
extern crate bincode;
extern crate ring;

use self::rand::{Rng};
use self::ring::{digest};
use bincode::{serialize, Infinite};
use std::ops;

use approx_funcs::{ApproxFunc};


/// Generate random u64 elements:
fn gen_elems<R: Rng>(num_elems: usize, rng: &mut R) -> Vec<u64> {
    (0 .. num_elems)
        .map(|_| rng.gen::<u64>())
        .collect::<Vec<u64>>()
}

/// Hash a given u64 value using function number hash_index.
fn hash_elem(hash_index: usize, x: u64) -> u64 {
    // Serialize the hash_index and x:
    let enc_hash_index: Vec<u8> = serialize(&hash_index, Infinite).unwrap();
    let enc_x: Vec<u8> = serialize(&x, Infinite).unwrap();

    // Put everything into sha256:
    let mut ctx = digest::Context::new(&digest::SHA256);
    ctx.update(b"hash_func");
    ctx.update(&enc_hash_index);
    ctx.update(&enc_x);
    let digest = ctx.finish();
    let hash_output = digest.as_ref();

    let mut res: u64 = 0;

    // Read some of the output as a u64 number:
    for &h in hash_output {
        res <<= 8;
        res |= h as u64;
    }
    res
}


fn calc_mins(elems: &Vec<u64>, num_hashes: usize) -> Vec<u64> {
    (0 .. num_hashes)
        .map(|hash_index| elems.iter()
             .map(|&x| hash_elem(hash_index, x))
             .min().unwrap())
        .collect::<Vec<u64>>()
}

fn square_dist<T: Copy + ops::Sub<Output=T> 
                        + ops::Mul<Output=T> 
                        + PartialOrd 
                        + Ord>(x:T, y:T) -> T {
    if x > y {
        (x - y) * (x - y)
    } else {
        (y - x) * (y - x)
    }
}


/// Evaluate approx size method.
/// This is done by running approximation
pub fn eval_approx_size_funcs<R: Rng>(num_iters: usize, 
                           num_mins: usize, 
                           num_elems: usize, 
                           approx_size_funcs: &[&ApproxFunc],
                           rng: &mut R) -> Vec<f64> {

    let mut total_serrors = vec![0; approx_size_funcs.len()];
    for _ in 0 .. num_iters {
        let elems = gen_elems(num_elems, rng);
        let mins = calc_mins(&elems, num_mins);
        for (i, &approx_size_func) in approx_size_funcs.iter().enumerate() {
            let approx_size = approx_size_func(&mins);
            total_serrors[i] += square_dist(approx_size, elems.len());
        }

    }

    total_serrors.into_iter()
        .map(|te| te / num_iters)
        .map(|variance| (variance as f64).sqrt() / (num_elems as f64))
        .collect::<Vec<f64>>()
}


#[cfg(test)]
mod tests {
    use super::*;
    use self::rand::{StdRng};
    use self::approx_funcs::approx_size_harmonic_before;
 
    #[test]
    fn test_hash_elem() {
        let x = hash_elem(5,3);
        let y = hash_elem(5,3);
        // Consistency:
        assert_eq!(x, y);

        let z = hash_elem(5,4);
        assert_ne!(x, z);
    }

    #[test]
    fn test_calc_mins() {
        let seed: &[_] = &[1,2,3,4];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        let elems = gen_elems(20, &mut rng);
        assert_eq!(elems.len(), 20);
        let mins = calc_mins(&elems, 4);
        assert_eq!(mins.len(), 4);
    }

    #[test]
    fn test_eval_approx_size_funcs() {
        let seed: &[_] = &[1,2,3,4];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);

        let num_iters = 5;
        let num_mins = 10;
        let num_elems = 50;

        let err_ratios = eval_approx_size_funcs(num_iters, 
                                                num_mins, 
                                                num_elems, 
                                                &[&approx_size_harmonic_before], 
                                                &mut rng);
        assert_eq!(err_ratios.len(),1);
    }

}
