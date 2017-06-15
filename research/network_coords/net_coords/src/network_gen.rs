extern crate rand;
use self::rand::{Rng};

use network::{Network};
use chord::{RingKey};
use smallest_k::{SmallestK};
use std::collections::{HashSet, HashMap};
use self::rand::distributions::{IndependentSample, Range};


/// Generate a random graph to be used with chord.
/// Graph nodes are of type RingKey.
pub fn random_net_chord<R: Rng>(num_nodes: usize, num_neighbors: usize, l: usize, rng: &mut R) 
        -> Network<RingKey> {

    // Maximum key in the ring:
    let max_key = 2_u64.pow(l as u32);


    // We can't have too many nodes with respect to the keyspace.
    // We stay below sqrt(keyspace_size), to avoid collisions.
    assert!(num_nodes < (max_key as f64).sqrt() as usize, "Too many nodes!");
    assert!(num_nodes > 0, "We should have at least one node!");

    let mut net = Network::<RingKey>::new();

    // A hash set to make sure we don't have duplicate keys.
    let mut chosen_keys: HashSet<RingKey> = HashSet::new();

    // Insert num_nodes nodes with random keys:
    for _ in 0 .. num_nodes {
        let rand_key: Range<RingKey> = Range::new(0,max_key);
        let mut node_key = rand_key.ind_sample(rng);
        while chosen_keys.contains(&node_key) {
            node_key = rand_key.ind_sample(rng);
        }
        chosen_keys.insert(node_key.clone());
        net.add_node(node_key);
    }

    for v in 0 .. num_nodes {
        for _ in 0 .. num_neighbors {
            let rand_node: Range<usize> = Range::new(0,num_nodes);
            let u = rand_node.ind_sample(rng);
            if u == v  {
                // Avoid self loops
                continue
            }
            if net.igraph.contains_edge(v,u) {
                // Already has this edge.
                continue
            }
            // Add edge:
            net.igraph.add_edge(v,u,1);
            // println!("add_edge {}, {}",v,u);
        }
    }
    net
}

/// Generate a weighted random graph to be used with chord.
/// Graph nodes are of type RingKey.
/// Edges lengths are uniform in [min_edge_len, max_edge_len)
pub fn random_weighted_net_chord<R: Rng>(num_nodes: usize, num_neighbors: usize, 
        min_edge_len: u64, max_edge_len: u64, l: usize, rng: &mut R) 
        -> Network<RingKey> {

    // Maximum key in the ring:
    let max_key = 2_u64.pow(l as u32);


    // We can't have too many nodes with respect to the keyspace.
    // We stay below sqrt(keyspace_size), to avoid collisions.
    assert!(num_nodes < (max_key as f64).sqrt() as usize, "Too many nodes!");
    assert!(num_nodes > 0, "We should have at least one node!");

    let mut net = Network::<RingKey>::new();

    // A hash set to make sure we don't have duplicate keys.
    let mut chosen_keys: HashSet<RingKey> = HashSet::new();

    // Insert num_nodes nodes with random keys:
    for _ in 0 .. num_nodes {
        let rand_key: Range<RingKey> = Range::new(0,max_key);
        let mut node_key = rand_key.ind_sample(rng);
        while chosen_keys.contains(&node_key) {
            node_key = rand_key.ind_sample(rng);
        }
        chosen_keys.insert(node_key.clone());
        net.add_node(node_key);
    }

    let edge_length_range: Range<u64> = Range::new(min_edge_len,max_edge_len);
    for v in 0 .. num_nodes {
        for _ in 0 .. num_neighbors {
            let rand_node: Range<usize> = Range::new(0,num_nodes);
            let u = rand_node.ind_sample(rng);
            if u == v  {
                // Avoid self loops
                continue
            }
            if net.igraph.contains_edge(v,u) {
                // Already has this edge.
                continue
            }
            // Add edge:
            net.igraph.add_edge(v,u,edge_length_range.ind_sample(rng));
            // println!("add_edge {}, {}",v,u);
        }
    }
    net
}

/// Generate a two dimensional grid k X k network where nodes have random keys from the keyspace.
/// n -- approximation of amount of nodes.
pub fn random_weighted_net_grid2_chord<R: Rng>(min_edge_len: u64, max_edge_len: u64, 
                                               k: usize, l:usize, rng: &mut R) -> Network<RingKey> {
    let mut net = Network::<RingKey>::new();
    let mut coord_to_index: HashMap<(usize, usize),usize>  = HashMap::new();
    // let mut key_to_coord: HashMap<RingKey, (usize, usize)>  = HashMap::new();

    // Maximum key in the ring:
    let max_key = 2_u64.pow(l as u32);

    // Network is k X k:
    //
    // Insert n nodes:
    //
    // Insert num_nodes nodes with random keys:
    //
    // A hash set to make sure we don't have duplicate keys.
    let mut chosen_keys: HashSet<RingKey> = HashSet::new();

    // Add all grid coordinates, matches with random ring keys:
    for x in 0 .. k {
        for y in 0 .. k {
            let rand_key: Range<RingKey> = Range::new(0,max_key);
            let mut node_key = rand_key.ind_sample(rng);
            while chosen_keys.contains(&node_key) {
                node_key = rand_key.ind_sample(rng);
            }
            chosen_keys.insert(node_key.clone());
            let node_index = net.add_node(node_key);

            // Add coord entry to map:
            coord_to_index.insert((x,y), node_index);

        }
    }

    let edge_length_range: Range<u64> = Range::new(min_edge_len,max_edge_len);

    // Add all grid edges:
    for x in 0 .. k-1 {
        for y in 0 .. k {
            let &a_i = coord_to_index.get(&(x,y)).unwrap();
            let &b_i = coord_to_index.get(&(x+1,y)).unwrap();
            net.igraph.add_edge(a_i, b_i, edge_length_range.ind_sample(rng));
        }
    }

    for x in 0 .. k {
        for y in 0 .. k-1 {
            let &a_i = coord_to_index.get(&(x,y)).unwrap();
            let &b_i = coord_to_index.get(&(x,y+1)).unwrap();
            net.igraph.add_edge(a_i, b_i, edge_length_range.ind_sample(rng));
        }
    }

    net
}

/// Add a random graph of a two dimensional grid.
pub fn random_weighted_net_and_grid2_chord<R: Rng>(k: usize, num_neighbors: usize,
        min_edge_len: u64, max_edge_len: u64, l: usize, mut rng: &mut R) 
        -> Network<RingKey> {
    
    // First create a random chord 2d grid:
    let mut net = random_weighted_net_grid2_chord(min_edge_len,max_edge_len,k,l, &mut rng);
    let num_nodes = k*k;
    assert!(num_nodes == net.igraph.node_count());

    let edge_length_range: Range<u64> = Range::new(min_edge_len,max_edge_len);

    // Next we add the random edges between the nodes in the 2d grid:
    for v in 0 .. num_nodes {
        for _ in 0 .. num_neighbors {
            let rand_node: Range<usize> = Range::new(0,num_nodes);
            let u = rand_node.ind_sample(rng);
            if u == v  {
                // Avoid self loops
                continue
            }
            if net.igraph.contains_edge(v,u) {
                // Already has this edge.
                continue
            }
            // Add edge:
            net.igraph.add_edge(v,u,edge_length_range.ind_sample(rng));
        }
    }
    net
}

/// Generate a random planar like network.
/// Put nodes randomly in the plane and connect every node to the num_cons closest nodes.
pub fn random_weighted_net_planar<R: Rng>(num_nodes: usize, num_cons: usize,
      min_edge_len: u64, max_edge_len: u64, l: usize, rng: &mut R) -> Network<RingKey> {

    let mut net = Network::<RingKey>::new();
    // let mut coord_to_index: HashMap<(usize, usize),usize>  = HashMap::new();
    let mut index_to_coord: HashMap<usize, (u64, u64)> = HashMap::new();
    // let mut key_to_coord: HashMap<RingKey, (usize, usize)>  = HashMap::new();
    let edge_length_range: Range<u64> = Range::new(min_edge_len,max_edge_len);

    // Maximum key in the ring:
    let max_key = 2_u64.pow(l as u32);

    let coord_range: Range<u64> = Range::new(0,2_u64.pow(30_u32));

    // Randomize all nodes:

    // A hash set to make sure we don't have duplicate keys.
    let mut chosen_keys: HashSet<RingKey> = HashSet::new();

    for _ in 0 .. num_nodes {
        let rand_key: Range<RingKey> = Range::new(0,max_key);
        let mut node_key = rand_key.ind_sample(rng);
        while chosen_keys.contains(&node_key) {
            node_key = rand_key.ind_sample(rng);
        }
        chosen_keys.insert(node_key.clone());
        let node_index = net.add_node(node_key);

        // Generate a random coordinate in the plane for the new node:
        let x = coord_range.ind_sample(rng);
        let y = coord_range.ind_sample(rng);

        // Add coordinate entry to map:
        index_to_coord.insert(node_index, (x,y));
    }

    let planar_dist = |i: usize, j:usize| {
        let (a,b) = index_to_coord.get(&i).unwrap().clone();
        let (c,d) = index_to_coord.get(&j).unwrap().clone();
        (c - a).pow(2) + (d - b).pow(2)
    };

    for u in 0 .. num_nodes {
        let lt = |&a: &usize, &b: &usize| planar_dist(a,u) < planar_dist(b,u);
        let mut sk = SmallestK::new(num_cons, &lt);
        for j in 0 .. num_nodes {
            sk.update(&j);
        }
        // Add edges to all planar closest nodes:
        for v in sk.smallest_k {
            if net.igraph.contains_edge(u,v) {
                // Already has this edge.
                continue
            }
            net.igraph.add_edge(u,v,edge_length_range.ind_sample(rng));
        }
    }

    net
}

/// Generate a network according to given type.
/// g -- amount of nodes (logarithmic).
/// l -- maximum key space for chord based networks (logarithmic)
pub fn gen_network<R:Rng>(net_type: usize, g: usize,l: usize, 
        min_weighted_len: u64, max_weighted_len: u64, mut rng: &mut R) -> Network<RingKey> {
    assert!(l >= 2*g, "Key collisions are too likely!");
    match net_type {
        0 => {
            // Random network.
            let num_nodes: usize = ((2 as u64).pow(g as u32)) as usize;
            let num_neighbors: usize = (1.5 * (num_nodes as f64).ln()) as usize;
            random_weighted_net_chord(num_nodes, num_neighbors, min_weighted_len, max_weighted_len,  l, &mut rng)
        }
        1 => {
            // 2d grid with random chord ids
            let num_nodes: usize = ((2 as u64).pow(g as u32)) as usize;
            let l: usize = (2 * g + 1)  as usize;
            let k = (num_nodes as f64).sqrt() as usize;
            random_weighted_net_grid2_chord(min_weighted_len, max_weighted_len, k, l, &mut rng)
        }
        2 => {
            // 2d grid combined with random network
            let num_nodes: usize = ((2 as u64).pow(g as u32)) as usize;
            let l: usize = (2 * g + 1)  as usize;
            let k = (num_nodes as f64).sqrt() as usize;
            let num_neighbors: usize = (1.5 * (num_nodes as f64).ln()) as usize;
            random_weighted_net_and_grid2_chord(k, num_neighbors, min_weighted_len, max_weighted_len, l, &mut rng)
        }
        3 => {
            // planar like network
            let num_nodes: usize = ((2 as u64).pow(g as u32)) as usize;
            let l: usize = (2 * g + 1)  as usize;
            // let num_neighbors: usize = (1.5 * (num_nodes as f64).ln()) as usize;
            let num_neighbors = 3;
            random_weighted_net_planar(num_nodes, num_neighbors, min_weighted_len, max_weighted_len, l, rng)
        }
        _ => unreachable!()

    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use self::rand::{StdRng};

    #[test]
    fn test_random_net_chord() {
        let seed: &[_] = &[1,2,3,4,9];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        let num_nodes = 5;
        let num_neighbors = 2;
        let l: usize = 6; // Size of keyspace
        random_net_chord(num_nodes,num_neighbors,l,&mut rng);
    }

    #[test]
    fn test_random_weighted_net_chord() {
        let seed: &[_] = &[1,2,3,4,9];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        let num_nodes = 5;
        let num_neighbors = 2;
        let l: usize = 6; // Size of keyspace
        random_weighted_net_chord(num_nodes,num_neighbors,100,104,l,&mut rng);
    }

    #[test]
    fn test_weighted_net_grid2_chord() {
        let seed: &[_] = &[1,2,3,4,9];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        let k = 5; // 5 X 5 grid
        let l: usize = 6; // Size of keyspace
        random_weighted_net_grid2_chord(1, 2, k,l,&mut rng);
    }

    #[test]
    fn test_random_weighted_net_and_grid2_chord() {
        let seed: &[_] = &[1,2,3,4,9];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        let num_neighbors = 2;
        let k = 5; // 5 X 5 grid
        let l: usize = 6; // Size of keyspace
        random_weighted_net_and_grid2_chord(k,num_neighbors, 1, 2, l, &mut rng);
    }
}
