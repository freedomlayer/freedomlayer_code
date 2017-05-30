pub mod coords;
pub mod coord_mappers;

extern crate rand;

use self::rand::{Rng};
use self::rand::distributions::{Weighted, WeightedChoice, 
    IndependentSample};

use network::{Network};
use landmarks::coord_mappers::{approx_max_dist};

/// Try to find a path in the network between src_node and dst_node.
/// Using a variation of random walk.
/// Returns None if path was not found, or Some(path_length)
pub fn find_path_landmarks<R: Rng>(src_node: usize, dst_node: usize, 
         amount_close: usize, net: &Network<usize>, 
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
            net.closest_nodes(cur_node).take(amount_close)
                .min_by_key(|&(i, _, _)| node_dist(dst_node, i)).unwrap();

        if node_dist(new_cur_node, dst_node) >= node_dist(cur_node, dst_node) {

            // Pick a best local destination randomly in a "smart" way:
            let mut items = net.closest_nodes(cur_node).take(amount_close)
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
