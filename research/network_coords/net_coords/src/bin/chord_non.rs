extern crate net_coords;
extern crate rand;
extern crate ordered_float;

use rand::{StdRng};
use net_coords::network::{Network, random_net};
use net_coords::chord::{random_net_chord, converge_fingers};


const FINGERS_SEED: usize = 0x1337;


#[cfg(not(test))]
fn main() {
    for l in 11 .. 21 {
    // let l: u32 = 15;
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
        let net = random_net_chord(n,num_neighbours,&mut rng);

        let fingers = init_chord_fingers(&net);
        converge_fingers(&net, &mut fingers, FINGERS_SEED);

    }
}

