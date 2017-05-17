extern crate net_coords;
extern crate rand;
extern crate ordered_float;

use rand::{StdRng};
use net_coords::chord::{random_net_chord, init_chord_fingers, 
    converge_fingers, find_path};
use net_coords::random_util::choose_k_nums;


const FINGERS_SEED: usize = 0x1337;


#[cfg(not(test))]
fn main() {
    let pair_iters = 100;
    for g in 13 .. 16 {
        // Keyspace size:
        let l: usize = (2 * g + 1)  as usize;

        println!("--------------------------------");
        let num_nodes: usize = ((2 as u64).pow(g)) as usize;
        // let num_neighbours: usize = (1.5 * (num_nodes as f64).ln()) as usize;

        let num_neighbours: usize = 3;

        println!("num_nodes = {}",num_nodes);
        println!("num_neighbours = {}",num_neighbours);

        let seed: &[_] = &[1,2,3,4,5];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        println!("Creating the network...");
        let net = random_net_chord(num_nodes, num_neighbours, l, &mut rng);
        println!("Initializing chord fingers...");
        let mut fingers = init_chord_fingers(&net, l);
        println!("Converge chord fingers...");
        converge_fingers(&net, &mut fingers, FINGERS_SEED, l);


        println!("Finding average length of path...");
        // Find average length of path:
        let mut sum_length: u64 = 0;
        for _ in 0 .. pair_iters {
            let node_pair: Vec<usize> = choose_k_nums(2,net.igraph.node_count(),&mut rng)
                .into_iter().collect::<Vec<_>>();
            let src_id = net.index_to_node(node_pair[0]).unwrap().clone();
            let dst_id = net.index_to_node(node_pair[1]).unwrap().clone();

            let path = find_path(src_id, dst_id, &net, &fingers, l).unwrap();
            sum_length += path.len() as u64;
        }

        println!("Average length of path: {}", (sum_length as f64) / (pair_iters as f64));


    }
}

