#![cfg(not(test))]
extern crate net_coords;
extern crate rand;
extern crate ordered_float;

use rand::{StdRng};
// use std::hash::Hash;
use net_coords::landmarks::coords::{build_coords, choose_landmarks,
    randomize_coord};
use net_coords::landmarks::coord_mappers::{max_dist};
use net_coords::network_gen::{gen_network};


/*
 * An experiment to find out how balanced is our method to find
 * a randomly looking coordinate in the network.
 */

#[cfg(not(test))]
fn main() {
    let net_types = 3;
    let net_iters = 3;
    // We generate num_nodes * iter_mult random coordinates:
    let iter_mult = 3;

    println!("iter_mult = {}", iter_mult);
    println!();

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
                let seed: &[_] = &[1,g,net_type,net_iter];
                let mut network_rng: StdRng = rand::SeedableRng::from_seed(seed);
                let net = gen_network(net_type, g, l, &mut network_rng);
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

                let mut node_repeats: Vec<usize> = vec![0; net.igraph.node_count()];
                for _ in 0 .. net.igraph.node_count() * iter_mult {
                    let rcoord = randomize_coord(&landmarks, &coords, &mut network_rng);
                    let (closest_node_index, _) = coords.iter().enumerate()
                        .min_by_key(|&(i,coord)| (max_dist(&rcoord, &coord),i)).unwrap();

                    // println!("rcoord = {:?}", rcoord);
                    // println!("closest_node_index = {}", closest_node_index);
                    node_repeats[closest_node_index] += 1;
                }

                // println!("-----------------------");

                let max_node_repeats: usize = node_repeats.iter().max().unwrap().clone();
                print!("max_nr = {}",max_node_repeats);


                println!();
            }
        }
        println!();
    }
}


