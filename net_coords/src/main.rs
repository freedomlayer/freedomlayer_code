extern crate rand;

mod network_sim;
use network_sim::{Network,coord_to_ring, coord_to_ring_all_pairs};




#[cfg(not(test))]
fn check_unique_coord() {
    let mut rng = rand::thread_rng();
    let l: u32 = 16;
    let n: usize = ((2 as u64).pow(l)) as usize;
    let num_neighbours: usize = (1.5 * (n as f64).ln()) as usize;
    let num_landmarks: usize = (((l*l) as u32)/3) as usize;
    // let num_landmarks: usize = (5*l) as usize;

    println!("n = {}",n);
    println!("num_neighbours = {}",num_neighbours);
    println!("num_landmarks = {}",num_landmarks);

    let mut net = Network::new()
        .build_network(n,num_neighbours,&mut rng)
        .choose_landmarks(num_landmarks,&mut rng);

    if !net.converge_coords() {
        println!("Resulting graph is not connected!");
        return
    }
    
    let is_unique = net.is_coord_unique();
    println!("is_unique = {}",is_unique);

    net.print_some_coords(20);


}

fn check_ring_nums() {
    /*
    println!("{}",vec_to_ring(vec![1,2,3,4,5]));
    println!("{}",vec_to_ring(vec![5,2,3,4,5]));
    println!("{}",vec_to_ring(vec![5,2,4,4,5]));
    println!("{}",vec_to_ring(vec![5,2,4,8,5]));
    println!("{}",vec_to_ring(vec![5,2,5,8,5]));
    println!("{}",vec_to_ring(vec![5,3,5,9,6]));
    println!("{}",vec_to_ring(vec![6,4,6,10,7]));

    println!("{}",vec_to_ring(vec![1,2,3]));
    println!("{}",vec_to_ring(vec![2,3,1]));
    println!("{}",vec_to_ring(vec![3,1,2]));

    println!("-----");
    println!("{}",coord_to_ring(&vec![3,3,3,2]));
    println!("{}",coord_to_ring(&vec![2,3,3,3]));
    println!("{}",coord_to_ring(&vec![3,2,3,3]));
    println!("{}",coord_to_ring(&vec![3,3,2,3]));

    println!("-----");
    */

    println!("{}",coord_to_ring(&vec![1,2,3]));
    println!("{}",coord_to_ring(&vec![2,3,1]));
    println!("{}",coord_to_ring(&vec![3,1,2]));

    println!("----------");

    println!("{}",coord_to_ring(&vec![5,2,4,8,5,9,4,1,1,5,8,7,3]));
    println!("{}",coord_to_ring(&vec![5,3,4,8,5,9,4,1,1,5,8,7,3]));
    println!("{}",coord_to_ring(&vec![5,4,4,8,5,9,4,1,1,5,8,7,3]));
    println!("{}",coord_to_ring(&vec![5,4,4,8,5,8,3,1,1,5,8,7,3]));
    println!("{}",coord_to_ring(&vec![5,4,4,8,5,8,3,1,1,5,8,55,3]));
    println!("{}",coord_to_ring(&vec![5,3,4,8,5,8,3,1,1,5,8,55,3]));
    println!("{}",coord_to_ring(&vec![5,3,4,8,5,3,3,1,1,5,8,55,3]));
    println!("{}",coord_to_ring(&vec![5,3,4,8,5,3,3,2,1,5,8,55,3]));
    println!("{}",coord_to_ring(&vec![5,4,4,8,6,2,3,2,1,5,7,55,2]));
}


#[cfg(not(test))]
fn main() {
    // check_ring_nums();
    check_unique_coord();
}
