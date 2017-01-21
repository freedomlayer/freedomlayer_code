mod network;
mod coords;
mod coord_mappers;
mod random_util;
mod statistic;
mod checks;

#[cfg(not(test))]
use checks::check_unique_coord;

#[cfg(not(test))]
fn main() {
    // check_ring_nums();
    check_unique_coord(16);
}

