extern crate petgraph;
extern crate rand;

use std::collections::{HashMap, HashSet};
use std::hash::{Hash};

use self::rand::{Rng};
use self::rand::distributions::{IndependentSample, Range};
use self::petgraph::algo::{dijkstra, connected_components};
use self::petgraph::visit::{EdgeRef};

pub struct Network<Node> {
    pub igraph: petgraph::graphmap::GraphMap<usize,u64,petgraph::Undirected>,
    nodes_index: HashMap<Node, usize>, // Node -> Index
    index_nodes: Vec<Node>, // Index -> Node
}

pub struct ClosestNodes<'a, Node: 'a> {
    net: &'a Network<Node>,
    pending: HashMap<usize, (u64, Option<usize>)>,
    done: HashSet<usize>,
}


/// An iterator for closest nodes to a given
/// node in a graph.
impl<'a, Node> Iterator for ClosestNodes<'a, Node> {
    // node_index, distance from source, gateway node index
    type Item = (usize, u64, usize); 

    fn next(&mut self) -> Option<(usize,u64, usize)> {
        let (node_index, node_dist, gateway_index) : (usize, u64, Option<usize>) = { 
            let min_elem = self.pending.iter()
                .min_by_key(|&(index, &(dist, _))| (dist,index));

            let (&node_index, &(node_dist, gateway_index)) = match min_elem {
                None => return None,
                Some(x) => x,
            };

            (node_index, node_dist, gateway_index)
        };

        self.pending.remove(&node_index);

        for (_, nei_index, weight) in self.net.igraph.edges(node_index) {

            let nei_gateway = match gateway_index {
                Some(index) => index,
                None => nei_index,
            };

            let new_dist = node_dist + weight;
            if self.done.contains(&nei_index) {
                continue;
            }
            if !self.pending.contains_key(&nei_index) {
                self.pending.insert(nei_index, (new_dist, Some(nei_gateway)));
                continue;
            }
            if self.pending[&nei_index].0 > new_dist {
                self.pending.insert(nei_index, (new_dist, Some(nei_gateway)));
                continue;
            }
        }

        self.done.insert(node_index);

        match gateway_index {
            Some(gindex) => Some((node_index, node_dist, gindex)),
            None => self.next(),
        }
    }
}

pub struct ClosestNodesStructure<'a, Node: 'a> {
    net: &'a Network<Node>,
    pending: HashMap<usize, (u64, Option<usize>)>,
    done: HashSet<usize>,
}

/// An iterator for closest nodes to a given
/// node in a graph.
impl<'a, Node> Iterator for ClosestNodesStructure<'a, Node> {
    // node_index, distance from source, gateway node index
    type Item = (usize, u64, usize); 

    fn next(&mut self) -> Option<(usize,u64, usize)> {
        let (node_index, node_dist, gateway_index) : (usize, u64, Option<usize>) = { 
            let min_elem = self.pending.iter()
                .min_by_key(|&(index, &(dist, _))| (dist,index));

            let (&node_index, &(node_dist, gateway_index)) = match min_elem {
                None => return None,
                Some(x) => x,
            };

            (node_index, node_dist, gateway_index)
        };

        self.pending.remove(&node_index);

        for (_, nei_index, weight) in self.net.igraph.edges(node_index) {

            let nei_gateway = match gateway_index {
                Some(index) => index,
                None => nei_index,
            };

            // Always assuming weight = 1
            // Possibly change this later.
            let _ = weight;
            let new_dist = node_dist + 1;
            if self.done.contains(&nei_index) {
                continue;
            }
            if !self.pending.contains_key(&nei_index) {
                self.pending.insert(nei_index, (new_dist, Some(nei_gateway)));
                continue;
            }
            if self.pending[&nei_index].0 > new_dist {
                self.pending.insert(nei_index, (new_dist, Some(nei_gateway)));
                continue;
            }
        }

        self.done.insert(node_index);

        match gateway_index {
            Some(gindex) => Some((node_index, node_dist, gindex)),
            None => self.next(),
        }
    }
}


impl <Node: Hash + Eq + Clone> Network <Node> {
    pub fn new() -> Self {
        Network::<Node> {
            igraph: petgraph::graphmap::GraphMap::new(),
            nodes_index: HashMap::new(),
            index_nodes: Vec::new()
        }
    }

    pub fn add_node(&mut self, node: Node) -> usize {
        assert!(!self.nodes_index.contains_key(&node), "We already have this node! Aborting.");
        let node_num = self.index_nodes.len();
        self.nodes_index.insert(node.clone(), node_num);
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

    /// Get an Iterator for the closest nodes to node <index>
    pub fn closest_nodes<'a>(&'a self, index: usize) -> ClosestNodes<'a, Node> {
        ClosestNodes {
            net: &self,
            pending: [(index, (0, None)),].iter().cloned().collect(),
            done: HashSet::new(),
        }
    }

    /// Get an Iterator for the closest nodes to node <index>.
    /// Ignore weights. Assume all edges are of length 1.
    pub fn closest_nodes_structure<'a>(&'a self, index: usize) -> ClosestNodesStructure<'a, Node> {
        ClosestNodesStructure {
            net: &self,
            pending: [(index, (0, None)),].iter().cloned().collect(),
            done: HashSet::new(),
        }
    }

    /// Check if the network is connected (As an undirected graph).
    pub fn is_connected(&self) -> bool {
        connected_components(&self.igraph) <= 1
    }
}
/// Create a 2d grid network k X k
pub fn grid2_net(k: usize) -> Network<usize> {

    let mut net = Network::<usize>::new();

    // Insert n nodes:
    for v in 0 .. k*k {
        net.add_node(v);
    }

    // Add edges:
    for x in 0 .. k-1 {
        for y in 0 .. k {
            net.igraph.add_edge(y + x*k, y + (x+1)*k, 1);
        }
    }

    for x in 0 .. k {
        for y in 0 .. k-1 {
            net.igraph.add_edge(y + x*k, (y+1) + x*k, 1);
        }
    }

    net
}


/// A random network where all edges are of weight 1.
pub fn random_net<R: Rng>(n: usize, num_neighbours: usize, rng: &mut R) -> Network<usize> {

    let mut net = Network::<usize>::new();

    // Insert n nodes:
    for v in 0 .. n {
        net.add_node(v);
    }

    let rand_node: Range<usize> = Range::new(0,n);
    // Generate random distance between pairs of nodes:
    // let rand_dist: Range<u64> = Range::new(1,10);

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
            net.igraph.add_edge(v,u,1);
        }
    }
    net
}

/// A random network where edges are weighted.
pub fn random_net_weighted<R: Rng>(n: usize, num_neighbours: usize, rng: &mut R) -> Network<usize> {

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
    use self::rand::{StdRng};

    #[test]
    fn test_random_net() {
        let seed: &[_] = &[1,2,3,4];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        let net = random_net(100,5,&mut rng);
        assert!(net.igraph.node_count() == 100);
    }

    #[test]
    fn test_random_net_weighted() {
        let seed: &[_] = &[1,2,3,4];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        let net = random_net_weighted(100,5,&mut rng);
        assert!(net.igraph.node_count() == 100);
    }

    #[test]
    fn test_grid2_net() {
        let net = grid2_net(100);
        assert!(net.igraph.node_count() == 100*100);
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

    #[test]
    fn test_net_is_connected() {
        let mut net = Network::<usize>::new();

        // Insert n nodes:
        for v in 0 .. 5 {
            net.add_node(v);
        }

        net.igraph.add_edge(0,1,1);
        net.igraph.add_edge(1,2,2);
        net.igraph.add_edge(2,4,3);
        assert!(!net.is_connected());

        net.igraph.add_edge(2,3,1);
        assert!(net.is_connected());
    }

    #[test]
    fn test_closest_nodes() {
        let mut net = Network::<usize>::new();

        // Insert n nodes:
        for v in 0 .. 7 {
            net.add_node(v);
        }

        net.igraph.add_edge(0,1,1);
        net.igraph.add_edge(1,2,2);
        net.igraph.add_edge(2,4,3);
        net.igraph.add_edge(4,6,2);


        let closest: Vec<_> = net.closest_nodes(1).take(5).collect();
        assert!(closest.len() == 4);
        assert!(closest[0] == (0,1,0));
        assert!(closest[1] == (2,2,2));
        assert!(closest[2] == (4,5,2));
        assert!(closest[3] == (6,7,2));
    }

    #[test]
    fn test_lexicographic() {
        let a = (1,2);
        let b = (1,3);
        assert!(a < b);

        let a = (1,1,9);
        let b = (1,1,10);
        assert!(a < b);

        let a = (2,3,9);
        let b = (2,3,8);
        assert!(a > b);
    }
}
