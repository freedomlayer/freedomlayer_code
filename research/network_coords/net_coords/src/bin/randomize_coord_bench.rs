#![cfg(not(test))]
extern crate net_coords;
extern crate rand;
extern crate ordered_float;

use rand::{StdRng};
// use std::hash::Hash;
use net_coords::landmarks::coords::{build_coords, choose_landmarks};
// use net_coords::landmarks::randomize_coord::randomize_coord_rw;
use net_coords::landmarks::randomize_coord::randomize_coord_rw_sparse;
// use net_coords::landmarks::randomize_coord::randomize_coord_rw_directional;
use net_coords::network_gen::{gen_network};


#[cfg(not(test))]
fn main() {
    println!("Randomize coord functions benchmark");
    
    let g = 9;
    let l = 2 * g + 1;
    let net_type = 0;
    print!("g={:2}; ",g);
    match net_type {
        0 => print!("rand    ; "),
        1 => print!("2d      ; "),
        2 => print!("rand+2d ; "),
        _ => unreachable!(),
    }
    // print!("nt={:1}; ",net_type);
    println!("Generating network...");
    /* Generate network */
    let seed: &[_] = &[1,g,net_type];
    let mut network_rng: StdRng = rand::SeedableRng::from_seed(seed);
    let net = gen_network(net_type, g, l, 1000, 2000, &mut network_rng);

    // Generate helper structures for landmarks routing:
    // Calculate landmarks and coordinates for landmarks routing:
    // Amount of landmarks can not be above half of the node count:
    let mut num_landmarks: usize = (((g*g) as u32)) as usize;
    // let mut num_landmarks: usize = 10; // DEBUG
    if num_landmarks as f64 > (net.igraph.node_count() as f64) / 2.0 {
        num_landmarks = net.igraph.node_count() / 2;
    }
    println!();
    println!("Choosing landmarks...");
    let landmarks = choose_landmarks(&net, num_landmarks, &mut network_rng);
    println!("Building coordinates...");
    let coords = match build_coords(&net, &landmarks) {
        Some(coords) => coords,
        None => unreachable!(),
    };

    // Randomize a coordinate:
    for i in 0 .. 1 {
        let _ = randomize_coord_rw_sparse(&landmarks, &coords, &mut network_rng);
        println!("i = {}",i);
    }

    println!();
}



