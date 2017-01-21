mod network;
mod coords;
mod coord_mappers;
mod random_util;
mod statistic;
mod checks;

use coord_mappers::{approx_max_dist, approx_avg_dist};

#[cfg(not(test))]
use checks::{check_unique_coord, check_approx_dist};

#[cfg(not(test))]
fn main() {
    // check_ring_nums(16);
    // check_unique_coord(16);
    // check_approx_dist(14, approx_max_dist);
    // check_approx_dist(15, approx_max_dist);
    // check_approx_dist(16, approx_max_dist);
    // check_approx_dist(17, approx_max_dist);


    println!("approx_max_dist");
    check_approx_dist(16,approx_avg_dist);
}

