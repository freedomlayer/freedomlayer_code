use std::collections::HashSet;
use rand;
use rand::Rng;
use rand::distributions::{IndependentSample, Range};

pub struct Network {
    n: usize,
    neighbours: Vec<HashSet<usize>>,
    landmarks: Vec<usize>,
    coords: Vec<Vec<Option<usize>>>,
}


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

impl Network {
    pub fn new() -> Self {
        Network {
            n: 0, 
            neighbours: Vec::new(),
            landmarks: Vec::new(),
            coords: Vec::new(),
        }
    }

    pub fn build_network<R: Rng>(
        mut self, n: usize, num_neighbours: usize, rng: &mut R) -> Self {
        self.neighbours.clear();
        self.n = n;
        for _ in 0 .. n {
            self.neighbours.push(HashSet::new());
        }

        let rand_range: Range<usize> = Range::new(0,self.n);

        // Connect node v to about num_neighbours other nodes:
        for v in 0 .. self.n {
            for _ in 0 .. num_neighbours {
                let u = rand_range.ind_sample(rng);
                if u == v {
                    // Avoid self loops
                    continue
                }
                if self.neighbours[v].contains(&u) {
                    // Already has this edge.
                    continue
                }
                // Add edge:
                self.neighbours[v].insert(u);
                self.neighbours[u].insert(v);
            }
        }
        self
    }

    pub fn choose_landmarks<R: Rng> (mut self,num_landmarks: usize, rng: &mut R) 
        -> Self {

        self.landmarks = choose_k_nums(num_landmarks, self.n, rng)
            .into_iter().collect();

        // Initialize coordinates:
        for v in 0..self.n {
            let mut v_coords = Vec::new();
            for &l in self.landmarks.iter() {
                if v != l {
                    v_coords.push(None)
                } else {
                    v_coords.push(Some(0))
                }
            }
            self.coords.push(v_coords);
        }
        self
    }

    /// Every node asks neighbours about distance to landmarks and 
    /// updates his own distances accordingly.
    pub fn iter_coords(&mut self) {
        for v in 0..self.n {
            for &nei in self.neighbours[v].iter() {
                for c in 0..self.coords[nei].len() {
                    let dist = self.coords[nei][c];
                    if dist.is_none() {
                        continue
                    }
                    let cdist = dist.unwrap();
                    if self.coords[v][c].is_none() {
                        self.coords[v][c] = Some(cdist);
                        continue
                    }
                    if self.coords[v][c].unwrap() > cdist {
                        self.coords[v][c] = Some(cdist);
                    }
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(unused_variables)]
    fn test_basic() {
        let mut rng = rand::thread_rng();
        let mut net = Network::new()
            .build_network(40,3,&mut rng)
            .choose_landmarks(4,&mut rng);

        net.iter_coords();
        net.iter_coords();
    }

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
