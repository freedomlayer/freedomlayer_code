#![cfg(not(test))]
extern crate net_coords;
extern crate rand;
extern crate ordered_float;

use rand::{StdRng};
// use std::hash::Hash;
use net_coords::landmarks::coords::{build_coords, choose_landmarks};
use net_coords::landmarks::randomize_coord::{randomize_coord_landmarks_coords};
use net_coords::landmarks::{find_path_landmarks_approx, find_path_landmarks_by_coord};
use net_coords::network_gen::{gen_network};
use net_coords::random_util::choose_k_nums;


/*
 * An experiment to see if one node can find another node's kept
 * coordinates, somewhere in the network.
 */

#[cfg(not(test))]
fn main() {
    let net_types = 3;
    let net_iters = 3;
    // We generate num_nodes * iter_mult random coordinates:
    let num_pairs = 100;
    let max_visits = 3;

    println!("Find ratio of matches for approximate finding of a random coordinate");
    println!("from two different sources.");
    println!();
    println!("max_visits = {}", max_visits);
    println!("num_pairs = {}", num_pairs);
    println!();

    for g in 6 .. 20 { // Iterate over size of network.
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
                let seed: &[_] = &[1,g,net_type,net_iter];
                let mut network_rng: StdRng = rand::SeedableRng::from_seed(seed);
                let net = gen_network(net_type, g, l, 10000, 20000 , &mut network_rng);
                print!("ni={:1} |",net_iter);

                // Generate helper structures for landmarks routing:
                // Calculate landmarks and coordinates for landmarks routing:
                // Amount of landmarks can not be above half of the node count:
                let mut num_landmarks: usize = (((l*l) as u32)) as usize;
                // let mut num_landmarks: usize = 10; // DEBUG
                if num_landmarks as f64 > (net.igraph.node_count() as f64) / 2.0 {
                    num_landmarks = net.igraph.node_count() / 2;
                }
                let landmarks = choose_landmarks(&net, num_landmarks, &mut network_rng);
                let coords = match build_coords(&net, &landmarks) {
                    Some(coords) => coords,
                    None => unreachable!(),
                };
                let avg_degree = ((((2*net.igraph.edge_count()) as f64) / 
                    (net.igraph.node_count() as f64)) + 1.0) as usize;
                let amount_close = avg_degree.pow(2);

                let mut pair_rng: StdRng = 
                    rand::SeedableRng::from_seed(&[2,g, net_type, net_iter] as &[_]);
                let mut coord_rng: StdRng = 
                    rand::SeedableRng::from_seed(&[3,g, net_type, net_iter] as &[_]);
                let mut route_rng: StdRng = 
                    rand::SeedableRng::from_seed(&[4,g, net_type, net_iter] as &[_]);

                let mut num_paths_found = 0;
                let mut sum_path_len = 0;
                let mut sum_num_attempts = 0;

                for _ in 0 .. num_pairs {
                    // Randomize a pair of nodes.
                    let mut node_pair = choose_k_nums(2,net.igraph.node_count(),
                            &mut pair_rng).into_iter().collect::<Vec<usize>>();
                    // Sort for determinism:
                    node_pair.sort();

                    // Randomize a coordinate (randomize_coord)
                    let rcoord = randomize_coord_landmarks_coords(&landmarks, &coords, &mut coord_rng);

                    let (found_node_i, _) =  
                        find_path_landmarks_by_coord(node_pair[0], &rcoord,
                                   amount_close, max_visits, &net, 
                                   &coords, &landmarks, &mut route_rng);

                    let mut found = false;
                    let mut num_attempts = 0;
                    while !found {
                        num_attempts += 1;
                        // First go to a random place in the network:
                        let my_rcoord = randomize_coord_landmarks_coords(&landmarks, &coords, &mut coord_rng);
                        let (my_node_i, first_part_len) = 
                            find_path_landmarks_by_coord(node_pair[1], &my_rcoord,
                                       amount_close, max_visits, &net, 
                                       &coords, &landmarks, &mut route_rng);
                        // Starting from the random place in the network, try to find
                        // the wanted coordinate:
                        let opt_path_len = 
                            find_path_landmarks_approx(my_node_i, found_node_i, &rcoord,
                                       (g as u64).pow(3), amount_close, &net, 
                                       &coords, &landmarks, &mut route_rng);

                        if let Some(path_len) = opt_path_len {
                            sum_path_len += path_len + first_part_len;
                            num_paths_found += 1;
                            found = true;
                        } else {
                            // println!();
                            // println!("rcoord = {:?}", rcoord);
                            // println!();
                        }
                    }
                    sum_num_attempts += num_attempts;
                }


                let avg_path_len = (sum_path_len as f64) / (num_paths_found as f64);
                print!("avg_path_len = {:6.3} |",avg_path_len);
                let avg_num_attempts = (sum_num_attempts as f64) / (num_pairs as f64);
                print!("avg_num_attempts = {:4.3} |",avg_num_attempts);


                println!();
            }
        }
        println!();
    }
}


