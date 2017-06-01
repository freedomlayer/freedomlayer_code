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
use net_coords::network::{Network};
use net_coords::landmarks::coords::{build_coords, choose_landmarks};
use net_coords::random_util::choose_k_nums;

use net_coords::chord::network_gen::{random_net_chord, 
    random_net_and_grid2_chord, random_grid2_net_chord};

use net_coords::chord;
use net_coords::chord::{init_fingers, 
    converge_fingers, create_semi_chains,
    verify_global_optimality};

use net_coords::chord::{RingKey};
// use net_coords::chord::semi_chains_array::SemiChainsArray;



#[derive(Debug)]
struct RoutingStats {
    mean_route_length: f64,
    max_route_length: u64,
    success_ratio: f64,
}

/// Get routing stats for a a pair of functions:
/// - Randomize node pair
/// - Attempt to find a path between the two nodes
fn get_routing_stats<R: Rng>(rand_node_pair: &mut FnMut(&mut R) -> Vec<usize>,
                   find_path: &mut FnMut(usize, usize) -> Option<u64>, 
                   mut node_pair_rng: &mut R,
                   iters: usize) -> RoutingStats {
    // Amount of routing failures:
    let mut num_route_fails: usize = 0;
    // Sum of path length (Used for average later)
    let mut sum_route_length: u64 = 0;
    // Maximum route length:
    let mut max_route_length: u64 = 0;

    for _ in 0 .. iters {
        let node_pair: Vec<usize> = rand_node_pair(&mut node_pair_rng);

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


/// Generate a network according to given type.
/// g -- amount of nodes (logarithmic).
/// l -- maximum key space for chord based networks (logarithmic)
fn gen_network<R:Rng>(net_type: usize, g: usize,l: usize, mut rng: &mut R) -> Network<RingKey> {
    assert!(l >= 2*g, "Key collisions are too likely!");
    match net_type {
        0 => {
            // Random network.
            let num_nodes: usize = ((2 as u64).pow(g as u32)) as usize;
            let num_neighbors: usize = (1.5 * (num_nodes as f64).ln()) as usize;
            random_net_chord(num_nodes, num_neighbors, l, &mut rng)
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

/*
struct RoutePrecompute<'a, Node: 'a> {
    net: &'a Network<Node>,
    semi_chains: &'a Vec<SemiChainsArray>,
    coords: &'a Vec<Vec<u64>>,
    landmarks: &'a Vec<usize>
}
*/



#[cfg(not(test))]
fn main() {
    let chord_num_iters = 1000;
    let landmarks_num_iters = 100;
    println!("      Network        |          chord         |    landmarks nei^2     |     landmarks nei^3     ");
    println!("---------------------+------------------------+------------------------+------------------------+");

    for g in 6 .. 21 { // Iterate over size of network.
        let l = 2 * g + 1;
        for net_type in 0 .. 3 { // Iterate over type of network
            for net_iter in 0 .. 3 { // Three iterations for each type of network

                print!("g={:2}; ",g);
                match net_type {
                    0 => print!("rand    ; "),
                    1 => print!("2d      ; "),
                    2 => print!("rand+2d ; "),
                    _ => unreachable!(),
                }
                // print!("nt={:1}; ",net_type);
                /* Generate network */
                let seed: &[_] = &[1,2,3,4,5,g,net_type,net_iter];
                let mut network_rng: StdRng = rand::SeedableRng::from_seed(seed);
                let net = gen_network(net_type, g, l, &mut network_rng);
                let avg_degree = ((((2*net.igraph.edge_count()) as f64) / 
                    (net.igraph.node_count() as f64)) + 1.0) as usize;
                print!("ni={:1} |",net_iter);

                /* Chord routing  */
                let node_pair_rng_seed: &[_] = &[1,2,3,4,5,9,g,net_type,net_iter];
                let mut node_pair_rng: StdRng = rand::SeedableRng::from_seed(
                    node_pair_rng_seed);
                let mut rand_node_pair = |mut rng: &mut StdRng| {
                    let mut node_pair = choose_k_nums(2,net.igraph.node_count(),
                            &mut rng).into_iter().collect::<Vec<usize>>();
                    // Sort for determinism:
                    node_pair.sort();
                    node_pair
                };
                let mut fingers = init_fingers(&net,l, &mut network_rng);
                converge_fingers(&net, &mut fingers,l);
                assert!(verify_global_optimality(&net, &fingers));
                let semi_chains = create_semi_chains(&net, &fingers);
                let mut find_path = |src_i: usize, dst_i: usize| {
                    let src_id = net.index_to_node(src_i).unwrap().clone();
                    let dst_id = net.index_to_node(dst_i).unwrap().clone();
                    chord::find_path(src_id, dst_id, &net, &semi_chains)
                        .map(|x| x as u64)
                };

                let routing_stats = 
                    get_routing_stats(&mut rand_node_pair, &mut find_path,
                                      &mut node_pair_rng, chord_num_iters);

                print!("{:9.2}, {:6}, {:02.2} |", 
                       routing_stats.mean_route_length,
                       routing_stats.max_route_length,
                       routing_stats.success_ratio);

                // Calculate landmarks and coordinates for landmarks routing:
                // Amount of landmarks can not be above half of the node count:
                let mut num_landmarks: usize = (((l*l) as u32)) as usize;
                if num_landmarks as f64 > (net.igraph.node_count() as f64) / 2.0 {
                    num_landmarks = net.igraph.node_count() / 2;
                }
                let landmarks = choose_landmarks(&net, num_landmarks, &mut network_rng);
                let coords = match build_coords(&net, &landmarks) {
                    Some(coords) => coords,
                    None => unreachable!(),
                };

                /* Landmarks routing nei^2  */

                let mut find_path = |src_i: usize, dst_i: usize| {
                    let random_walk_rng_seed: &[_] = &[6,8,3,4,5,g,net_type,net_iter];
                    let mut random_walk_rng: StdRng = rand::SeedableRng::from_seed(
                        random_walk_rng_seed);
                    let amount_close = avg_degree.pow(2);
                    find_path_landmarks(src_i, dst_i,
                            amount_close, &net, &coords, &landmarks, &mut random_walk_rng)
                };

                node_pair_rng = rand::SeedableRng::from_seed(node_pair_rng_seed);
                let routing_stats = 
                    get_routing_stats(&mut rand_node_pair, &mut find_path,
                                      &mut node_pair_rng, landmarks_num_iters);
                print!("{:9.2}, {:6}, {:02.2} |", 
                       routing_stats.mean_route_length,
                       routing_stats.max_route_length,
                       routing_stats.success_ratio);


                /* Landmarks routing nei^3  */

                let mut find_path = |src_i: usize, dst_i: usize| {
                    let random_walk_rng_seed: &[_] = &[6,8,3,4,5,g,net_type,net_iter];
                    let mut random_walk_rng: StdRng = rand::SeedableRng::from_seed(
                        random_walk_rng_seed);
                    let amount_close = avg_degree.pow(3);
                    find_path_landmarks(src_i, dst_i,
                            amount_close, &net, &coords, &landmarks, &mut random_walk_rng)
                };

                node_pair_rng = rand::SeedableRng::from_seed(node_pair_rng_seed);
                let routing_stats = 
                    get_routing_stats(&mut rand_node_pair, &mut find_path,
                                      &mut node_pair_rng, landmarks_num_iters);
                print!("{:9.2}, {:6}, {:02.2} |", 
                       routing_stats.mean_route_length,
                       routing_stats.max_route_length,
                       routing_stats.success_ratio);

                println!();
            }
        }
        println!();
    }
}


