extern crate rand;

mod network_sim;
use network_sim::Network;


#[cfg(not(test))]
fn check_unique_coord() {
    let mut rng = rand::thread_rng();
    let l: u32 = 20;
    let n: usize = ((2 as u64).pow(l)) as usize;
    let num_neighbours: usize = (l/2) as usize;
    let num_landmarks: usize = (2*l) as usize;

    let mut net = Network::new()
        .build_network(n,num_neighbours,&mut rng)
        .choose_landmarks(num_landmarks,&mut rng);

    for i in 0..7 {
        net.iter_coords();
        println!("Iter number {}",i);
    }

    
    let is_unique = net.is_coord_unique();
    println!("is_unique = {}",is_unique);

    // net.print_some_coords(10);

}



#[cfg(not(test))]
fn main() {
    // let net = Network::new();
    // let mut rng = rand::thread_rng();
    check_unique_coord();
}
