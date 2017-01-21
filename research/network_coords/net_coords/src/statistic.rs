extern crate num;

use self::num::{Num, Zero, One, FromPrimitive};

use std::fmt::Debug;
use std::iter::Sum;
use std::ops::Div;
use std::cmp::Ordering::Less;



trait Stream<N> {
    fn to_rank(&self) -> Vec<usize>;
    fn mean(&self) -> N;
    // fn variance(&self) -> N;
}

impl<N> Stream<N> for Vec<N> where
    N: Num + Div + PartialOrd + Copy + FromPrimitive + Debug {

    fn to_rank(&self) -> Vec<usize> {
        let mut svec: Vec<(usize,&N)> = self.iter().enumerate().collect();
        svec.sort_by(|&(_, &val_a), &(_, &val_b)| val_a.partial_cmp(&val_b).unwrap_or(Less));
        // TODO: Use quickersort package here instead---^^^^^^^^^^^

        let mut perm_vec: Vec<(usize, usize)> = 
            svec.into_iter().map(|(index, _)| index)
            .enumerate().collect();
        perm_vec.sort_by(|&(_, pi_a), &(_, pi_b)| pi_a.cmp(&pi_b));
        perm_vec.into_iter().map(|(index, _)| index).collect()
    }

    fn mean(&self) -> N {
        self.iter().fold::<N,_>(num::zero(),|acc, &val| acc + val) / 
            (FromPrimitive::from_usize(self.len()).unwrap())
    }

    fn variance(&self) -> N {
        let m: N = self.mean();
        self.iter().fold::<N,_>(num::zero(),|acc, &val| acc + (m - val) * (m - val)) / 
            (FromPrimitive::from_usize(self.len()).unwrap())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_to_rank() {
        let rank = vec![6,3,4].to_rank();
        assert!(rank == vec![2,0,1]);
    }

    #[test]
    fn test_stream_to_rank_equals() {
        let rank = vec![5,5,4].to_rank();
        assert!(rank[2] == 0);
        assert!(vec![rank[0],rank[1]] == vec![1,2]);
    }

    #[test]
    fn test_mean() {
        let mean: f64 = vec![1.0,1.5,2.0].mean();
        assert!((mean - 1.5) < 0.0001);
    }

    #[test]
    fn test_variance() {
        let var: f64 = vec![1.0,1.5,2.0].variance();
        let var2: f64 = vec![0.5*0.5, 0.0, 0.5*0.5].mean();
        assert!((var - var2) < 0.0001)
    }
}
