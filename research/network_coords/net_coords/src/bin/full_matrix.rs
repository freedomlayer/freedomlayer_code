/* Check various combinations of routing schemes with
 * different network layouts. Print the results 
 * in a nice table
 */

#![cfg(not(test))]
extern crate net_coords;
extern crate rand;
extern crate ordered_float;

use rand::{Rng, StdRng};
// use std::hash::Hash;

use net_coords::landmarks::find_path_landmarks;
use net_coords::network::{Network, grid2_net};
use net_coords::landmarks::coords::{build_coords, choose_landmarks};
use net_coords::random_util::choose_k_nums;

use net_coords::chord::network_gen::{random_net_chord, 
    random_net_and_grid2_chord, random_grid2_net_chord};

use net_coords::chord::{init_fingers, 
    converge_fingers, create_semi_chains, find_path,
    verify_global_optimality};

use net_coords::chord::{RingKey};
use net_coords::chord::semi_chains_array::SemiChainsArray;

struct RoutingStats {
    mean_route_length: f64,
    max_route_length: u64,
    success_ratio: f64,
}

/*
/// Check the success rate of routing in the network using landmarks routing.
/// amount_close is the amount of close nodes every node keeps.
/// iters is the amount of iterations for this check.
fn landmarks_routing_check<Node:Hash + Eq + Clone, R:Rng>(net: &Network<Node>, 
          coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>, 
         amount_close: usize, iters: usize, mut rng: &mut R) -> RoutingStats {

    // Amount of routing failures:
    let mut num_route_fails: usize = 0;
    // Sum of path length (Used for average later)
    let mut sum_route_length: u64 = 0;
    // Maximum route length:
    let mut max_route_length: u64 = 0;

    for _ in 0 .. iters {
        let node_pair: Vec<usize> = choose_k_nums(2,net.igraph.node_count(),&mut rng)
            .into_iter().collect::<Vec<_>>();

        let total_distance = find_path_landmarks(node_pair[0], node_pair[1],
                            amount_close, &net, &coords, &landmarks, &mut rng);

        match total_distance {
            Some(route_length) => {
                sum_route_length += route_length;
                if route_length > max_route_length {
                    max_route_length = route_length;
                }
            },
            None => num_route_fails += 1,
        };
    }

    let num_route_success = iters - num_route_fails;
    let mean_route_length = (sum_route_length as f64) / (num_route_success as f64);

    let success_ratio = (num_route_success as f64) / (iters as f64);

    RoutingStats {
        mean_route_length,
        max_route_length,
        success_ratio,
    }
}
*/

/// Get routing stats for a a pair of functions:
/// - Randomize node pair
/// - Attempt to find a path between the two nodes
fn get_routing_stats<R: Rng>(rand_node_pair: &mut FnMut() -> Vec<usize>,
                   find_path: &Fn(usize, usize) -> Option<u64>, iters: usize) -> RoutingStats {
    // Amount of routing failures:
    let mut num_route_fails: usize = 0;
    // Sum of path length (Used for average later)
    let mut sum_route_length: u64 = 0;
    // Maximum route length:
    let mut max_route_length: u64 = 0;

    for _ in 0 .. iters {
        let node_pair: Vec<usize> = rand_node_pair();

        match find_path(node_pair[0], node_pair[1]) {
            Some(route_length) => {
                sum_route_length += route_length;
                if route_length > max_route_length {
                    max_route_length = route_length;
                }
            },
            None => {num_route_fails += 1;},
        };
    }

    let num_route_success = iters - num_route_fails;
    let mean_route_length = (sum_route_length as f64) / (num_route_success as f64);
    let success_ratio = (num_route_success as f64) / (iters as f64);

    RoutingStats {
        mean_route_length,
        max_route_length,
        success_ratio,
    }

}

/*
fn chord_routing_check<R: Rng>(net: &Network<RingKey>, 
                   semi_chains: &Vec<SemiChainsArray>, iters: usize, mut rng: &mut R) {

    // Find average length of path:
    let mut sum_route_length: u64 = 0;
    for _ in 0 .. iters {
        let mut node_pair: Vec<usize> = choose_k_nums(2,net.igraph.node_count(),&mut rng)
            .into_iter().collect::<Vec<_>>();
        node_pair.sort(); // Make computation deterministic
        let src_id = net.index_to_node(node_pair[0]).unwrap().clone();
        let dst_id = net.index_to_node(node_pair[1]).unwrap().clone();

        match find_path(src_id, dst_id, &net, &semi_chains) {
            Some(route_length) => {
                sum_route_length += route_length;
                if route_length > max_route_length {
                    max_route_length = route_length;
                }
            }
        }
        sum_length += path_len as u64;
}
*/

fn gen_network<R:Rng>(net_type: usize, g: usize, mut rng: &mut R) -> Network<RingKey> {
    match net_type {
        0 => {
            // Random network.
            let num_nodes: usize = ((2 as u64).pow(g as u32)) as usize;
            let num_neighbors: usize = (1.5 * (num_nodes as f64).ln()) as usize;
            random_net_chord(num_nodes, num_neighbors, g, &mut rng)
        }
        1 => {
            // 2d grid with random chord ids
            let num_nodes: usize = ((2 as u64).pow(g as u32)) as usize;
            let l: usize = (2 * g + 1)  as usize;
            let k = (num_nodes as f64).sqrt() as usize;
            random_grid2_net_chord(k, l, &mut rng)
        }
        2 => {
            // 2d grid combined with random network
            let num_nodes: usize = ((2 as u64).pow(g as u32)) as usize;
            let l: usize = (2 * g + 1)  as usize;
            let k = (num_nodes as f64).sqrt() as usize;
            let num_neighbors: usize = (1.5 * (num_nodes as f64).ln()) as usize;
            random_net_and_grid2_chord(k, num_neighbors, l, &mut rng)
        }
        _ => unreachable!()

    }

}



#[cfg(not(test))]
fn main() {
    for g in 6 .. 21 { // Iterate over size of network.
        for net_type in 0 .. 3 { // Iterate over type of network
            for net_iter in 0 .. 3 { // Three iterations for each type of network

                let seed: &[_] = &[1,2,3,4,5,g,net_type,net_iter];
                let mut network_rng: StdRng = rand::SeedableRng::from_seed(seed);
                let net = gen_network(net_type, g, &mut network_rng);

                let seed: &[_] = &[1,2,3,4,5,g,net_type,net_iter];
                let mut node_pair_rng: StdRng = rand::SeedableRng::from_seed(seed);
                let rand_node_pair = || {
                    let mut node_pair = choose_k_nums(2,net.igraph.node_count(),&mut node_pair_rng)
                        .into_iter().collect::<Vec<usize>>();
                    // Sort for determinism:
                    node_pair.sort();
                    node_pair
                };

                /*

                println!("Creating 2d grid network...");
                let k = (num_nodes as f64).sqrt() as usize;
                println!("k = {}", k);
                let net = grid2_net(k);
                let landmarks = choose_landmarks(&net,num_landmarks, &mut rng);
                println!("Iterating through coordinates");
                let coords = build_coords(&net, &landmarks);

                if coords.is_none() {
                    println!("graph is not connected! Aborting.");
                    return
                }

                let coords = coords.unwrap();
                
                println!("weighted_routing...");
                check_weighted_routing(&net, &coords, &landmarks, &mut (rng.clone()), 
                              num_neighbors.pow(2), 100);
                */
            }
        }
    }
}


