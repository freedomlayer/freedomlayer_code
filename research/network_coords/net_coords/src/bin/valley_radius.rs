#![cfg(not(test))]
extern crate net_coords;
extern crate rand;

use rand::{Rng, StdRng};

use net_coords::coord_mappers::{approx_max_dist, approx_avg_dist,
    approx_pairs_dist1, approx_pairs_dist1_normalized,
    approx_pairs_dist2, approx_pairs_dist2_normalized};
use net_coords::network::{Network, random_net};
use net_coords::coords::{build_coords, choose_landmarks};


use rand::distributions::{IndependentSample, Range};

/// Check if there are any local minima for network coordinates.
pub fn find_max_area(net: &Network<usize>, coords: &Vec<Vec<u64>>, landmarks: &Vec<usize>, 
         mut rng: &mut StdRng, iters: usize, node_sample: usize) {

    // Node distance function:
    let node_dist = |x,y| approx_max_dist(x,y,&coords, &landmarks);

    let mut max_area: u64 = 0;
    let mut sum_area: u64 = 0;

    for _ in 0 .. iters {
        let rand_range: Range<usize> = Range::new(0,net.igraph.node_count());
        let dst_node = rand_range.ind_sample(rng);

        for _ in 0 .. node_sample {
            // pick a random source node which is not the dest node:
            let mut src_node = dst_node;
            while src_node == dst_node {
                src_node = rand_range.ind_sample(rng);
            }

            let area: u64 = net.closest_nodes(src_node)
                .position(|(node_index, _, _)| 
                          node_dist(node_index, dst_node) < node_dist(src_node, dst_node))
                .unwrap() as u64;

            if max_area < area {
                max_area = area;
            }
            sum_area += area;
        }
    }

    println!("max_area = {}", max_area);
    println!("avg_area = {}", 
             (sum_area as f64) / ((iters * node_sample) as f64));
}

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
    let l: u32 = 11;
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

    println!("find_max_area:");
    find_max_area(&net, &coords, &landmarks, &mut (rng.clone()), 100,100);

}

