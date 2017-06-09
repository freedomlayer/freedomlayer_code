extern crate rand;

use std::collections::HashSet;

use self::rand::Rng;

use network::{Network};
use random_util::choose_k_nums;

use self::rand::distributions::{IndependentSample, Range};


pub fn choose_landmarks<R: Rng, Node> 
    (net: &Network<Node>, num_landmarks: usize, rng: &mut R) 
    -> Vec<usize> {
    choose_k_nums(num_landmarks, net.igraph.node_count(), rng)
        .into_iter().collect()
}


fn iter_coords<Node>(net: &Network<Node>, work_coords: &mut Vec<Vec<Option<u64>>>) -> bool {
    let mut has_changed = false;
    for v in net.igraph.nodes() {
        for (v,nei,&weight) in net.igraph.edges(v) {
            for c in 0..work_coords[nei].len() {
                let dist = work_coords[nei][c];
                if dist.is_none() {
                    continue
                }
                let cdist = dist.unwrap() + weight;
                if work_coords[v][c].is_none() {
                    work_coords[v][c] = Some(cdist);
                    has_changed = true;
                    continue
                }
                if work_coords[v][c].unwrap() > cdist {
                    work_coords[v][c] = Some(cdist);
                    has_changed = true;
                }
            }
        }
    }
    has_changed
}

/// Every node asks neighbours about distance to landmarks and 
/// updates his own distances accordingly.
/// Returns true if anything in the coords state has changed.
pub fn build_coords<Node>(net: &Network<Node>, landmarks:&Vec<usize>) -> Option<Vec<Vec<u64>>> 
{

    let mut work_coords: Vec<Vec<Option<u64>>> = Vec::new();

    // Initialize coordinates:
    for v in net.igraph.nodes() {
        let mut v_coords = Vec::new();
        for &l in landmarks.iter() {
            if v != l {
                v_coords.push(None)
            } else {
                v_coords.push(Some(0))
            }
        }
        work_coords.push(v_coords);
    }

    // println!("");
    let mut has_changed = true;
    while has_changed {
        has_changed = iter_coords(net, &mut work_coords);
        // println!("Iter");
        // print!(".");
    }
    // println!("");

    let is_disconnected: bool = 
        work_coords.iter().any(|coord| 
               coord.iter().any(|&c_opt| c_opt.is_none()));

    if is_disconnected {
        return None;
    }

    Some(work_coords.into_iter().map(|coord_opt| 
            coord_opt.into_iter().map(|c_opt| c_opt.unwrap()).collect::<Vec<_>>())
            .collect::<Vec<_>>())
    
}




/// Check if the coordinates system is unique
pub fn is_coord_unique(coords: &Vec<Vec<u64>>) -> bool {
    let mut coord_set = HashSet::new();
    for coord in coords.iter() {
        if coord_set.contains(coord) {
            return false
        }
        coord_set.insert(coord);
    }
    true
}


/*
/// Print some coordinates
pub fn print_some_coords(&self,amount: u32) {

    println!("coord_to_ring_all_pairs:");
    println!("{}", coord_to_ring(&self.coords[0 as usize]));
    println!("-------------");
    for &nei in self.neighbours[0].iter() {
        println!("{}", coord_to_ring(&self.coords[nei as usize]));
        // println!("{:?}", self.coords[nei as usize]);
    }
}
*/


/*
/// Generate a random coordinate
pub fn randomize_coord2<R: Rng>(landmarks: &Vec<usize>, coords: &Vec<Vec<u64>>,
                    mut rng: &mut R) -> Vec<u64> {

    // Generate random 16 bit integer coefficients:
    /*
    let rand_range: Range<u64> = 
        Range::new(0, 2_u64.pow(16_u32));
    */

    /*
    let int_coeffs: Vec<u64> = (0 .. landmarks.len())
        .map(|_| rand_range.ind_sample(&mut rng))
        .collect::<Vec<u64>>();
    */

    let mut int_coeffs: Vec<u64> = vec![0; landmarks.len()];
    let rand_range = Range::new(0,2);
    let mut coeffs_sum: u64 = 0;
    while coeffs_sum == 0 {
        for i in 0 .. landmarks.len() {
            int_coeffs[i] = rand_range.ind_sample(&mut rng);
        }

        coeffs_sum = int_coeffs.iter().sum::<u64>();
    }

    // Calculate linear combination of landmarks coordinates
    // according to coefficients:
    let mut comb_coord = vec![0_u64; landmarks.len()];
    for i in 0 .. landmarks.len() {
        for (j, &x) in coords[landmarks[i]].iter().enumerate() {
            comb_coord[j] += int_coeffs[i] * x;
        }
    }

    // Return normalized values for the coordinate:
    comb_coord.iter()
        .map(|&x| (x + (coeffs_sum / 2)) / coeffs_sum)
        .collect::<Vec<u64>>()

}

*/


/// Generate a random coordinate
/// This one was very good with the 2d network, but not very good with the
/// random network.
pub fn randomize_coord<R: Rng>(landmarks: &Vec<usize>, coords: &Vec<Vec<u64>>,
                    mut rng: &mut R) -> Vec<u64> {

    let rand_landmark: Range<usize> = 
        Range::new(0, landmarks.len());
    let interval_size: u64 = 2_u64.pow(0_u32);

    let mut coord: Vec<u64> = vec![];
    for i in 0 .. landmarks.len() {
        let mut cur_value = 0;
        for _ in 0 .. interval_size {
            // TODO: Check if having:
            cur_value += coords[landmarks[rand_landmark.ind_sample(&mut rng)]][i];
            // is better.
            // cur_value += coords[landmarks[rand_landmark.ind_sample(&mut rng)]][rand_landmark.ind_sample(&mut rng)];
        }
        coord.push(cur_value);
    }
    coord
}

/*
/// Generate a random coordinate
/// Linear cominbation of landmarks, undeflated.
pub fn randomize_coord<R: Rng>(landmarks: &Vec<usize>, coords: &Vec<Vec<u64>>,
                    mut rng: &mut R) -> Vec<u64> {

    // Generate random 16 bit integer coefficients:
    let interval_size: u64 = 2_u64.pow(20_u32);
    let interval_range: Range<u64> = Range::new(0, interval_size + 1);

    let mut cuts: Vec<u64> = Vec::new();
    cuts.push(0);
    for _ in 0 .. (landmarks.len() - 1) {
        cuts.push(interval_range.ind_sample(&mut rng));
    }
    cuts.push(interval_size);
    cuts.sort();

    let int_coeffs = (0 .. cuts.len() - 1)
        .map(|i| cuts[i+1] - cuts[i])
        .collect::<Vec<u64>>();

    // Some verifications for the int_coeffs:
    for &c in &int_coeffs {
        assert!(c <= interval_size);
    }
    assert!(int_coeffs.iter().sum::<u64>() == interval_size);


    // println!("int_coeffs:");
    // println!("{:?}", int_coeffs);

    // Calculate linear combination of landmarks coordinates
    // according to coefficients:
    // let mut total_sum = 0;
    let mut comb_coord = vec![0_u64; landmarks.len()];
    for i in 0 .. landmarks.len() {
        let mult = int_coeffs[i];
        for (j, &x) in coords[landmarks[i]].iter().enumerate() {
            comb_coord[j] += mult * x;
        }
        // total_sum += mult;
    }

    // Return normalized values for the coordinate:
    // comb_coord
    comb_coord.iter()
        .map(|&x| (x + (interval_size / 2)) / interval_size)
        .collect::<Vec<u64>>()
}
*/

/*
/// Generate a random coordinate
pub fn randomize_coord<R: Rng>(landmarks: &Vec<usize>, coords: &Vec<Vec<u64>>,
                    mut rng: &mut R) -> Vec<u64> {

    let _ = landmarks;
    // let landmarks_range: Range<usize> = Range::new(0, landmarks.len());
    // let coords_range: Range<usize> = Range::new(0, coords.len());
    let some_range: Range<usize> = Range::new(0, 15);

    // coords[coords_range.ind_sample(&mut rng)].clone()

    (0 .. landmarks.len())
        .map(|i| coords[some_range.ind_sample(&mut rng)][i])
        .collect::<Vec<u64>>()

}
*/



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashset_vec() {
        let mut my_set : HashSet<Vec<usize>> = HashSet::new();
        my_set.insert(vec![1,2,3]);
        assert!(my_set.contains(&vec![1,2,3]));
        assert!(!my_set.contains(&vec![1,2,4]));
    }
}
