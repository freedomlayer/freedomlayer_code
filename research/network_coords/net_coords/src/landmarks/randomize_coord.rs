extern crate rand;

use self::rand::distributions::{IndependentSample, Range};
use landmarks::coord_mappers::dist_u64;
use self::rand::Rng;

/// Generate a random coordinate
pub fn randomize_coord_rand_coeffs<R: Rng>(landmarks: &Vec<usize>, coords: &Vec<Vec<u64>>,
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


/// Generate a random coordinate
/// This one was very good with the 2d network, but not very good with the
/// random network.
pub fn randomize_coord_landmarks_coords<R: Rng>(landmarks: &Vec<usize>, coords: &Vec<Vec<u64>>,
                    mut rng: &mut R) -> Vec<u64> {

    let rand_landmark: Range<usize> = 
        Range::new(0, landmarks.len());
    let interval_size: u64 = 2_u64.pow(0_u32);

    let mut coord: Vec<u64> = vec![];
    for _ in 0 .. landmarks.len() {
        let mut cur_value = 0;
        for _ in 0 .. interval_size {
            // TODO: Check if having:
            // cur_value += coords[landmarks[rand_landmark.ind_sample(&mut rng)]][i];
            // is better.
            cur_value += coords[landmarks[rand_landmark.ind_sample(&mut rng)]][rand_landmark.ind_sample(&mut rng)];
        }
        coord.push(cur_value);
    }
    coord
}

/// Generate a random coordinate
/// Linear cominbation of landmarks, undeflated.
pub fn randomize_coord_fair_cuts<R: Rng>(landmarks: &Vec<usize>, coords: &Vec<Vec<u64>>,
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

/// Generate a random coordinate
pub fn randomize_coord_cheat<R: Rng>(landmarks: &Vec<usize>, coords: &Vec<Vec<u64>>,
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

////////////////////////////////////////////////////////////

/// Find integral average of all the landmarks.
/// Returns a coordinate that is the average.
fn average_landmarks(landmarks: &Vec<usize>, coords: &Vec<Vec<u64>>) -> Vec<u64> {
    // Sum all landmarks:
    let mut comb_coord = vec![0_u64; landmarks.len()];
    for i in 0 .. landmarks.len() {
        for (j, &x) in coords[landmarks[i]].iter().enumerate() {
            comb_coord[j] += x;
        }
    }
    // Divide each entry by the amount of landmarks:
    for i in 0 .. landmarks.len() {
        comb_coord[i] /= landmarks.len() as u64;
    }

    comb_coord
}

/// Get possible range (min_val, max_val) for entry i in a given coordinate.
/// Inside this range, all the triangle inequalities are valid.
fn get_entry_legal_range(cur_coord: &Vec<u64>, i: usize, landmarks: &Vec<usize>, 
                   coords: &Vec<Vec<u64>>) -> (u64, u64) {

    // A function to calculate distance between landmark i and landmark j:
    let dl = |i: usize, j: usize| coords[landmarks[i]][j];

    let lower_bound: u64 = (0 .. landmarks.len())
        .map(|j| dist_u64(cur_coord[j],dl(i,j)))
        .max()
        .unwrap();

    let upper_bound: u64 = (0 .. landmarks.len())
        .map(|j| cur_coord[j] + dl(i,j))
        .min()
        .unwrap();

    (lower_bound, upper_bound)
}

/*
/// Get possible random walk range (min_val, max_val) for entry i in a given coordinate.
/// Inside this range, all the triangle inequalities are valid and some additional conditions are
/// satisfied.
fn get_entry_rw_range(cur_coord: &Vec<u64>, i: usize, landmarks: &Vec<usize>, 
                   coords: &Vec<Vec<u64>>) -> (u64, u64) {

    // A function to calculate distance between landmark i and landmark j:
    let dl = |i: usize, j: usize| coords[landmarks[i]][j];

    let (lower_bound, mut upper_bound) = get_entry_legal_range(&cur_coord, i, &landmarks, &coords);

    // The largest distance seen from landmark i.
    // We have this constraint so that we don't diverge
    // to inifinitely large numbers.
    let upper_constraint = (0 .. landmarks.len())
        .map(|j| 2*dl(i,j))
        .max()
        .unwrap();

    if upper_constraint < upper_bound {
        upper_bound = upper_constraint;
    }

    (lower_bound, upper_bound)
}
*/

/// Check if a coordinate satisfies all triangle inequalities
fn is_legal_coord(cur_coord: &Vec<u64>, landmarks: &Vec<usize>, 
                  coords: &Vec<Vec<u64>>) -> bool {

    for i in 0 .. landmarks.len() {
        let (low, high) = get_entry_legal_range(&cur_coord, i, &landmarks, &coords);
        let val = cur_coord[i];
        if val < low {
            return false;
        }
        if val > high {
            return false;
        }
    }
    true
}


/// Generate a random coordinate using a random walk
pub fn randomize_coord_rw<R: Rng>(landmarks: &Vec<usize>, coords: &Vec<Vec<u64>>,
                    mut rng: &mut R) -> Vec<u64> {

    let mut cur_coord = average_landmarks(&landmarks, &coords);
    assert!(is_legal_coord(&cur_coord, &landmarks, &coords));
    // let entry_range: Range<usize> = Range::new(0, landmarks.len());

    let diff_range: Range<i64> = Range::new(-100,101);


    let mut good_iters = 0;
    // Iterations of random walk:
    while good_iters < landmarks.len().pow(2) {
    // for _ in 0 .. landmarks.len().pow(2) {

        let new_coord = (0 .. landmarks.len())
            .map(|i| {
                // Make sure that the new value doesn't go below 0:
                let mut diff = diff_range.ind_sample(&mut rng);
                while (cur_coord[i] as i64) + diff < 0 {
                    diff = diff_range.ind_sample(&mut rng);
                }
                ((cur_coord[i] as i64) + diff) as u64
            })
            .collect::<Vec<u64>>();

        if is_legal_coord(&new_coord, &landmarks, &coords) {
            cur_coord = new_coord;
            good_iters += 1;
        }

        /*
        let i = entry_range.ind_sample(&mut rng);

        // Get range of valid values for entry number i:
        let (low, high) = get_entry_rw_range(&cur_coord, i, &landmarks, &coords);

        if low < high {
            good_iters += 1;
            println!("low = {}, high = {}",low,high);
        }

        // Set the new random value to the entry:
        let value_range: Range<u64> = Range::new(low, high + 1);
        cur_coord[i] = value_range.ind_sample(&mut rng);
        */
    }

    cur_coord
}
