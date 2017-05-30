#![cfg(not(test))]
extern crate net_coords;
extern crate rand;
extern crate ordered_float;

use rand::{StdRng};

use net_coords::landmarks::find_path_landmarks;
use net_coords::network::{Network, random_net};
use net_coords::landmarks::coords::{build_coords, choose_landmarks};
use net_coords::random_util::choose_k_nums;


///
/// Check the success rate of routing in the network.
/// amount_close is the amount of close nodes every node keeps.
/// iters is the amount of iterations for this check.
pub fn check_weighted_routing(net: &Network<usize>, coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>, 
         mut rng: &mut StdRng, amount_close: usize, iters: usize) {

    // Amount of routing failures:
    let mut num_route_fails: usize = 0;
    // Sum of path length (Used for average later)
    let mut sum_route_length: u64 = 0;

    for _ in 0 .. iters {
        let node_pair: Vec<usize> = choose_k_nums(2,net.igraph.node_count(),&mut rng)
            .into_iter().collect::<Vec<_>>();

        let total_distance = find_path_landmarks(node_pair[0], node_pair[1],
                            amount_close, &net, &coords, &landmarks, &mut rng);

        match total_distance {
            Some(num) => sum_route_length += num,
            None => num_route_fails += 1,
        };
    }

    let num_route_success = iters - num_route_fails;
    let mean_route_length = (sum_route_length as f64) / (num_route_success as f64);

    let success_ratio = (num_route_success as f64) / (iters as f64);

    println!("success_ratio = {}", success_ratio);
    println!("mean_route_length = {}", mean_route_length);
}

#[cfg(not(test))]
fn main() {
    println!("Landmarks routing in a random graph, using lookahead");
    for l in 11 .. 21 {
    // let l: u32 = 15;
        println!("--------------------------------");
        let num_nodes: usize = ((2 as u64).pow(l)) as usize;
        let num_neighbours: usize = (1.5 * (num_nodes as f64).ln()) as usize;
        let num_landmarks: usize = (((l*l) as u32)/3) as usize;

        println!("l = {}",l);
        println!("num_nodes = {}",num_nodes);
        println!("num_neighbours = {}",num_neighbours);
        println!("num_landmarks = {}",num_landmarks);

        let seed: &[_] = &[1,2,3,4,5];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        println!("Creating the network...");
        let net = random_net(num_nodes, num_neighbours,&mut rng);
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
                      num_neighbours.pow(2), 100);
    }
}

