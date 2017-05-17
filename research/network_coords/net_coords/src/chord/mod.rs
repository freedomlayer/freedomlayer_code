extern crate petgraph;
extern crate rand;

pub mod ids_chain;

use network::{Network};
use self::ids_chain::{ids_chain};

use self::rand::{Rng, StdRng};
use self::rand::distributions::{Weighted, WeightedChoice, 
    IndependentSample, Range};

type RingKey = u64; // A key in the chord ring
type NodeChain = Vec<RingKey>;
type NeighborConnector = Vec<NodeChain>;


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
fn prepare_candidates(x_id: RingKey, net: &Network<RingKey>, 
                      fingers: &Vec<ChordFingers>) -> Vec<NodeChain> {

    let x_i = net.node_to_index(&x_id).unwrap();

    // Collect all chains to one vector. 
    let mut candidates: Vec<NodeChain> = Vec::new();

    // Add trivial chain (x):
    candidates.push(vec![x_id]);

    // Add trivial chains (x,nei) where nei is any neighbor of x:
    for neighbor_index in net.igraph.neighbors(x_i) {
        candidates.push(vec![net.index_to_node(neighbor_index).unwrap().clone(), x_id])
    }

    // Add all "proposed" chains from all neighbors:
    for neighbor_index in net.igraph.neighbors(x_i) {
        // Add trivial chain (x,nei):
        candidates.push(vec![net.index_to_node(neighbor_index).unwrap().clone(), x_id]);

        // Add proposed chains:
        for &chain in extract_chains(&fingers[neighbor_index]).iter() {
            let mut cchain = chain.clone();
            // Add our own id to the chain, possibly eliminating cycles:
            add_id_to_chain(&mut cchain, x_id);
            candidates.push(cchain);
        }
    }

    // Add all current chains:
    candidates.extend(
        extract_chains(&fingers[x_i]).iter().map(|&chain| chain.clone())
    );

    candidates
}


/// Checksum the contents of a chain
fn csum_chain(chain: &NodeChain, l: usize) -> RingKey {
    chain.iter().fold(0, |acc, &x| acc.wrapping_add(x) % (2_u64.pow(l as u32)))
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
        candidates.iter().min_by_key(|c: &&NodeChain| 
            (vdist(c[0], x_id,l), c.len(), csum_chain(c,l) )).unwrap().clone());

    // Find the chain that is closest to target_id from the right.
    // Lexicographic sorting: 
    // We first care about closest id in keyspace. Next we want the shortest chain possible.
    let best_right_chain = |target_id| candidates.iter().min_by_key(|c| 
             (vdist(target_id, c[0],l), c.len(), csum_chain(c,l) )).unwrap().clone();

    // Update all right fingers:
    for i in 0 .. l {
        has_changed |= assign_check_changed(&mut
            fingers[x_i].right_positive[i], 
                best_right_chain((x_id + 2_u64.pow(i as u32)) % 2_u64.pow(l as u32)));

    }
    for i in 0 .. l {
        has_changed |= assign_check_changed(&mut
            fingers[x_i].right_negative[i], 
                best_right_chain((x_id.wrapping_sub(2_u64.pow(i as u32))) % 2_u64.pow(l as u32)));
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
                best_right_chain(cur_id));
        }
    }

    // Obtain deterministic rng to be used with the following randomized
    // fingers:
    let seed: &[_] = &[fingers_seed as usize, x_i as usize];
    let mut rng: StdRng = rand::SeedableRng::<&[usize]>::from_seed(seed);

    // Update right randomized fingers:
    for i in 0 .. l {
        // Randomize a finger value in [2^i, 2^(i+1))
        let rand_range: Range<RingKey> = Range::new(2_u64.pow(i as u32),2_u64.pow((i + 1) as u32));
        let rand_id = rand_range.ind_sample(&mut rng);
        has_changed |= assign_check_changed(&mut fingers[x_i].right_randomized[i], 
                        best_right_chain(rand_id));
    }

    // Update random fingers:
    for i in 0 .. l {
        // Randomize a finger value in [0, 2^l). Completely random in the ring key space.
        let rand_range: Range<RingKey> = Range::new(0u64,2_u64.pow(l as u32));
        let rand_id = rand_range.ind_sample(&mut rng);
        has_changed |= assign_check_changed(&mut fingers[x_i].fully_randomized[i], 
            best_right_chain(rand_id));
    }

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

/// Find next best chain of steps in the network to arrive the node dst_index.
fn next_chain(cur_id: RingKey, dst_id: RingKey, 
        net: &Network<RingKey>, fingers: &Vec<ChordFingers>, l: usize)
            -> Option<NodeChain>{

    // Get all chains of order 1:
    let chains1 = prepare_candidates(cur_id, &net, &fingers);
    let mut all_chains: Vec<NodeChain> = chains1.clone();

    // Collect all relevant chains:
    // - Regular one iteration chain.
    // - Two iters chains: According to "Know thy neighbor" article
    for c1 in chains1.iter() {
        let vneighbor_id: RingKey = c1[0];

        // Concatenate pairs of chains.
        // Remember that a chain from id x to id y is of the form:
        // 0 1 2 3 4  <-- Vector index
        // y . . . x  <-- Value
        all_chains.extend(
            prepare_candidates(vneighbor_id, &net, &fingers).iter()
                .map(|c| {
                    let mut clone: NodeChain = c.clone();
                    // Concatenate chains:
                    clone.extend(c1.iter().skip(1).cloned().collect::<NodeChain>());
                    clone
                })
                .collect::<Vec<NodeChain>>()
        );
    }

    
    // Pick the closest chain, using csum_chain as tie breaker:
    let best_chain: NodeChain = all_chains.iter().min_by_key(|c: &&NodeChain| 
         ( vdist(c[0], dst_id,l), c.len(), csum_chain(c,l) )).unwrap().clone();

    // If chain leads to us, return None. Otherwise return the chain.
    match best_chain[0] == cur_id {
        true => None,
        false => Some(best_chain)
    }
}

/// Find a path between src_id and dst_id
/// Return the full path as a chain of node ids.
pub fn find_path(src_id: RingKey, dst_id: RingKey, net: &Network<RingKey>, 
    fingers: &Vec<ChordFingers>, l: usize) -> Option<NodeChain> {

    let mut total_chain: NodeChain = NodeChain::new();
    total_chain.push(src_id);
    let mut cur_id = src_id;
    while cur_id != dst_id {
        let cur_chain_wrapped = next_chain(cur_id, dst_id, &net, &fingers, l);
        match cur_chain_wrapped {
            Some(mut cur_chain) => {
                total_chain.pop(); // Avoid duplicity
                cur_chain.reverse();
                total_chain.extend(cur_chain);
                cur_id = total_chain[total_chain.len() - 1];

            },
            None => return None,
        };
        // Check if total_chain has got too long:
        if total_chain.len() > net.igraph.node_count() {
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

    // We can't have too many nodes with respect to the keyspace.
    // We stay below sqrt(keyspace_size), to avoid collisions.
    assert!(num_nodes < (max_key as f64).sqrt() as usize, "Too many nodes!");
    assert!(num_nodes > 0, "We should have at least one node!");

    let mut net = Network::<RingKey>::new();

    // Insert num_nodes nodes with random keys:
    for _ in 0 .. num_nodes {
        let rand_key: Range<RingKey> = Range::new(0,max_key);
        let node_key = rand_key.ind_sample(rng);
        net.add_node(node_key);
    }

    // Connect node v to about num_neighbors previous nodes:
    // This should ensure connectivity, even if num_neighbors is small.
    for v in 1 .. num_nodes {
        for _ in 0 .. num_neighbors {
            let rand_prev_node: Range<usize> = Range::new(0,v);
            let u = rand_prev_node.ind_sample(rng);
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
        let l = 42;
        assert!(csum_chain(&vec![1,2,3,4],l) == 10);
        assert!(csum_chain(&vec![],l) == 0);
        assert!(csum_chain(&vec![1],l) == 1);
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
    fn test_chord_basic() {
        let seed: &[_] = &[1,2,3,4,9];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        let num_nodes = 30;
        let num_neighbors = 2;
        let l: usize = 10; // Size of keyspace
        let net = random_net_chord(num_nodes,num_neighbors,l,&mut rng);
        let mut chord_fingers = init_chord_fingers(&net,l);
        let fingers_seed = 0x1338;
        converge_fingers(&net, &mut chord_fingers, fingers_seed,l);

        // Try to find a path:
        let src_id = net.index_to_node(0).unwrap().clone();
        let dst_id = net.index_to_node(22).unwrap().clone();
        println!("src_id = {}", src_id);
        println!("dst_id = {}", dst_id);
        let path = find_path(src_id, dst_id, &net, &chord_fingers, l).unwrap();

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
