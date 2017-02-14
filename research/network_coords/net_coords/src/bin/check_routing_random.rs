extern crate net_coords;
extern crate rand;
extern crate ordered_float;

use rand::{StdRng};
use rand::distributions::{IndependentSample, Range};

use net_coords::coord_mappers::{approx_max_dist, approx_avg_dist,
    approx_pairs_dist1, approx_pairs_dist1_normalized,
    approx_pairs_dist2, approx_pairs_dist2_normalized};
use net_coords::network::{Network, random_net};
use net_coords::coords::{build_coords, choose_landmarks};
use net_coords::random_util::choose_k_nums;

use self::ordered_float::OrderedFloat;


/// Try to find a path in the network between src_node and dst_node.
/// Using a variation of random walk.
/// Returns None if path was not found, or Some(path_length)
fn try_route_random(src_node: usize, dst_node: usize, 
         amount_close: usize, net: &Network<usize>, 
         coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>,
         mut rng: &mut StdRng) -> Option<u64> {

    // Node distance function:
    let node_dist = |x,y| approx_max_dist(x,y,&coords, &landmarks);
    let mut num_hops = 0;

    let mut cur_node = src_node;

    // println!("------------------------");
    // println!("Routing from {} to {}",src_node, dst_node); 
    
    while cur_node != dst_node {
        let closest_nodes: Vec<(usize, u64)> = net.closest_nodes(cur_node)
            .take(amount_close)
            .collect::<Vec<_>>();

        let &(mut new_cur_node, mut new_dist): &(usize, u64) = closest_nodes.iter()
            .min_by_key(|&&(i, dist)| OrderedFloat(node_dist(dst_node, i))).unwrap();

        while new_cur_node == cur_node {
            let rand_range: Range<usize> = Range::new(0,closest_nodes.len());
            let tup = closest_nodes[rand_range.ind_sample(rng)];
            new_cur_node = tup.0;
            new_dist = tup.1;
        }
        num_hops += new_dist;
        cur_node = new_cur_node;
    }
    Some(num_hops)
}

///
/// Check the success rate of routing in the network.
/// amount_close is the amount of close nodes every node keeps.
/// iters is the amount of iterations for this check.
pub fn check_routing_random(net: &Network<usize>, coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>, 
         mut rng: &mut StdRng, amount_close: usize, iters: usize) {

    // Amount of routing failures:
    let mut num_route_fails: usize = 0;
    // Sum of path length (Used for average later)
    let mut sum_route_length: u64 = 0;

    for _ in 0 .. iters {
        let node_pair: Vec<usize> = choose_k_nums(2,net.igraph.node_count(),&mut rng)
            .into_iter().collect::<Vec<_>>();

        let num_hops = try_route_random(node_pair[0], node_pair[1],
                            amount_close, &net, &coords, &landmarks, &mut rng);

        match num_hops {
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
    // check_ring_nums(16);
    // check_unique_coord(16);
    // check_approx_dist(14, approx_max_dist);
    // check_approx_dist(15, approx_max_dist);
    // check_approx_dist(16, approx_max_dist);
    // check_approx_dist(17, approx_max_dist);
    
    // Set up graph parameters:
    // let l: u32 = 16;
    //
    let l: u32 = 12;
    let n: usize = ((2 as u64).pow(l)) as usize;
    let num_neighbours: usize = (1.5 * (n as f64).ln()) as usize;
    let num_landmarks: usize = (((l*l) as u32)/3) as usize;

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

    // TODO: Possibly feed all check_approx_dist calls with the same list of pairs of nodes.
    // Currently each one generates a different set of pairs, which might affect the results.
    
    check_routing_random(&net, &coords, &landmarks, &mut (rng.clone()), 
                  num_neighbours.pow(3), 1000);

    /*
    println!("approx_max_dist");
    check_approx_dist(l*l,approx_max_dist, &net, &coords, &landmarks, &mut (rng.clone()));
    println!("approx_avg_dist");
    check_approx_dist(l*l,approx_avg_dist, &net, &coords, &landmarks, &mut (rng.clone()));
    println!("approx_pairs_dist1");
    check_approx_dist(l*l,approx_pairs_dist1,&net, &coords, &landmarks, &mut (rng.clone()));
    println!("approx_pairs_dist1_normalized");
    check_approx_dist(l*l,approx_pairs_dist1_normalized,&net, &coords, &landmarks, &mut (rng.clone()));
    println!("approx_pairs_dist2");
    check_approx_dist(l*l,approx_pairs_dist2,&net, &coords, &landmarks, &mut (rng.clone()));
    println!("approx_pairs_dist2_normalized");
    check_approx_dist(l*l,approx_pairs_dist2_normalized,&net, &coords, &landmarks, &mut (rng.clone()));
    */
}

