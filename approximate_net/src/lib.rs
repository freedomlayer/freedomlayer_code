extern crate rand;
extern crate bincode;
extern crate ring;

use self::rand::{Rng};
use self::ring::{digest};
use bincode::{serialize, Infinite};

/// Generate random u64 elements:
pub fn gen_elems<R: Rng>(num_elems: usize, rng: &mut R) -> Vec<u64> {
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


pub fn calc_mins(elems: &Vec<u64>, num_hashes: usize) -> Vec<u64> {
    (0 .. num_hashes)
        .map(|hash_index| elems.iter()
             .map(|&x| hash_elem(hash_index, x))
             .min().unwrap())
        .collect::<Vec<u64>>()
}

/// Calculate harmonic mean of given values
fn harmonic_mean(vals: &[f64]) -> f64 {
    let fsum: f64 = vals.iter()
        .map(|&x| 1.0 / x)
        .sum();

    (vals.len() as f64) / fsum
}

pub fn approx_size_harmonic(mins: &Vec<u64>) -> usize {
    let trans = mins.iter()
        .map(|&m| (u64::max_value() / m) - 1)
        .map(|x| x as f64)
        .collect::<Vec<f64>>();

    harmonic_mean(&trans) as usize
}


#[cfg(test)]
mod tests {
    use super::*;
    use self::rand::{StdRng};
 
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
    fn test_approx_size_harmonic() {
        let seed: &[_] = &[1,2,3,4];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        let elems = gen_elems(20, &mut rng);
        assert_eq!(elems.len(), 20);
        let mins = calc_mins(&elems, 4);
        assert_eq!(mins.len(), 4);
        approx_size_harmonic(&mins);
    }

}
