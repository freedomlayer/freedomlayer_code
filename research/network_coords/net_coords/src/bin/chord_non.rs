#![cfg(not(test))]

extern crate net_coords;
extern crate rand;
extern crate ordered_float;

use rand::{StdRng};
use net_coords::chord::{random_net_chord, init_fingers, 
    converge_fingers, create_semi_chains, find_path,
    verify_global_optimality};
use net_coords::random_util::choose_k_nums;



#[cfg(not(test))]
fn main() {
    let pair_iters = 1000;
    for g in 11 .. 21 {
        // Keyspace size:
        let l: usize = (2 * g + 1)  as usize;

        println!("--------------------------------");
        let num_nodes: usize = ((2 as u64).pow(g)) as usize;
        let num_neighbours: usize = (1.5 * (num_nodes as f64).ln()) as usize;

        // let num_neighbours: usize = 3;

        println!("g = {}", g);
        println!("num_nodes = {}",num_nodes);
        println!("num_neighbours = {}",num_neighbours);

        let seed: &[_] = &[1,2,3,4,5];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        println!("Creating the network...");
        let net = random_net_chord(num_nodes, num_neighbours, l, &mut rng);
        println!("Initializing chord fingers...");
        let mut fingers = init_fingers(&net, l, &mut rng);
        println!("Converge chord fingers...");
        converge_fingers(&net, &mut fingers, l);
        println!("Verifying global optimality...");
        assert!(verify_global_optimality(&net, &fingers));
        println!("Creating semi routes...");
        let semi_chains = create_semi_chains(&net, &fingers);

        println!("Finding average length of path...");
        // Find average length of path:
        let mut sum_length: u64 = 0;
        for _ in 0 .. pair_iters {
            let mut node_pair: Vec<usize> = choose_k_nums(2,net.igraph.node_count(),&mut rng)
                .into_iter().collect::<Vec<_>>();
            node_pair.sort(); // Make computation deterministic
            let src_id = net.index_to_node(node_pair[0]).unwrap().clone();
            let dst_id = net.index_to_node(node_pair[1]).unwrap().clone();

            let path_len = find_path(src_id, dst_id, &net, &semi_chains).unwrap();
            sum_length += path_len as u64;

        }

        println!("Average length of path: {}", (sum_length as f64) / (pair_iters as f64));


    }
}

