extern crate rand;

mod network_sim {
    use std::collections::HashSet;
    use rand::Rng;
    use rand::distributions::{IndependentSample, Range};


    pub struct Network {
        n: u32,
        num_neighbours: u32,
        neighbours: Vec<HashSet<u32>>,
    }


    impl Network {
        pub fn new(n: u32, num_neighbours: u32) -> Self {
            let mut net: Self = Network {
                n: n, 
                num_neighbours: num_neighbours,
                neighbours: Vec::new()
            };

            for _ in 0 .. n {
                net.neighbours.push(HashSet::new());
            }

            net
        }

        pub fn build_network<R: Rng>(&mut self, rng: &mut R) -> &mut Self {
            let rand_range: Range<u32> = Range::new(0,self.n);
            // Connect node v to about num_neighbours other nodes:
            for v in 0 .. self.n {
                for _ in 0 .. self.num_neighbours {
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
    }
}

#[cfg(test)]
mod test {
    extern crate rand;
    use rand::Rng;
    use rand::distributions::{IndependentSample, Range};
    use network_sim::Network;

    #[test]
    fn test_build_network() {
        let mut rng = rand::thread_rng();
        let net = Network::new(40,3).
            build_network(&mut rng);
    }
}


#[cfg(not(test))]
fn main() {
    // let net = Network::new();
    // let mut rng = rand::thread_rng();

    println!("Hello, world!");
}
