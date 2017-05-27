#![cfg(not(test))]
extern crate net_coords;
extern crate rand;
extern crate ordered_float;

use rand::{StdRng};
use rand::distributions::{Weighted, WeightedChoice, 
    IndependentSample, Range};

use net_coords::coord_mappers::{approx_max_dist, approx_avg_dist,
    approx_pairs_dist1, approx_pairs_dist1_normalized,
    approx_pairs_dist2, approx_pairs_dist2_normalized};
use net_coords::network::{Network, random_net, random_net_weighted};
use net_coords::coords::{build_coords, choose_landmarks};
use net_coords::random_util::choose_k_nums;


/// Try to find a path in the network between src_node and dst_node.
/// Using a variation of random walk.
/// Returns None if path was not found, or Some(path_length)
fn try_route_weighted_random(src_node: usize, dst_node: usize, 
         amount_close: usize, net: &Network<usize>, 
         coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>,
         mut rng: &mut StdRng) -> Option<u64> {

    // Node distance function:
    let node_dist = |x,y| approx_max_dist(x,y,&coords, &landmarks);
    // let calc_weight = |i: usize| ((-(node_dist(i, dst_node) as f64)).exp() * 100.0) as u32;
    let calc_weight = |_: usize| 1 as u32;

    let mut total_distance = 0;
    let mut cur_node = src_node;

    // println!("------------------------");
    // println!("Routing from {} to {}",src_node, dst_node); 
    
    while cur_node != dst_node {
        // println!("dst_node = {}. cur_node = {}", dst_node, cur_node);
        
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
        cur_node = new_cur_node;
        // cur_node = gateway_index;

    }
    Some(total_distance)
}

///
/// Check the success rate of routing in the network.
/// amount_close is the amount of close nodes every node keeps.
/// iters is the amount of iterations for this check.
pub fn check_weighted_routing(net: &Network<usize>, coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>, 
         mut rng: &mut StdRng, amount_close: usize, iters: usize) {

    // Amount of routing failures:
    let mut num_route_fails: usize = 0;
    // Sum of path length (Used for average later)
    let mut sum_route_length: u64 = 0;

    for _ in 0 .. iters {
        let node_pair: Vec<usize> = choose_k_nums(2,net.igraph.node_count(),&mut rng)
            .into_iter().collect::<Vec<_>>();

        let total_distance = try_route_weighted_random(node_pair[0], node_pair[1],
                            amount_close, &net, &coords, &landmarks, &mut rng);

        match total_distance {
            Some(num) => sum_route_length += num,
            None => num_route_fails += 1,
        };
    }

    let num_route_success = iters - num_route_fails;
    let mean_route_length = (sum_route_length as f64) / (num_route_success as f64);

    let success_ratio = (num_route_success as f64) / (iters as f64);

    println!("success_ratio = {}", success_ratio);
    println!("mean_route_length = {}", mean_route_length);
}

#[cfg(not(test))]
fn main() {
    for l in 11 .. 22 {
    // let l: u32 = 15;
        println!("--------------------------------");
        let n: usize = ((2 as u64).pow(l)) as usize;
        let num_neighbours: usize = (1.5 * (n as f64).ln()) as usize;
        let num_landmarks: usize = (((l*l) as u32)) as usize;
        // let num_landmarks: usize = (((l*l) as u32)) as usize;

        println!("n = {}",n);
        println!("num_neighbours = {}",num_neighbours);
        println!("num_landmarks = {}",num_landmarks);

        let seed: &[_] = &[1,2,3,4,5];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        println!("Creating the network...");
        let net = random_net(n,num_neighbours,&mut rng);
        let landmarks = choose_landmarks(&net,num_landmarks, &mut rng);
        println!("Iterating through coordinates");
        let coords = build_coords(&net, &landmarks);

        if coords.is_none() {
            println!("graph is not connected! Aborting.");
            return
        }

        let coords = coords.unwrap();
        
        println!("weighted_routing direct on weighted network...");
        check_weighted_routing(&net, &coords, &landmarks, &mut (rng.clone()), 
                      num_neighbours.pow(3), 100);
    }
}

