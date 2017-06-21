#![cfg(not(test))]
extern crate net_coords;
extern crate rand;
extern crate ordered_float;

use self::rand::{StdRng};

// use std::hash::Hash;
use net_coords::landmarks::coords::{build_coords, choose_landmarks};
use net_coords::landmarks::{find_path_landmarks_areas_approx, 
    find_path_landmarks_areas, gen_areas};
use net_coords::network_gen::{gen_network};
use net_coords::random_util::choose_k_nums;
use net_coords::landmarks::randomize_coord::{drift_coordinate};

use std::collections::HashSet;


/*
 * An experiment to see if one node can find another node
 * even after his coordinates drifted a bit.
 */


#[cfg(not(test))]
fn main() {
    let net_types = 4;
    let net_iters = 2;
    // We generate num_nodes * iter_mult random coordinates:
    let num_pairs = 100;
    let max_visits = 2;
    let drift_size = 0x8000;

    println!("Try to find a node given drifted coordinates");
    println!();
    println!("max_visits = {}", max_visits);
    println!("num_pairs = {}", num_pairs);
    println!("drift_size = {}", drift_size);
    println!();

    for g in 8 .. 20 { // Iterate over size of network.
        let l = 2 * g + 1;
        for net_type in 0 .. net_types { // Iterate over type of network
            for net_iter in 0 .. net_iters { // Three iterations for each type of network
                print!("g={:2}; ",g);
                match net_type {
                    0 => print!("rand    ; "),
                    1 => print!("2d      ; "),
                    2 => print!("rand+2d ; "),
                    3 => print!("planar  ; "),
                    _ => unreachable!(),
                }
                // print!("nt={:1}; ",net_type);
                /* Generate network */
                let seed: &[_] = &[1,g,net_type,net_iter];
                let mut network_rng: StdRng = rand::SeedableRng::from_seed(seed);
                // let net = gen_network(net_type, g, l, 0x10000, 0x20000 , &mut network_rng);
                // let net = gen_network(net_type, g, l, 0x10000, 0x20000 , &mut network_rng);
                let net = gen_network(net_type, g, l, 0x10000, 0x20000 , &mut network_rng);
                print!("ni={:1} |",net_iter);

                // let avg_degree = ((((2*net.igraph.edge_count()) as f64) / 
                //     (net.igraph.node_count() as f64)) + 1.0) as usize;
                // let amount_close = avg_degree.pow(2);
                let amount_close = ((net.igraph.node_count() as f64).log(2.0) as usize).pow(2);

                // Generate helper structures for landmarks routing:
                // Calculate landmarks and coordinates for landmarks routing:
                // Amount of landmarks can not be above half of the node count:
                let mut num_landmarks: usize = (((g*g) as u32)) as usize;
                // let mut num_landmarks: usize = 10; // DEBUG
                if num_landmarks as f64 > (net.igraph.node_count() as f64) / 2.0 {
                    num_landmarks = net.igraph.node_count() / 2;
                }
                let areas = gen_areas(amount_close, &net);
                let landmarks = choose_landmarks(&net, num_landmarks, &mut network_rng);
                let coords = match build_coords(&net, &landmarks) {
                    Some(coords) => coords,
                    None => unreachable!(),
                };
                // let upper_constraints = calc_upper_constraints(&landmarks, &coords);

                let mut pair_rng: StdRng = 
                    rand::SeedableRng::from_seed(&[2,g, net_type, net_iter] as &[_]);
                let mut coord_rng: StdRng = 
                    rand::SeedableRng::from_seed(&[3,g, net_type, net_iter] as &[_]);
                let mut route_rng: StdRng = 
                    rand::SeedableRng::from_seed(&[4,g, net_type, net_iter] as &[_]);

                let mut num_paths_found: usize = 0;
                let mut sum_path_len: u64 = 0;

                for _ in 0 .. num_pairs {
                    // Randomize a pair of nodes.
                    let mut node_pair = choose_k_nums(2,net.igraph.node_count(),
                            &mut pair_rng).into_iter().collect::<Vec<usize>>();
                    // Sort for determinism:
                    node_pair.sort();

                    // First make sure that node_pair[0] can find node_pair[1].
                    assert!(find_path_landmarks_areas(node_pair[0], node_pair[1], &net, &coords, &landmarks, 
                                              &areas, &mut route_rng).is_some());

                    // Drift node_pair[1]'s coordinate:
                    let mut drifted_coord = coords[node_pair[1]].clone();
                    drift_coordinate(drift_size, &mut drifted_coord, &mut coord_rng);

                    let mut hs = HashSet::new();
                    hs.insert(node_pair[1]);

                    let opt_path_len = 
                        find_path_landmarks_areas_approx(node_pair[0], &hs, &drifted_coord,
                                   net.igraph.node_count() as u64, &net, 
                                   &coords, &landmarks, &areas, &mut route_rng);

                    if let Some(path_len) = opt_path_len {
                        sum_path_len += path_len;
                        num_paths_found += 1;
                    } else {
                        // println!();
                        // println!("rcoord = {:?}", rcoord);
                        // println!();
                    }
                }


                let avg_path_len = (sum_path_len as f64) / (num_paths_found as f64);
                print!("avg_path_len = {:8.3} |",avg_path_len);
                let found_ratio = (num_paths_found as f64) / (num_pairs as f64);
                print!("found_ratio = {:5.3} |",found_ratio);


                println!();
            }
        }
        println!();
    }
}



