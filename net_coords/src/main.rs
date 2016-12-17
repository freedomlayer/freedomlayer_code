extern crate rand;
use std::collections::HashSet;

use rand::Rng;
use rand::distributions::Range;

struct Network {
    n: u32,
    neighbours: Vec<HashSet<u32>>,
}


impl Network {
    fn new(n: u32, num_neighbours: u32) -> Self {
        let net: Self = Network {n: n, neighbours: Vec::new()};

        for _ in 0..n {
            self.neighbours.push(HashSet::new());
        }

        net
    }

    fn build_network(&mut self, rng: &mut Rng) {
        let range = Range::new(0,self.n);
        // Connect node v to about num_neighbours other nodes:
        for v in 0..n {
            for _ in 0..self.num_neighbours {
                let u = range.ind_sample(rng);
                if u == v {
                    // Avoid self loops
                    continue
                }
                if self.neighbours[v].contains(u) {
                    // Already has this edge.
                    continue
                }
                // Add edge:
                self.neighbours[v].insert(u);
                self.neighbours[u].insert(v)
            }
        }
        self
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_build_network() {
        let mut rng = rand::thread_rng();
        let net = Network::new().
            build_network(40,3,&mut rng);
    }
}


#[cfg(not(test))]
fn main() {
    let net = Network::new();
    let mut rng = rand::thread_rng();

    println!("Hello, world!");
}
