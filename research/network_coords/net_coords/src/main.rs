extern crate rand;

mod network;
mod coords;
mod coord_mappers;
mod random_util;
mod statistic;
mod checks;

use rand::{StdRng};

use coord_mappers::{approx_max_dist, approx_avg_dist,
    approx_pairs_dist1, approx_pairs_dist1_normalized,
    approx_pairs_dist2, approx_pairs_dist2_normalized};
use network::{random_net};
use coords::{build_coords, choose_landmarks};

#[cfg(not(test))]
use checks::{check_unique_coord, check_approx_dist};

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
    let l: u32 = 10;
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
}

