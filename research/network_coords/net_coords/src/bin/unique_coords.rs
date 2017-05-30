#![cfg(not(test))]

extern crate rand;

extern crate net_coords;

use net_coords::network::{random_net};
use net_coords::landmarks::coords::{build_coords, choose_landmarks, is_coord_unique};

use rand::{StdRng};

#[cfg(not(test))]
fn main() {
    let l: u32 = 15;
    println!("--------------------------------");
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

    println!("Checking uniqueness of coords ...");
    let is_unique = is_coord_unique(&(coords.unwrap()));
    println!("is_unique = {}",is_unique);
}

