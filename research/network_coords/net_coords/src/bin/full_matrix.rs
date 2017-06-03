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
use net_coords::chord::semi_chains_array::SemiChainsArray;

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

fn run_routing_by_type<R: Rng>(routing_type: usize, net: &Network<RingKey>,
        chord_num_iters: usize, landmarks_num_iters: usize,
        coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>,
        semi_chains: &Vec<SemiChainsArray>, avg_degree: usize,
        mut node_pair_rng: &mut R, mut routing_rng: &mut R) -> RoutingStats {

    // A function to pick a random node pair from the network,
    // based on a given rng:
    let mut rand_node_pair = |mut rng: &mut R| {
        let mut node_pair = choose_k_nums(2,net.igraph.node_count(),
                &mut rng).into_iter().collect::<Vec<usize>>();
        // Sort for determinism:
        node_pair.sort();
        node_pair
    };
    match routing_type {
        0 => { /* chord routing */
            let mut find_path = |src_i: usize, dst_i: usize| {
                let src_id = net.index_to_node(src_i).unwrap().clone();
                let dst_id = net.index_to_node(dst_i).unwrap().clone();
                chord::find_path(src_id, dst_id, &net, &semi_chains)
                    .map(|x| x as u64)
            };

            get_routing_stats(&mut rand_node_pair, &mut find_path,
                                  &mut node_pair_rng, chord_num_iters)
        },
        1 => { /* landmarks routing nei^2 */
            let mut find_path = |src_i: usize, dst_i: usize| {
                let amount_close = avg_degree.pow(2);
                find_path_landmarks(src_i, dst_i,
                        amount_close, &net, &coords, &landmarks, &mut routing_rng)
            };

            get_routing_stats(&mut rand_node_pair, &mut find_path,
                                  &mut node_pair_rng, landmarks_num_iters)
        },
        2 => { /* landmarks routing nei^3 */
            let mut find_path = |src_i: usize, dst_i: usize| {
                let amount_close = avg_degree.pow(3);
                find_path_landmarks(src_i, dst_i,
                        amount_close, &net, &coords, &landmarks, &mut routing_rng)
            };

            get_routing_stats(&mut rand_node_pair, &mut find_path,
                              &mut node_pair_rng, landmarks_num_iters)
        },
        _ => unreachable!(),
    }
}


#[cfg(not(test))]
fn main() {
    let chord_num_iters = 1000;
    let landmarks_num_iters = 100;
    let net_types = 3;
    let net_iters = 3;
    let routing_types = 3;
    // Keep the last max route length for combinations of [net_type][routing_type]
    let mut last_max_route_lengths: Vec<Vec<u64>> =
        (0 .. net_types).map(|_| (0 .. routing_types).map(|_| 0).collect::<Vec<_>>())
            .collect::<Vec<Vec<_>>>();
    // max_route_length should not pass this value (Which is too slow for routing).
    // If it does, next time we are not going to try to route with the same net_type
    // and routing_type
    let allowed_max_route_length = 500000;

    println!("      Network        |          chord         |    landmarks nei^2     |     landmarks nei^3     ");
    println!("---------------------+------------------------+------------------------+------------------------+");

    for g in 6 .. 21 { // Iterate over size of network.
        let l = 2 * g + 1;

        for net_type in 0 .. net_types { // Iterate over type of network
            for net_iter in 0 .. net_iters { // Three iterations for each type of network
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

                // Prepare rand_node_pair:
                let node_pair_rng_seed: &[_] = &[1,g,net_type,net_iter];
                let base_node_pair_rng: StdRng = rand::SeedableRng::from_seed(
                    node_pair_rng_seed);

                // Prepare routing rng:
                let routing_rng_seed: &[_] = &[2,g,net_type,net_iter];
                let base_routing_rng: StdRng = rand::SeedableRng::from_seed(
                    routing_rng_seed);

                // Generate helper structures for chord routing:
                let mut fingers = init_fingers(&net,l, &mut network_rng);
                converge_fingers(&net, &mut fingers,l);
                assert!(verify_global_optimality(&net, &fingers));
                let semi_chains = create_semi_chains(&net, &fingers);

                // Generate helper structures for landmarks routing:

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

                for routing_type in 0 .. routing_types { // Routing type

                    if last_max_route_lengths[net_type][routing_type] >
                        allowed_max_route_length {
                            print!("**********************|");
                            continue
                    }

                    // Duplicate the random state, so that each routing attempt will
                    // have the same random to begin with.
                    let mut node_pair_rng = base_node_pair_rng.clone();
                    let mut routing_rng = base_routing_rng.clone();

                    let routing_stats = run_routing_by_type(routing_type,
                        &net, chord_num_iters, landmarks_num_iters,
                        &coords, &landmarks, &semi_chains, avg_degree,
                        &mut node_pair_rng, &mut routing_rng);

                    // Update last max route_length:
                    last_max_route_lengths[net_type][routing_type] = 
                        routing_stats.max_route_length;

                    // Print routing statistics:
                    print!("{:9.2}, {:6}, {:02.2} |", 
                           routing_stats.mean_route_length,
                           routing_stats.max_route_length,
                           routing_stats.success_ratio);


                } // routing type iteration
                println!();
            }
        }
        println!();
    }
}


