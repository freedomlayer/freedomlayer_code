extern crate petgraph;
extern crate rand;

use self::petgraph;
use self::petgraph::Graph;

use self::rand::Rng;
use self::rand::distributions::{IndependentSample, Range};

type Network<Node> = GraphMap<Node,f64,petgraph::Undirected>;


pub fn random_net<R: Rng>(n: usize, num_neighbours: usize, rng: &mut R) -> Network<usize> {
    let mut net: Network<usize>::new();

    // Insert n nodes:
    for v in 0 .. n {
        net.add_node(v);
    }

    let rand_node: Range<usize> = Range::new(0,n);
    // Generate random distance between pairs of nodes:
    let rand_dist: Range<f64> = Range::new(1.0,10.0);

    // Connect node v to about num_neighbours other nodes:
    for v in 0 .. n {
        for _ in 0 .. num_neighbours {
            let u = rand_node.ind_sample(rng);
            if u == v {
                // Avoid self loops
                continue
            }
            if net.contains_edge(v,u) {
                // Already has this edge.
                continue
            }
            // Add edge:
            net.add_edge(v,u,rand_dist.ind_sample(rng));
        }
    }
    net
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_net() {
        let net = random_net(100,5,&mut rng);
        assert!(net.node_count() == 100);
    }
}
