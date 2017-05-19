extern crate petgraph;
extern crate rand;

pub mod ids_chain;
pub mod chains_array;

use std::collections::{HashSet};

use self::rand::{Rng, StdRng};
use self::rand::distributions::{IndependentSample, Range};

use network::{Network};
use self::ids_chain::{ids_chain};
use self::chains_array::{ChainsArray};


pub type RingKey = u64; // A key in the chord ring
pub type NodeChain = Vec<RingKey>;
pub type NeighborConnector = Vec<NodeChain>;


pub struct ChordFingers {
    left: NodeChain, 
    right_positive: Vec<NodeChain>,
    right_negative: Vec<NodeChain>,
    // Connectors for neighbors:
    neighbor_connectors: Vec<NeighborConnector>,

    right_randomized: Vec<NodeChain>,

    // Additional random fingers from the keyspace:
    fully_randomized: Vec<NodeChain>, 
}

// TODO: Keep ids of maintained nodes instead of all fragmented types
// of fingers. Possibly make two types: right and left.
// Generate ids only once, and since then use the ids blindly.

/// Create initial ChordFingers structure for node with index x_i
fn init_node_chord_fingers(x_i: usize, net: &Network<RingKey>, l: usize) 
    -> ChordFingers {

    let x_id = net.index_to_node(x_i).unwrap().clone();

    let mut cf = ChordFingers {
        left: vec![x_id],
        right_positive: Vec::new(),
        right_negative: Vec::new(),
        neighbor_connectors: Vec::new(),
        right_randomized: Vec::new(),
        fully_randomized: Vec::new(),
    };

    for i in 0 .. l {
        cf.right_positive.push(vec![x_id]);
        cf.right_negative.push(vec![x_id]);
        cf.right_randomized.push(vec![x_id]);
        cf.fully_randomized.push(vec![x_id]);
    }

    // Initialize neighbor connectors (Depends on neighbors):
    let mut s_neighbors: Vec<usize> =
        net.igraph.neighbors(x_i).collect::<Vec<_>>();


    s_neighbors.sort();

    for (neighbor_vec_index, &neighbor_index) in s_neighbors.iter().enumerate() {
        let neighbor_id: RingKey = net.index_to_node(neighbor_index).unwrap().clone();
        cf.neighbor_connectors.push(NeighborConnector::new());
        for (j,cur_id) in ids_chain(x_id, neighbor_id).enumerate() {
            cf.neighbor_connectors[neighbor_vec_index].push(vec![x_id]);
        }
    }

    // assert!(verify_fingers(x_id, &cf, &net));

    cf
}


/// Calculate ring distance from x to y clockwise
fn vdist(xk:RingKey, yk: RingKey, l: usize) -> RingKey {
    (yk.wrapping_sub(xk)) % 2_u64.pow(l as u32)
}


fn extract_chains<'a> (fingers: &'a ChordFingers) -> 
    Vec<&'a NodeChain> {

    let mut res: Vec<&NodeChain> = Vec::new();
    res.push(&fingers.left);
    {
        let mut push_chains = |chains: &'a Vec<NodeChain>| {
            for chain in chains {
                res.push(chain);
            }
        };
        push_chains(&fingers.right_positive);
        push_chains(&fingers.right_negative);
        push_chains(&fingers.right_randomized);
        for conn in &fingers.neighbor_connectors {
            push_chains(&conn)
        }
    }

    res
}

/// Make sure that a given chain is made of adjacent nodes.
fn verify_chain(chain: &NodeChain, net: &Network<RingKey>) -> bool {
    for i in 0 .. (chain.len() - 1) {
        let a = net.node_to_index(&chain[i]).unwrap();
        let b = net.node_to_index(&chain[i+1]).unwrap();
        if !net.igraph.contains_edge(a,b) {
            return false
        }
    }
    true
}

/*
fn verify_fingers(x_id: RingKey, chord_fingers: &ChordFingers, 
          net: &Network<RingKey>) -> bool {

    let l = chord_fingers.right_positive.len();

    // A function to get the top element:
    let check_chain = |chain: &NodeChain| (chain[chain.len() - 1] == x_id) &&
        verify_chain(chain, &net);

    let mut res: bool = true;

    res &= check_chain(&chord_fingers.left);

    for i in 0 .. l {
        res &= check_chain(&chord_fingers.right_positive[i]);
        res &= check_chain(&chord_fingers.right_negative[i]);
        res &= check_chain(&chord_fingers.right_randomized[i]);
        res &= check_chain(&chord_fingers.fully_randomized[i]);
    }

    for nei_connector in chord_fingers.neighbor_connectors.iter() {
        for chain in nei_connector {
            res &= check_chain(&chain);
        }
    }
    res

}
*/


/// Add an id to chain. Eliminate cycle if created.
fn add_id_to_chain(chain: &mut NodeChain, id: RingKey) {
    match chain.iter().position(|&x| x == id) {
        None => {
            chain.push(id);
        },
        Some(position) => {
            chain.resize(position + 1, 0);
        }
    };
}

/// Prepare all candidate chains for node x_i.
/// New best chains will be chosen out of those chains.
fn prepare_candidates(x_id: RingKey, net: &Network<RingKey>, 
                      fingers: &Vec<ChordFingers>) -> ChainsArray {

    let x_i = net.node_to_index(&x_id).unwrap();

    // Collect all chains to one vector. 
    let mut candidates: ChainsArray = ChainsArray::new();

    // Add trivial chain (x):
    candidates.insert_chain(vec![x_id]);

    // Add trivial chains (x,nei) where nei is any neighbor of x:
    for neighbor_index in net.igraph.neighbors(x_i) {
        candidates.insert_chain(vec![net.index_to_node(neighbor_index).unwrap().clone(), x_id])
    }

    // Add all "proposed" chains from all neighbors:
    for neighbor_index in net.igraph.neighbors(x_i) {

        let neighbor_id = net.index_to_node(neighbor_index).unwrap().clone();
        // assert!(verify_fingers(neighbor_id, &fingers[neighbor_index], &net));

        // Add proposed chains:
        for &chain in extract_chains(&fingers[neighbor_index]).iter() {
            assert!(chain[chain.len() - 1] == neighbor_id);

            let mut cchain = chain.clone();
            // Add our own id to the chain, possibly eliminating cycles:
            assert!(verify_chain(&cchain, net));
            add_id_to_chain(&mut cchain, x_id);
            assert!(verify_chain(&cchain, &net));
            assert!(cchain[cchain.len() - 1] == x_id);
            candidates.insert_chain(cchain);
        }
    }

    // Add all current chains:
    for chain in extract_chains(&fingers[x_i]) {
        candidates.insert_chain(chain.clone());
    }

    candidates.index();
    candidates
}


/// Checksum the contents of a chain
fn csum_chain(chain: &NodeChain) -> u64 {
    chain.iter().fold(0, |acc, &x| acc.wrapping_add(x))
}

///
/// Assign value and check if value has changed.
fn assign_check_changed<T: Eq>(dest: &mut T, src: T) -> bool {
    let has_changed: bool = *dest != src;
    *dest = src;
    has_changed
}


/// Perform one fingers iteration for node x: 
/// Take all chains from neighbors and update own chains to the best found chains.
/// Return true if any assignment has changed, false otherwise (Stationary state).
pub fn iter_node_fingers(x_i: usize, net: &Network<RingKey>, 
             fingers: &mut Vec<ChordFingers>, fingers_seed: usize, l: usize) -> bool {

    let mut has_changed: bool = false;
    let x_id = net.index_to_node(x_i).unwrap().clone();


    // Get all chain candidates:
    let candidates = prepare_candidates(x_id, &net, &fingers);

    // Update left finger:
    has_changed |= assign_check_changed(&mut fingers[x_i].left, 
        candidates.find_closest_left(x_id).clone());

    // Update all right fingers:
    for i in 0 .. l {
        has_changed |= assign_check_changed(&mut
            fingers[x_i].right_positive[i], 
                candidates.find_closest_right((x_id + 2_u64.pow(i as u32)) % 2_u64.pow(l as u32)).clone());

    }
    for i in 0 .. l {
        has_changed |= assign_check_changed(&mut
            fingers[x_i].right_negative[i], 
                candidates.find_closest_right((x_id.wrapping_sub(2_u64.pow(i as u32))) % 2_u64.pow(l as u32)).clone());
    }

    // Update neighbor connectors.
    // For determinism, we sort the neighbors before iterating.
    let mut s_neighbors: Vec<usize> =
        net.igraph.neighbors(x_i).collect::<Vec<_>>();
    s_neighbors.sort();

    for (neighbor_vec_index, &neighbor_index) in s_neighbors.iter().enumerate() {
        let neighbor_id: RingKey = net.index_to_node(neighbor_index).unwrap().clone();

        for (j,cur_id) in ids_chain(x_id, neighbor_id).enumerate() {
            has_changed |= assign_check_changed(
                &mut fingers[x_i].neighbor_connectors[neighbor_vec_index][j], 
                candidates.find_closest_right(cur_id).clone());
        }
    }

    // Obtain deterministic rng to be used with the following randomized
    // fingers:
    let seed: &[_] = &[fingers_seed as usize, x_i as usize];
    let mut rng: StdRng = rand::SeedableRng::<&[usize]>::from_seed(seed);

    // Update right randomized fingers:
    for i in 0 .. l {
        // Randomize a finger value in [2^i, 2^(i+1))
        let rand_range: Range<RingKey> = 
            Range::new(2_u64.pow(i as u32),2_u64.pow((i + 1) as u32));
        let rand_id = rand_range.ind_sample(&mut rng);
        has_changed |= assign_check_changed(&mut fingers[x_i].right_randomized[i], 
                        candidates.find_closest_right(rand_id).clone());
    }

    // Update random fingers:
    for i in 0 .. l {
        // Randomize a finger value in [0, 2^l). Completely random in the ring key space.
        let rand_range: Range<RingKey> = Range::new(0u64,2_u64.pow(l as u32));
        let rand_id = rand_range.ind_sample(&mut rng);
        has_changed |= assign_check_changed(&mut fingers[x_i].fully_randomized[i], 
            candidates.find_closest_right(rand_id).clone());
    }

    // assert!(verify_fingers(x_id, &fingers[x_i], &net));

    has_changed
}

/// Iter all nodes fingers. Return true if anything has changed,
/// false otherwise (Stationary state)
fn iter_fingers(net: &Network<RingKey>, 
             mut fingers: &mut Vec<ChordFingers>, fingers_seed: usize, l: usize) -> bool {

    // Has anything changed in the chosen fingers:
    let mut has_changed = false;
    for node_index in net.igraph.nodes() {
        has_changed |= iter_node_fingers(node_index, &net, &mut fingers, fingers_seed, l);
    }
    has_changed
}

pub fn converge_fingers(net: &Network<RingKey>, 
             mut fingers: &mut Vec<ChordFingers>, fingers_seed: usize, l: usize) {
    println!("iter_fingers...");
    while iter_fingers(&net, &mut fingers, fingers_seed, l) {
        println!("iter_fingers...");
    }
}

/// Create initial chord fingers for all the nodes in the network.
pub fn init_chord_fingers(net: &Network<RingKey>, l: usize) -> Vec<ChordFingers> {
    let mut chord_fingers_res: Vec<ChordFingers> = Vec::new();
    for node_index in net.igraph.nodes() {
        let chord_fingers = init_node_chord_fingers(node_index, &net, l);
        chord_fingers_res.push(chord_fingers);
    }
    chord_fingers_res
}

/*
fn route_chains(net: &Network<RingKey>, fingers: &Vec<ChordFingers>, l:usize) {

}
*/

/// Get routing chains for a given node.
/// Includes Neighbor of Neighbor chains too.
fn get_route_chains_node(x_i: usize, net: &Network<RingKey>, 
            fingers: &Vec<ChordFingers>, l:usize) -> ChainsArray {

    let mut route_chains = ChainsArray::new();
    // Get all chains of order 1:
    let chains1 = extract_chains(&fingers[x_i]);

    // First add all chains of order 1:
    for &chain in &chains1 {
        route_chains.insert_chain(chain.clone());
    }

    // Find all chains of order 2:

    for chain in chains1 {
        let neighbor_id = chain[chain.len() - 1];
        let neighbor_index = net.node_to_index(&neighbor_id).unwrap();
        let neighbor_chains = extract_chains(&fingers[neighbor_index]);
        for nchain in neighbor_chains {
            let mut concat_chain = nchain.clone();
            concat_chain.extend(chain.iter().skip(1));
            route_chains.insert_chain(concat_chain);
        }
    }

    route_chains.index();
    route_chains
}

/*
 *
/// Check if current fingers values are globally optimal for the node
/// with id x_id.
fn verify_global_optimality_node(x_id: RingKey, net: &Network<RingKey>,
            sorted_node_keys: &Vec<RingKey>, node_fingers: &ChordFingers, l: usize) -> bool {

    let get_right = |target_id| 
        sorted_node_keys[sorted_node_keys.binary_search(target_id) % sorted_node_keys.len()];

    let get_left = |target_id| 
        sorted_node_keys[
            (sorted_node_keys.len() - 1 + sorted_node_keys.binary_search(target_id)) 
                % sorted_node_keys.len()];

    // Checking left:
    if node_fingers.left[0] != get_left(x_id.wrapping_sub(1) % 2_u64.pow(l as u32)) {
        return false;
    }
    

    // TODO: Continue here.
    assert!(false);

    true
}

/// Verify that the current finger values are globally best for all nodes.
fn verify_global_optimality(net: &Network<RingKey>, 
           fingers: &Vec<ChordFingers>, l: usize) -> bool {

    // Get all node keys in the network:
    node_keys = net.igraph.nodes()
        .map(|node_index| net.index_to_node(node_index).unwrap().clone())
        .collect::<Vec<RingKey>>();

    node_keys.sort();

    for node_index in net.igraph.nodes() {
        let node_id = net.index_to_node(node_index).unwrap().clone();
        if !verify_global_optimality_node(node_id, &net, &node_keys, 
                                          &fingers[node_index], l) {
            return false;
        }
    }
    true

}

*/

/// Create indexed route chains ChainsArray structs for all nodes.
pub fn get_route_chains(net: &Network<RingKey>, 
                    fingers: &Vec<ChordFingers>, l:usize) -> Vec<ChainsArray> {

    net.igraph.nodes()
        .map(|node_index| get_route_chains_node(node_index, &net, &fingers, l))
        .collect::<Vec<ChainsArray>>()
}


/// Find next best chain of steps in the network to arrive the node dst_index.
fn next_chain(cur_id: RingKey, dst_id: RingKey, 
        net: &Network<RingKey>, route_chains: &Vec<ChainsArray>, l: usize)
            -> Option<NodeChain>{
    
    println!("cur_id = {}", cur_id);
    println!("dst_id = {}", dst_id);

    let cur_node_index = net.node_to_index(&cur_id).unwrap();
    println!("cur_node_index = {}", cur_node_index);
    println!("dst_node_index = {}", net.node_to_index(&dst_id).unwrap());

    let best_chain: NodeChain = 
        route_chains[cur_node_index].find_closest_left(dst_id).clone();

    // If chain leads to us, return None. Otherwise return the chain.
    match best_chain[0] == cur_id {
        true => None,
        false => Some(best_chain)
    }
}

/// Find a path between src_id and dst_id
/// Return the full path as a chain of node ids.
pub fn find_path(src_id: RingKey, dst_id: RingKey, net: &Network<RingKey>, 
    route_chains: &Vec<ChainsArray>, l: usize) -> Option<NodeChain> {

    let mut total_chain: NodeChain = NodeChain::new();
    total_chain.push(src_id);
    let mut cur_id = src_id;
    while cur_id != dst_id {
        let cur_chain_wrapped = next_chain(cur_id, dst_id, &net, &route_chains, l);
        match cur_chain_wrapped {
            Some(mut cur_chain) => {
                println!("cur_chain = {:?}", cur_chain);
                total_chain.pop(); // Avoid duplicity
                cur_chain.reverse();
                total_chain.extend(cur_chain);
                cur_id = total_chain[total_chain.len() - 1];

            },
            None => {
                println!("next chain not found!");
                return None;
            }
        };
        // Check if total_chain has got too long:
        if total_chain.len() > net.igraph.node_count() {
            println!("Too long!");
            return None;
        }
    }
    Some(total_chain)
}

/// Generate a random graph to be used with chord.
/// Graph nodes are of type RingKey.
pub fn random_net_chord<R: Rng>(num_nodes: usize, num_neighbors: usize, l: usize, rng: &mut R) 
        -> Network<RingKey> {

    // Maximum key in the ring:
    let max_key = 2_u64.pow(l as u32);

    // A hash set to make sure we don't have duplicate keys.
    let mut chosen_keys: HashSet<RingKey> = HashSet::new();

    // We can't have too many nodes with respect to the keyspace.
    // We stay below sqrt(keyspace_size), to avoid collisions.
    assert!(num_nodes < (max_key as f64).sqrt() as usize, "Too many nodes!");
    assert!(num_nodes > 0, "We should have at least one node!");

    let mut net = Network::<RingKey>::new();

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

    // Add a straight line, to ensure connectivity.
    // Possibly change this later to a random tree.
    for v in 0 .. num_nodes - 1 {
        net.igraph.add_edge(v, v + 1, 1);
        // println!("add_edge {}, {}",v,v + 1);
    }

    // Connect node v to about num_neighbors previous nodes:
    // This should ensure connectivity, even if num_neighbors is small.
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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_d() {
        let l = 42;
        assert!(vdist(1u64,2,l) == 1);
        assert!(vdist(1u64,101,l) == 100);
        assert!(vdist(2_u64.pow(l as u32) - 1,1,l) == 2);
        assert!(vdist(2_u64.pow(l as u32) - 1,0,l) == 1);
        assert!(vdist(1,0,l) == 2_u64.pow(l as u32) - 1);
    }


    #[test]
    fn test_csum_chain() {
        assert!(csum_chain(&vec![1,2,3,4]) == 10);
        assert!(csum_chain(&vec![]) == 0);
        assert!(csum_chain(&vec![1]) == 1);
    }

    #[test]
    fn test_inner_lexicographic() {
        // Make sure that vectors participate inside
        // lexicographic comparison.
        let d = (1,2,vec![3,6]);
        let a = (1,2,vec![3,4]);
        let b = (1,2,vec![3,5]);
        let c = (1,2,vec![3,6]);

        assert!(a < b);
        assert!(a < c);
        assert!(a < d);

        let aa = (1,2,vec![4,4]);
        assert!(aa > a);

        let m = (5,2,vec![1,1]);
        assert!(m > a);
    }

    #[test]
    fn test_assign_check_changed() {
        let mut x = 5;
        assert!(!assign_check_changed(&mut x, 5));
        assert!(assign_check_changed(&mut x, 6));
        assert!(!assign_check_changed(&mut x, 6));
        assert!(assign_check_changed(&mut x, 7));
    }

    #[test]
    fn test_add_id_to_chain_basic() {
        let mut chain = vec![1,2,3,4,5];
        add_id_to_chain(&mut chain, 3);
        assert!(chain == vec![1,2,3]);

        let mut chain = vec![1,2,3];
        add_id_to_chain(&mut chain, 3);
        assert!(chain == vec![1,2,3]);

        let mut chain = vec![1,2,3];
        add_id_to_chain(&mut chain, 4);
        assert!(chain == vec![1,2,3,4]);

        let mut chain = vec![1,2,3];
        add_id_to_chain(&mut chain, 1);
        assert!(chain == vec![1]);

        let mut chain = vec![1,2];
        add_id_to_chain(&mut chain, 1);
        assert!(chain == vec![1]);

        let mut chain = vec![1];
        add_id_to_chain(&mut chain, 1);
        assert!(chain == vec![1]);

    }

    #[test]
    fn test_chord_basic() {
        let seed: &[_] = &[1,2,3,4,9];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        let num_nodes = 5;
        let num_neighbors = 2;
        let l: usize = 6; // Size of keyspace
        let net = random_net_chord(num_nodes,num_neighbors,l,&mut rng);
        let mut chord_fingers = init_chord_fingers(&net,l);
        let fingers_seed = 0x1339;
        converge_fingers(&net, &mut chord_fingers, fingers_seed,l);

        let route_chains = get_route_chains(&net, &chord_fingers, l);

        for index_a in 0 .. num_nodes {
            for index_b in index_a + 1 .. num_nodes {
                // Try to find a path:
                let src_id = net.index_to_node(index_a).unwrap().clone();
                let dst_id = net.index_to_node(index_b).unwrap().clone();
                let path = find_path(src_id, dst_id, &net, &route_chains, l).unwrap();

                // Make sure that all nodes in the path are connected by edges in the graph:
                for i in 0 .. (path.len() - 1) {
                    let a = net.node_to_index(&path[i]).unwrap();
                    let b = net.node_to_index(&path[i+1]).unwrap();
                    assert!(net.igraph.contains_edge(a,b));
                }

                // Make sure that path begins with src_id and ends with dst_id:
                assert!(path[0] == src_id);
                assert!(path[path.len() - 1] == dst_id);
            }
        }
    }

}
