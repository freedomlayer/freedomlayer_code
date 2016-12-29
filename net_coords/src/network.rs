extern crate petgraph;
extern crate rand;

use std::collections::HashMap;

use self::rand::{Rng, StdRng};
use self::rand::distributions::{IndependentSample, Range};
use self::petgraph::graphmap::NodeTrait;

pub struct Network<Node> {
    pub igraph: petgraph::graphmap::GraphMap<usize,u64,petgraph::Undirected>,
    nodes_index: HashMap<Node, usize>, // Node -> Index
    index_nodes: Vec<Node>, // Index -> Node
}


impl <Node: NodeTrait> Network <Node> {
    pub fn new() -> Self {
        Network::<Node> {
            igraph: petgraph::graphmap::GraphMap::new(),
            nodes_index: HashMap::new(),
            index_nodes: Vec::new()
        }
    }

    pub fn add_node(&mut self, node: Node) {
        let node_num = self.index_nodes.len();
        self.index_nodes.push(node);
        self.igraph.add_node(node_num);
    }

    /*
    pub fn add_edge(&mut self, a: Node, b: Node,weight: u64) {
        let a_index = self.nodes_index.get(&a).unwrap();
        let b_index = self.nodes_index.get(&b).unwrap();
        self.igraph.add_edge(a_index,b_index,weight)
    }
    */

    pub fn index_to_node<'a>(&'a self, index: usize) -> Option<&'a Node> {
        if index > self.index_nodes.len() {
            return None
        }
        Some(&self.index_nodes[index])
    }

    pub fn node_to_index(&self, node: &Node) -> Option<usize> {
        self.nodes_index.get(node).map(|&num| num)
    }
}


pub fn random_net<R: Rng>(n: usize, num_neighbours: usize, rng: &mut R) -> Network<usize> {

    let mut net =  Network::<usize>::new();

    // Insert n nodes:
    for v in 0 .. n {
        net.add_node(v);
    }

    let rand_node: Range<usize> = Range::new(0,n);
    // Generate random distance between pairs of nodes:
    let rand_dist: Range<u64> = Range::new(1,10);

    // Connect node v to about num_neighbours other nodes:
    for v in 0 .. n {
        for _ in 0 .. num_neighbours {
            let u = rand_node.ind_sample(rng);
            if u == v {
                // Avoid self loops
                continue
            }
            if net.igraph.contains_edge(v,u) {
                // Already has this edge.
                continue
            }
            // Add edge:
            net.igraph.add_edge(v,u,rand_dist.ind_sample(rng));
        }
    }
    net
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_net() {
        let seed: &[_] = &[1,2,3,4];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        let net = random_net(100,5,&mut rng);
        assert!(net.igraph.node_count() == 100);
    }
}
