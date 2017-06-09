pub mod coords;
pub mod coord_mappers;

extern crate rand;

use self::rand::{Rng};
use self::rand::distributions::{Weighted, WeightedChoice, 
    IndependentSample, Range};

use std::hash::Hash;

use network::{Network};
use landmarks::coord_mappers::{approx_max_dist, max_dist};


/// Try to find a path in the network between src_node and dst_node.
/// Using a variation of random walk.
/// Returns None if path was not found, or Some(path_length)
pub fn find_path_landmarks<R: Rng, Node: Hash + Eq + Clone>(src_node: usize, dst_node: usize, 
         amount_close: usize, net: &Network<Node>, 
         coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>,
         mut rng: &mut R) -> Option<u64> {

    // Node distance function:
    let node_dist = |x,y| approx_max_dist(x,y,&coords, &landmarks);
    // let calc_weight = |i: usize| ((-(node_dist(i, dst_node) as f64)).exp() * 100.0) as u32;
    let calc_weight = |_: usize| 1 as u32;

    let mut total_distance: u64 = 0;
    let mut cur_node = src_node;
    
    while cur_node != dst_node {
        let (mut new_cur_node, mut new_dist , _): (usize, u64, _) = 
            net.closest_nodes_structure(cur_node).take(amount_close)
                .min_by_key(|&(i, _, _)| node_dist(dst_node, i)).unwrap();

        if node_dist(new_cur_node, dst_node) >= node_dist(cur_node, dst_node) {

            // Pick a best local destination randomly in a "smart" way:
            let mut items = net.closest_nodes_structure(cur_node).take(amount_close)
                .map(|(i, dist, gateway)| 
                     Weighted { weight: calc_weight(i), item: (i, dist, gateway) })
                .collect::<Vec<_>>();

            // Pick the next step as the gateway of the chosen local destination:
            let wc = WeightedChoice::new(&mut items);
            let smp = wc.ind_sample(&mut rng);
            // gateway_index = smp.2;
            new_cur_node = smp.0;
            new_dist = smp.1;
        }

        total_distance += new_dist;
        // total_distance += *net.igraph.edge_weight(cur_node, gateway_index).unwrap();
        // The path is already too long. We abort.
        if total_distance as usize > net.igraph.node_count() {
            return None
        }
        cur_node = new_cur_node;
        // cur_node = gateway_index;

    }
    Some(total_distance)
}

/// Try to find a path in the network between src_node and dst_node.
/// Using a variation of random walk. Tries to return the closest node found to the given
/// coordinate.
/// Returns (node_index, path_length)
pub fn find_path_landmarks_by_coord<R: Rng, Node: Hash + Eq + Clone>(src_node: usize, dst_coord: &Vec<u64>, 
         amount_close: usize, max_visits: usize, net: &Network<Node>, 
         coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>,
         mut rng: &mut R) -> (usize, u64) {

    let _ = landmarks; // Currently unused.

    // Remember amount of visits to every node.
    // If a node is visited too many times, we return it (Even if it is not an exact match).
    let mut visits: Vec<usize> = vec![0; net.igraph.node_count()];

    // Node distance function:
    let node_dist = |x| max_dist(&coords[x], &dst_coord);
    // let calc_weight = |i: usize| ((-(node_dist(i, dst_node) as f64)).exp() * 100.0) as u32;
    let calc_weight = |_: usize| 1 as u32;

    let mut total_distance: u64 = 0;
    let mut cur_node = src_node;
    
    while visits[cur_node] < max_visits {
        let (mut new_cur_node, mut new_dist , _): (usize, u64, _) = 
            net.closest_nodes_structure(cur_node).take(amount_close)
                .min_by_key(|&(i, _, _)| node_dist(i)).unwrap();

        if node_dist(new_cur_node) >= node_dist(cur_node) {
            visits[cur_node] += 1;

            // Pick a best local destination randomly in a "smart" way:
            let mut items = net.closest_nodes_structure(cur_node).take(amount_close)
                .map(|(i, dist, gateway)| 
                     Weighted { weight: calc_weight(i), item: (i, dist, gateway) })
                .collect::<Vec<_>>();

            // Pick the next step as the gateway of the chosen local destination:
            let wc = WeightedChoice::new(&mut items);
            let smp = wc.ind_sample(&mut rng);
            // gateway_index = smp.2;
            new_cur_node = smp.0;
            new_dist = smp.1;
        }

        total_distance += new_dist;
        cur_node = new_cur_node;

    }
    // Wanted node was found!. We return it.
    (cur_node, total_distance)
}


/// Try to find a path in the network between src_node and dst_node.
/// This is done given an approximate coordinate of dst_node, which is not 
/// his exact coordinate.
/// Routing is done using a variation of random walk over landmarks coordinates.
/// Returns path_length
pub fn find_path_landmarks_approx<R: Rng, Node: Hash + Eq + Clone>(src_node: usize, dst_node: usize, approx_dst_coord: &Vec<u64>, 
         amount_close: usize, net: &Network<Node>, 
         coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>,
         mut rng: &mut R) -> u64 {

    let _ = landmarks; // Currently unused.

    // Node distance function:
    let node_dist = |x| max_dist(&coords[x], &approx_dst_coord);
    // let calc_weight = |i: usize| ((-(node_dist(i, dst_node) as f64)).exp() * 100.0) as u32;
    // let calc_weight = |_: usize| 1 as u32;
    let rand_steps = (net.igraph.node_count() as f64).log(2.0) as usize;

    let mut total_distance: u64 = 0;
    let mut cur_node = src_node;
    
    while cur_node != dst_node {
        let (mut new_cur_node, mut new_dist , _): (usize, u64, _) = 
            net.closest_nodes_structure(cur_node).take(amount_close)
                .min_by_key(|&(i, _, _)| node_dist(i)).unwrap();

        if node_dist(new_cur_node) >= node_dist(cur_node) {
            // Do some random walk:
            new_cur_node = cur_node;
            new_dist = 0;
            for _ in 0 .. rand_steps {
                let neighbors = net.igraph.neighbors(new_cur_node).into_iter().collect::<Vec<usize>>();
                let neighbor_range : Range<usize> = Range::new(0, neighbors.len());
                new_cur_node = neighbors[neighbor_range.ind_sample(&mut rng)];
                new_dist += 1;
            }

            /*

            // Pick a best local destination randomly in a "smart" way:
            let mut items = net.closest_nodes_structure(cur_node).take(amount_close*2)
                .map(|(i, dist, gateway)| 
                     Weighted { weight: calc_weight(i), item: (i, dist, gateway) })
                .collect::<Vec<_>>();

            // Pick the next step as the gateway of the chosen local destination:
            let wc = WeightedChoice::new(&mut items);
            let smp = wc.ind_sample(&mut rng);
            // gateway_index = smp.2;
            new_cur_node = smp.0;
            new_dist = smp.1;
            */
        }

        total_distance += new_dist;
        cur_node = new_cur_node;

    }
    // Wanted node was found!. We return it.
    total_distance
}




#[cfg(test)]
mod tests {
    use super::*;
    use random_util::choose_k_nums;
    use landmarks::coords::{build_coords, choose_landmarks, randomize_coord};
    use network::{random_net};
    use self::rand::{StdRng};

    #[test]
    fn test_find_path_landmarks() {
        let l = 5;
        let num_nodes: usize = ((2 as u64).pow(l)) as usize;
        let num_neighbours: usize = (1.5 * (num_nodes as f64).ln()) as usize;
        let num_landmarks: usize = (((l*l) as u32)) as usize;
        let amount_close = num_neighbours.pow(2);

        let seed: &[_] = &[1,2,3,4,5];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        // Creating the network:
        let net = random_net(num_nodes,num_neighbours,&mut rng);
        let landmarks = choose_landmarks(&net,num_landmarks, &mut rng);

        // Iterating through coordinates:
        // Make sure that the graph is connected:
        let coords = match build_coords(&net, &landmarks) {
            Some(coords) => coords,
            None => unreachable!(),
        };

        // Get a random node pair:
        let node_pair: Vec<usize> = choose_k_nums(2,net.igraph.node_count(),&mut rng)
                .into_iter().collect::<Vec<_>>();

        // Try to route from one of the nodes in the pair to the other:
        let _ = find_path_landmarks(node_pair[0], node_pair[1],
                            amount_close, &net, &coords, &landmarks, &mut rng).unwrap();
    }

    #[test]
    fn test_find_path_landmarks_by_coord() {
        let l = 5;
        let num_nodes: usize = ((2 as u64).pow(l)) as usize;
        let num_neighbours: usize = (1.5 * (num_nodes as f64).ln()) as usize;
        let num_landmarks: usize = (((l*l) as u32)) as usize;
        let amount_close = num_neighbours.pow(2);

        let seed: &[_] = &[1,2,3,4,5];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        // Creating the network:
        let net = random_net(num_nodes,num_neighbours,&mut rng);
        let landmarks = choose_landmarks(&net,num_landmarks, &mut rng);

        // Iterating through coordinates:
        // Make sure that the graph is connected:
        let coords = match build_coords(&net, &landmarks) {
            Some(coords) => coords,
            None => unreachable!(),
        };

        // Get a random node pair:
        let node_pair: Vec<usize> = choose_k_nums(2,net.igraph.node_count(),&mut rng)
                .into_iter().collect::<Vec<_>>();

        // Try to route from one of the nodes in the pair to the other:
        let _ = find_path_landmarks_by_coord(node_pair[0], &coords[node_pair[1]],5,
                            amount_close, &net, &coords, &landmarks, &mut rng);
    }

    #[test]
    fn test_find_path_landmarks_approx() {
        let l = 5;
        let num_nodes: usize = ((2 as u64).pow(l)) as usize;
        let num_neighbours: usize = (1.5 * (num_nodes as f64).ln()) as usize;
        let num_landmarks: usize = (((l*l) as u32)) as usize;
        let amount_close = num_neighbours.pow(2);

        let seed: &[_] = &[1,2,3,4,5];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        // Creating the network:
        let net = random_net(num_nodes,num_neighbours,&mut rng);
        let landmarks = choose_landmarks(&net,num_landmarks, &mut rng);

        // Iterating through coordinates:
        // Make sure that the graph is connected:
        let coords = match build_coords(&net, &landmarks) {
            Some(coords) => coords,
            None => unreachable!(),
        };

        // Get a random node pair:
        let node_pair: Vec<usize> = choose_k_nums(2,net.igraph.node_count(),&mut rng)
                .into_iter().collect::<Vec<_>>();

        // Try to route from one of the nodes in the pair to the other:
        let _ = find_path_landmarks_approx(node_pair[0], node_pair[1], &coords[node_pair[1]],
                            amount_close, &net, &coords, &landmarks, &mut rng);
    }

    #[test]
    fn test_randomize_coord() {
        let l = 5;
        let num_nodes: usize = ((2 as u64).pow(l)) as usize;
        let num_neighbours: usize = (1.5 * (num_nodes as f64).ln()) as usize;
        let num_landmarks: usize = (((l*l) as u32)) as usize;

        let seed: &[_] = &[1,2,3,4,5];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        // Creating the network:
        let net = random_net(num_nodes,num_neighbours,&mut rng);
        let landmarks = choose_landmarks(&net,num_landmarks, &mut rng);
        // Iterating through coordinates:
        // Make sure that the graph is connected:
        let coords = match build_coords(&net, &landmarks) {
            Some(coords) => coords,
            None => unreachable!(),
        };

        // Generate a random coordinate:
        let rand_coord = randomize_coord(&landmarks, &coords, &mut rng);
        assert!(rand_coord.len() == coords[0].len());
    }
}
