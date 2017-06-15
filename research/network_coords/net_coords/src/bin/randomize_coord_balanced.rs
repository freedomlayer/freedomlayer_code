#![cfg(not(test))]
extern crate net_coords;
extern crate rand;
extern crate ordered_float;

use rand::{StdRng};
// use std::hash::Hash;
use net_coords::landmarks::coords::{build_coords, choose_landmarks};
// use net_coords::landmarks::randomize_coord::{/*randomize_coord_landmarks_coords,*/ randomize_coord_rw_sparse};
use net_coords::landmarks::randomize_coord::randomize_coord_rw_directional;
use net_coords::landmarks::randomize_coord::calc_upper_constraints;
use net_coords::landmarks::coord_mappers::{max_dist};
use net_coords::network_gen::{gen_network};
use self::rand::distributions::{IndependentSample, Range};


/*
 * An experiment to find out how balanced is our method to find
 * a randomly looking coordinate in the network.
 */

#[cfg(not(test))]
fn main() {
    let net_types = 4;
    let net_iters = 2;
    // We generate num_nodes * iter_mult random coordinates:
    let iter_mult = 1;

    println!("iter_mult = {}", iter_mult);
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
                    3 => print!("planar  ; "),
                    _ => unreachable!(),
                }
                // print!("nt={:1}; ",net_type);
                /* Generate network */
                let seed: &[_] = &[1,g,net_type,net_iter];
                let mut network_rng: StdRng = rand::SeedableRng::from_seed(seed);
                let net = gen_network(net_type, g, l, 0x10000, 0x20000, &mut network_rng);
                print!("ni={:1} |",net_iter);

                // Generate helper structures for landmarks routing:
                // Calculate landmarks and coordinates for landmarks routing:
                // Amount of landmarks can not be above half of the node count:
                let mut num_landmarks: usize = (((g*g) as u32)) as usize;
                // let mut num_landmarks: usize = 10; // DEBUG
                if num_landmarks as f64 > (net.igraph.node_count() as f64) / 2.0 {
                    num_landmarks = net.igraph.node_count() / 2;
                }
                let landmarks = choose_landmarks(&net, num_landmarks, &mut network_rng);
                let coords = match build_coords(&net, &landmarks) {
                    Some(coords) => coords,
                    None => unreachable!(),
                };
                let upper_constraints = calc_upper_constraints(&landmarks, &coords);

                let choose_seed: &[_] = &[2,g,net_type,net_iter];
                let mut choose_rng: StdRng = rand::SeedableRng::from_seed(choose_seed);

                let interval_size = 2_u64.pow(0_u32);
                let inflate_coord = |coord: &Vec<u64>| coord.iter()
                    .map(|&c| c * interval_size)
                    .collect::<Vec<u64>>();

                let mut node_repeats: Vec<usize> = vec![0; net.igraph.node_count()];
                let mut sum_min_indices = 0;
                for _ in 0 .. net.igraph.node_count() * iter_mult {
                    // let rcoord = randomize_coord_landmarks_coords(&landmarks, &coords, &mut network_rng);
                    let rcoord = randomize_coord_rw_directional(&upper_constraints, 
                                                                &landmarks, &coords, &mut network_rng);
                    let min_value = coords.iter().enumerate()
                        .map(|(_,coord)| max_dist(&rcoord,&inflate_coord(&coord)))
                        .min().unwrap();

                    let mut min_indices = vec![];
                    for (i,coord) in coords.iter().enumerate() {
                        if max_dist(&rcoord, &inflate_coord(coord)) == min_value {
                            min_indices.push(i);
                        }
                    }
                    sum_min_indices += min_indices.len();
                    let choice_range: Range<usize> = Range::new(0, min_indices.len());
                    let closest_node_index = min_indices[choice_range.ind_sample(&mut choose_rng)];

                    // println!("rcoord = {:?}", rcoord);
                    /*
                    println!("----------------------------------------------------");
                    println!("closest_node_index = {}", closest_node_index);
                    println!("coord:");
                    println!("{:?}", coords[closest_node_index]);

                    println!("inflate_rcoord:");
                    println!("{:?}",inflate_rcoord);
                    */

                    node_repeats[closest_node_index] += 1;
                }

                // println!("-----------------------");

                let max_node_repeats: usize = node_repeats.iter().max().unwrap().clone();
                print!("max_nr = {:4}",max_node_repeats);
                print!("| average_min_indices = {}",(sum_min_indices as f64) / ((net.igraph.node_count() * iter_mult) as f64));


                println!();
            }
        }
        println!();
    }
}


