extern crate rand;
use self::rand::Rng;
use self::rand::distributions::{IndependentSample, Range};

/// Randomly choose k distinct numbers from the range [0,n) 
fn choose_k_nums<R: Rng>(k:usize, n:usize, rng: &mut R) -> HashSet<usize> {
    
    let mut res_set = HashSet::new();
    let rand_range: Range<usize> = Range::new(0,n);
    for _ in 0..k {
        let mut x = rand_range.ind_sample(rng);
        while res_set.contains(&x) {
            x = rand_range.ind_sample(rng);
        }
        res_set.insert(x);
    }
    res_set
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_choose_k_nums() {
        let mut rng = rand::thread_rng();
        let knums = choose_k_nums(3,100,&mut rng);
        assert!(knums.len() == 3);
        for x in knums.into_iter() {
            assert!(x < 100);
        }
    }
}
