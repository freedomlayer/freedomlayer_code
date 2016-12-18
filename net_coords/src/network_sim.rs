use std::collections::HashSet;
use rand;
use rand::Rng;
use rand::distributions::{IndependentSample, Range};

pub struct Network {
    n: u64,
    neighbours: Vec<HashSet<u64>>,
    landmarks: Vec<u64>,
}


fn choose_k_nums<R: Rng>(k:u64, n:u64, rng: &mut R) -> HashSet<u64> {
    /// Randomly choose k distinct numbers from the range [0,n) 
    
    let mut res_set = HashSet::new();
    let rand_range: Range<u64> = Range::new(0,n);
    for i in 0 .. k {
        let mut x = rand_range.ind_sample(rng);
        while res_set.contains(&x) {
            x = rand_range.ind_sample(rng);
        }
        res_set.insert(x);
    }
    res_set
}

impl Network {
    pub fn new() -> Self {
        let mut net: Self = Network {
            n: 0, 
            neighbours: Vec::new(),
            landmarks: Vec::new(),
        };
        net
    }

    pub fn build_network<R: Rng>(
        mut self, n: u64, num_neighbours: u64, rng: &mut R) -> Self {
        self.neighbours.clear();
        self.n = n;
        for _ in 0 .. n {
            self.neighbours.push(HashSet::new());
        }

        let rand_range: Range<u64> = Range::new(0,self.n);

        // Connect node v to about num_neighbours other nodes:
        for v in 0 .. self.n {
            for _ in 0 .. num_neighbours {
                let u = rand_range.ind_sample(rng);
                if u == v {
                    // Avoid self loops
                    continue
                }
                if self.neighbours[v as usize].contains(&u) {
                    // Already has this edge.
                    continue
                }
                // Add edge:
                self.neighbours[v as usize].insert(u);
                self.neighbours[u as usize].insert(v);
            }
        }
        self
    }

    pub fn choose_landmarks<R: Rng> (num_landmarks: u64, rng: &mut R) {
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(unused_variables)]
    fn test_build_network() {
        let mut rng = rand::thread_rng();
        let net = Network::new()
            .build_network(40,3,&mut rng);
    }

    #[test]
    fn test_choose_k_nums() {
        let mut rng = rand::thread_rng();
        let knums = choose_k_nums(3,100,&mut rng);
        assert!(knums.len() == 3);
    }
}
