extern crate petgraph;
extern crate rand;

use std::collections::{HashMap, HashSet};

use self::rand::{Rng, StdRng};
use self::rand::distributions::{IndependentSample, Range};
use self::petgraph::graphmap::NodeTrait;
use self::petgraph::algo::dijkstra;
use self::petgraph::visit::EdgeRef;

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

    pub fn add_node(&mut self, node: Node) -> usize {
        let node_num = self.index_nodes.len();
        self.index_nodes.push(node);
        self.igraph.add_node(node_num);

        // Return the index of the new node:
        node_num
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

    pub fn dist(&self, a_index: usize, b_index: usize) -> Option<u64> {
        let scores = dijkstra(&self.igraph, 
                 a_index,
                 Some(b_index),
                 |e| *e.weight());

        scores.get(&b_index).map(|x| *x)
    }

    /// Get all nodes of distance <= dist from a given node
    /// Returns a set of all those nodes
    pub fn get_near_nodes(&self, index: usize, dist: usize) -> HashSet<usize> {
        let res_set = HashSet::new();
        res_set.insert(index);

        for node_index in res_set.clone() {
            for nei_index in self.igraph.neighbors(node_index) {
                res_set.insert(nei_index)
            }
        }
        return res_set
    }
}


pub fn random_net<R: Rng>(n: usize, num_neighbours: usize, rng: &mut R) -> Network<usize> {

    let mut net = Network::<usize>::new();

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

    #[test]
    fn test_net_dist() {
        let mut net = Network::<usize>::new();

        // Insert n nodes:
        for v in 0 .. 5 {
            net.add_node(v);
        }

        net.igraph.add_edge(0,1,1);
        net.igraph.add_edge(1,2,2);
        net.igraph.add_edge(2,4,3);

        assert!(net.dist(0,4).unwrap() == 6);
        assert!(net.dist(1,4).unwrap() == 5);
        assert!(net.dist(1,3).is_none());
    }
}
