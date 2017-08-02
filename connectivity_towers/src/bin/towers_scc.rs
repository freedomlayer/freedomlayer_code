/*
 * Check if the local towers connections form a 
 * strongly connected directed overlay graph in various networks.
 */

#![cfg(not(test))]
extern crate net_coords;
extern crate rand;
extern crate ordered_float;

use rand::{StdRng};

use net_coords::network_gen::{gen_network};
use net_coords::towers::{choose_towers, 
                         calc_towers_info, 
                         is_connected,
                         is_towers_info_filled};


#[cfg(not(test))]
fn main() {
    let net_types = 5;
    let net_iters = 2;
    let experiment_seed = 0x1337;

    println!("Checking if local towers overlay graph is strongly connected");
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
                    3 => print!("planar  ; "),
                    4 => print!("tree    ; "),
                    _ => unreachable!(),
                }
                print!("ni={:1} |",net_iter);

                /* Generate network */
                let seed: &[_] = &[experiment_seed,1,g,net_type,net_iter];
                let mut network_rng: StdRng = rand::SeedableRng::from_seed(seed);
                let net = gen_network(net_type, g, l, 1, 2, &mut network_rng);
                // Makes sure that the resulting network is connected:
                assert!(net.is_connected());

                // Choose towers:
                let seed: &[_] = &[experiment_seed,2,g,net_type,net_iter];
                let mut towers_rng: StdRng = rand::SeedableRng::from_seed(seed);
                let num_colors = g*2;
                let total_num_towers = (2_u64.pow(g as u32) as f64).sqrt() as usize;
                let num_towers = 1 + (total_num_towers / num_colors);
                let chosen_towers = choose_towers(&net, num_towers, num_colors, &mut towers_rng);
                // let chosen_towers = choose_towers(&net, 1, num_colors, &mut towers_rng); // Sanity check
                let towers_info = calc_towers_info(&net, &chosen_towers);
                // Make sure that towers_info are valid:
                assert!(is_towers_info_filled(&towers_info));

                print!("num_colors = {:5} |", num_colors);
                print!("num_towers = {:5} |", num_towers);

                let (connected, sconnected) = is_connected(&chosen_towers, &towers_info);
                print!(" connected = ");
                if connected {
                    print!("V");
                } else {
                    print!("X");
                }

                print!(" | sconnected = ");
                if sconnected {
                    print!("V");
                } else {
                    print!("X");
                }

                println!();
            }
        }
        println!();
    }
}



