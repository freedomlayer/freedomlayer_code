extern crate rand;
extern crate ordered_float;

// A trait alias for the distance function:
pub trait DistAFunc: Fn(usize,usize,&Vec<Vec<u64>>,&Vec<usize>) -> f64 {}
impl<T: Fn(usize,usize,&Vec<Vec<u64>>,&Vec<usize>) -> f64> DistAFunc for T {}


#[cfg(test)]
mod tests {
    use super::*;
    use network::{random_net};
    use coords::{build_coords, choose_landmarks, is_coord_unique};
    use self::rand::{StdRng};


    #[test]
    fn test_basic() {
        let seed: &[_] = &[1,2,3,4,5];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);
        let net = random_net(60,5,&mut rng);
        let landmarks = choose_landmarks(&net,10,&mut rng);
        let coords = build_coords(&net, &landmarks);

        is_coord_unique(&(coords.unwrap()));
    }
}
