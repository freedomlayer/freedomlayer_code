extern crate petgraph;
extern crate rand;

pub mod ids_chain;
pub mod semi_routes_array;
pub mod node_fingers;

use std::collections::{HashSet, HashMap, VecDeque};

use self::rand::{Rng, StdRng};
use self::rand::distributions::{IndependentSample, Range};

use network::{Network};
use self::ids_chain::{ids_chain};
use self::semi_routes_array::{SemiRoutesArray, sroute_final_id};
use self::node_fingers::{NodeFingers, SemiChain};


pub type RingKey = u64; // A key in the chord ring
pub type NodeChain = Vec<RingKey>;
pub type NeighborConnector = Vec<NodeChain>;

type RouteField = Vec<HashMap<RingKey,SemiChain>>;
type SemiRoute = Vec<SemiChain>;


/// Calculate ring distance from x to y clockwise
fn vdist(xk:RingKey, yk: RingKey, l: usize) -> RingKey {
    (yk.wrapping_sub(xk)) % 2_u64.pow(l as u32)
}

/// Add cyclic (x + diff) % max_key
fn add_cyc(x: RingKey, diff: i64, l: usize) -> RingKey {
    let max_key = 2_u64.pow(l as u32);
    (if diff >= 0 {
        x.wrapping_add(diff as u64)
    } else {
        x.wrapping_sub((-diff) as u64)
    }) % max_key
}


/// Generate a vector of maintained left target_ids for node with id x_id.
fn gen_left_target_ids(x_id: RingKey, l: usize) -> Vec<RingKey> {
    vec![add_cyc(x_id,-1,l)]
}

/// Generate a vector of maintained right target_ids for node with id x_id.
fn gen_right_target_ids<R: Rng>(x_id: RingKey, net: &Network<RingKey>, 
                                l: usize, mut rng: &mut R) -> Vec<RingKey> {

    let mut target_ids: Vec<RingKey> = Vec::new();

    // Basic right fingers:
    for i in 0 .. l {
        let pow_val = 2_i64.pow(i as u32);
        target_ids.push(add_cyc(x_id,pow_val,l));
        target_ids.push(add_cyc(x_id,-pow_val,l));
    }

    // Neighbor connectors:
    let x_i = net.node_to_index(&x_id).unwrap();
    for neighbor_index in net.igraph.neighbors(x_i) {
        let neighbor_id: RingKey = net.index_to_node(neighbor_index).unwrap().clone();
        for cur_id in ids_chain(x_id, neighbor_id) {
            target_ids.push(cur_id);
        }
    }

    // Right randomized fingers:
    for i in 0 .. l {
        // Randomize a finger value in [2^i, 2^(i+1))
        let rand_range: Range<RingKey> = 
            Range::new(2_u64.pow(i as u32),2_u64.pow((i + 1) as u32));
        let rand_id = rand_range.ind_sample(&mut rng);
        target_ids.push(rand_id);
    }

    // Update random fingers:
    for _ in 0 .. l {
        // Randomize a finger value in [0, 2^l). Completely random in the ring key space.
        let rand_range: Range<RingKey> = Range::new(0u64,2_u64.pow(l as u32));
        let rand_id = rand_range.ind_sample(&mut rng);
        target_ids.push(rand_id);
    }
    
    target_ids
}

/// Initialize maintained fingers for node with index x_i.
fn create_node_fingers<R: Rng>(x_i: usize, net: &Network<RingKey>, 
             l: usize, mut rng: &mut R) -> NodeFingers {

    let x_id = net.index_to_node(x_i).unwrap().clone();
    let target_ids_left = gen_left_target_ids(x_id, l);
    let target_ids_right = gen_right_target_ids(x_id, &net, l, &mut rng);

    let mut nf = NodeFingers::new(x_id, &target_ids_left, &target_ids_right);
    
    nf
}

pub fn init_fingers<R: Rng>(net: &Network<RingKey>, 
                l: usize, mut rng: &mut R) -> Vec<NodeFingers> {

    let mut res_fingers = Vec::new();
    for x_i in 0 .. net.igraph.node_count() {
        res_fingers.push(create_node_fingers(x_i, &net, l, &mut rng));
    }

    res_fingers
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

struct PendingSemiChain {
    schain: SemiChain,
    origin_id: RingKey,
}

pub fn converge_fingers(net: &Network<RingKey>, 
             mut fingers: &mut Vec<NodeFingers>, l: usize) {

    let mut pending_chains: VecDeque<PendingSemiChain> = VecDeque::new();

    // First iteration: We insert all edges:
    for x_i in 0 .. net.igraph.node_count() {
        let x_id = net.index_to_node(x_i).unwrap().clone();
        for neighbor_i in net.igraph.neighbors(x_i) {
            // Note that that information about the origin of the change
            // is contained as the last element of the NodeChain.
            let neighbor_id = net.index_to_node(neighbor_i).unwrap().clone();

            // let chain = vec![neighbor_id, x_id];
            let schain = SemiChain {
                next_id: neighbor_id,
                final_id: neighbor_id,
                length: 1,
            };
            if fingers[x_i].update(&schain,l) {
                // If a change happened to x_id fingers, we queue the new chain:
                pending_chains.push_back(PendingSemiChain {
                    schain: schain, 
                    origin_id: x_id
                });
            }

        }
    }

    // Keep going as long as we have pending chains:
    while let Some(pending_schain) = pending_chains.pop_front() {
        let cur_id = pending_schain.origin_id;
        let cur_i = net.node_to_index(&cur_id).unwrap();
        for neighbor_i in net.igraph.neighbors(cur_i) {
            let neighbor_id = net.index_to_node(neighbor_i).unwrap().clone();
            let schain = SemiChain {
                next_id: cur_id,
                final_id: pending_schain.schain.final_id,
                length: pending_schain.schain.length + 1,
            };

            if fingers[neighbor_i].update(&schain,l) {
                pending_chains.push_back(PendingSemiChain {
                    schain: schain,
                    origin_id: neighbor_id,
                });
            }
        }
    }
}


/// Generate a routing field: Store for every node how to get to various
/// other nodes, by going through a neighbor.
pub fn create_route_field(net: &Network<RingKey>, fingers: &Vec<NodeFingers>, 
                      l: usize) -> RouteField {

    let mut route_field: RouteField = Vec::new();
    for x_i in 0 .. net.igraph.node_count() {
        let x_id = net.index_to_node(x_i).unwrap().clone();
        let mut route_map: HashMap<RingKey, SemiChain> = HashMap::new();

        for fing in &fingers[x_i].left.sorted_fingers {
            let should_insert = match route_map.get(&fing.schain.final_id) {
                None => true,
                Some(schain) => 
                    (schain.length, schain.next_id) < (fing.schain.length, fing.schain.next_id)
            };
            if should_insert {
                route_map.insert(fing.schain.final_id, fing.schain.clone());
            }
        }
        route_field.push(route_map);
    }

    route_field
}


fn create_semi_routes_node(x_i: usize, net: &Network<RingKey>, route_field: &RouteField,
                           l: usize) -> SemiRoutesArray {

    let mut semi_routes_array = SemiRoutesArray::new();

    for (target_id, schain) in &route_field[x_i] {
        let mut semi_route: Vec<SemiChain> = Vec::new();
        semi_route.push(schain.clone());
        semi_routes_array.insert_sroute(semi_route.clone());

        // Concat with another iteration of SemiChain 
        // (Implementing Neighbor of Neighbor method):
        
        let target_i = net.node_to_index(&target_id).unwrap();
        for (next_target_id, next_schain) in &route_field[target_i] {
            semi_route.push(next_schain.clone());
            semi_routes_array.insert_sroute(semi_route.clone());
            semi_route.pop();
        }
    }
    semi_routes_array.index();
    semi_routes_array
}

pub fn create_semi_routes(net: &Network<RingKey>, route_field: &RouteField,
                           l: usize) -> Vec<SemiRoutesArray> {

    let mut res_vec = Vec::new();
    for x_i in 0 .. net.igraph.node_count() {
        res_vec.push(create_semi_routes_node(x_i,&net, &route_field, l));
    }
    res_vec
}

pub fn find_path(src_id: RingKey, dst_id: RingKey, net: &Network<RingKey>, 
    route_field: &RouteField, semi_routes: &Vec<SemiRoutesArray>, l: usize) 
        -> Option<NodeChain> {

    let mut path: NodeChain = NodeChain::new();
    let mut cur_id = src_id;
    path.push(cur_id);
    while cur_id != dst_id {
        let cur_semi_routes = &semi_routes[net.node_to_index(&cur_id).unwrap()];

        let semi_route = cur_semi_routes.find_closest_left(dst_id);
        if sroute_final_id(semi_route) == cur_id {
            return None;
        }
        for semi_chain in semi_route {
            println!("---");
            println!("semi_chain: {:?}", semi_chain);
            let mut active_semi_chain = semi_chain.clone();
            while cur_id != semi_chain.final_id {
                cur_id = active_semi_chain.next_id;
                path.push(cur_id);
                println!("cur_id = {}", cur_id);
                println!("final_id = {}", semi_chain.final_id);
                let cur_i = net.node_to_index(&cur_id).unwrap();
                active_semi_chain = route_field[cur_i]
                    .get(&semi_chain.final_id).unwrap().clone();
            }
        }
    }

    Some(path)
}


/*

/// Follow semi chain to a full chain of ids.
fn follow_chain(orig_id: RingKey, schain: &SemiChain, net: &Network<RingKey>,
                fingers: &Vec<NodeFingers>, l: usize) -> NodeChain {

    let mut res_chain: NodeChain = NodeChain::new();
    
    let mut cur_schain = schain;
    res_chain.push(orig_id);
    while schain.next_id != schain.final_id {
        let next_i = net.node_to_index(&schain.next_id).unwrap();
        res_chain.push(schain.next_id);
    }

    res_chain

}



/// Get routing chains for a given node.
/// Includes Neighbor of Neighbor chains too.
fn get_route_chains_node(x_i: usize, net: &Network<RingKey>, 
            fingers: &Vec<NodeFingers>, l:usize) -> ChainsArray {

    let mut route_chains = ChainsArray::new();
    // Get all chains of order 1:
    let schains1: Vec<SemiChain> = fingers[x_i].all_chains();

    // First add all chains of order 1:
    for schain in &schains1 {
        route_chains.insert_chain(schain.clone());
    }

    // Find all chains of order 2:

    for chain in chains1 {
        let neighbor_id = chain[chain.len() - 1];
        let neighbor_index = net.node_to_index(&neighbor_id).unwrap();
        let neighbor_chains = fingers[neighbor_index].all_chains();
        for nchain in neighbor_chains {
            let mut concat_chain = nchain.clone();
            concat_chain.extend(chain.iter().skip(1));
            route_chains.insert_chain(concat_chain);
        }
    }

    route_chains.index();
    route_chains
}
*/

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

/*
/// Create indexed route chains ChainsArray structs for all nodes.
pub fn get_route_chains(net: &Network<RingKey>, 
                    fingers: &Vec<NodeFingers>, l:usize) -> Vec<ChainsArray> {

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
*/


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
    fn test_add_cyc() {
        // Check add:
        assert!(add_cyc(0,1,5) == 1);
        assert!(add_cyc(1,1,5) == 2);
        assert!(add_cyc(30,1,5) == 31);
        assert!(add_cyc(31,1,5) == 0);

        // Check sub:
        assert!(add_cyc(2,-1,5) == 1);
        assert!(add_cyc(1,-1,5) == 0);
        assert!(add_cyc(0,-1,5) == 31);
        assert!(add_cyc(31,-1,30) == 30);
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
        let mut fingers = init_fingers(&net,l, &mut rng);
        converge_fingers(&net, &mut fingers,l);
        let route_field = create_route_field(&net, &fingers, l);
        let semi_routes = create_semi_routes(&net, &route_field,l);

        // let route_chains = get_route_chains(&net, &fingers, l);

        for index_a in 0 .. num_nodes {
            for index_b in index_a + 1 .. num_nodes {
                // Try to find a path:
                let src_id = net.index_to_node(index_a).unwrap().clone();
                let dst_id = net.index_to_node(index_b).unwrap().clone();
                let path = find_path(src_id, dst_id, &net, 
                                     &route_field, &semi_routes, l).unwrap();

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
