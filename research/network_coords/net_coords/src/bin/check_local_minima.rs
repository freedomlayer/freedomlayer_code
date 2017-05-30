#![cfg(not(test))]
extern crate net_coords;
extern crate rand;

use rand::{StdRng};

/*
use net_coords::coord_mappers::{approx_max_dist, approx_avg_dist,
    approx_pairs_dist1, approx_pairs_dist1_normalized,
    approx_pairs_dist2, approx_pairs_dist2_normalized};
    */
use net_coords::landmarks::coord_mappers::{approx_max_dist};
use net_coords::network::{Network, random_net};
use net_coords::landmarks::coords::{build_coords, choose_landmarks};


use rand::distributions::{IndependentSample, Range};

/// Check if there are any local minima for network coordinates.
pub fn check_local_minima(net: &Network<usize>, coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>, 
         mut rng: &mut StdRng, amount_close: usize, iters: usize) {

    // Node distance function:
    let node_dist = |x,y| approx_max_dist(x,y,&coords, &landmarks);

    let mut sum_ratio: f64 = 0.0;

    for _ in 0 .. iters {
        let rand_range: Range<usize> = Range::new(0,net.igraph.node_count());
        let dst_node = rand_range.ind_sample(rng);

        let mut num_not_minimum = 0;

        for src_node in 0 .. net.igraph.node_count() {
            if src_node == dst_node {
                continue
            }
            let found_better: bool = net.closest_nodes(src_node)
                .take(amount_close)
                .any(|(i, _, _)| node_dist(i, dst_node) < node_dist(src_node, dst_node));

            if found_better {
                num_not_minimum += 1;
            }
        }

        let ratio_not_minimum = 
            (num_not_minimum as f64) / ((net.igraph.node_count() - 1) as f64);

        sum_ratio += ratio_not_minimum;
    }

    println!("success_ratio = {}", sum_ratio / (iters as f64));
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
    let l: u32 = 20;
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
    
    /*
    println!("check_routing_random:");
    check_routing_random(&net, &coords, &landmarks, &mut (rng.clone()), 
                  num_neighbours.pow(3), 1000);
    */

    println!("check_local_minima:");
    check_local_minima(&net, &coords, &landmarks, &mut (rng.clone()), 
                  num_neighbours.pow(3), 100);

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

