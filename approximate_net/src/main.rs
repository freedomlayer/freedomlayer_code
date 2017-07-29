extern crate rand;
extern crate approximate_net;

use self::rand::{StdRng};
use approximate_net::{
    eval_approx_size_funcs};

use approximate_net::approx_funcs::{APPROX_FUNCS_NAMED, ApproxFunc};

fn main() {
    let seed: &[_] = &[1,2,3,4,5,6];
    let mut rng: StdRng = rand::SeedableRng::from_seed(seed);

    let num_iters = 100;
    let num_mins = 40;
    let num_elems = 1000000;
    println!("Calculating error ratios for approximation functions...");
    println!("num_iters = {}",num_iters);
    println!("num_mins  = {}",num_mins);
    println!("num_elems = {}",num_elems);
    println!();

    let approx_funcs = APPROX_FUNCS_NAMED.iter()
        .map(|&(approx_func, _)| approx_func)
        .collect::<Vec<&ApproxFunc>>();

    let err_ratios = eval_approx_size_funcs(num_iters, 
                                            num_mins, 
                                            num_elems, 
                                            approx_funcs.as_slice(),
                                            &mut rng);

    println!("err_ratio for approximation functions:");
    for (i,&(_, func_name)) in APPROX_FUNCS_NAMED.iter().enumerate() {
        println!("{:30} : {}", func_name, err_ratios[i]);

    }

}
