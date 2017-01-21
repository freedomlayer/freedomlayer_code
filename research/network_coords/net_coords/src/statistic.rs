extern crate num;

use self::num::{Num, Zero, One, FromPrimitive, Float};

use std::fmt::Debug;
use std::iter::Sum;
use std::ops::Div;
use std::cmp::Ordering::Less;


trait Stream<N> {
    fn to_rank(&self) -> Vec<usize>;
    fn mean(&self) -> N;
    fn variance(&self, origin: N) -> N;
}

impl<N> Stream<N> for Vec<N> where
    N: Float + Div + Copy + FromPrimitive {

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

    fn variance(&self, origin: N) -> N {
        self.iter().fold::<N,_>(num::zero(),|acc, &val| acc + (origin - val) * (origin - val)) / 
            (FromPrimitive::from_usize(self.len()).unwrap())
    }
}

/// Calculate pearson correlation coefficient between two streams.
fn pearson<N>(a: &Vec<N>, b: &Vec<N>) -> Option<N> 
    where N: Float + Div + Copy + FromPrimitive {

    if a.len() != b.len() {
        return None
    }

    let mean_a: N = a.mean();
    let sig_a: N = a.variance(mean_a).sqrt();

    let mean_b: N = b.mean();
    let sig_b: N = b.variance(mean_b).sqrt();

    let p = (0 .. a.len()).map(|i| (a[i] - mean_a) * (b[i] - mean_b))
        .collect::<Vec<_>>().mean() / (sig_a * sig_b);

    Some(p)
}

/// Calculate spearman monotonicity correlation coefficient between two streams.
fn spearman<N>(a: &Vec<N>, b:&Vec<N>) -> Option<N>
    where N: Float + Div + Copy + FromPrimitive {

    let rank_n = |v: &Vec<N>| v.to_rank().into_iter()
        .map(|x| FromPrimitive::from_usize(x).unwrap())
        .collect::<Vec<_>>();

    pearson(&rank_n(a), &rank_n(b))
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_to_rank() {
        let rank = vec![6.0,3.0,4.0].to_rank();
        assert!(rank == vec![2,0,1]);
    }

    #[test]
    fn test_stream_to_rank_equals() {
        let rank = vec![5.0,5.0,4.0].to_rank();
        assert!(rank[2] == 0);
        assert!(vec![rank[0],rank[1]] == vec![1,2]);
    }

    #[test]
    fn test_mean() {
        let mean: f64 = vec![1.0,1.5,2.0].mean();
        assert!((mean - 1.5).abs() < 0.0001);
    }

    #[test]
    fn test_variance() {
        let v: Vec<f64> = vec![1.0, 1.5, 2.0];
        let var: f64 = v.variance(v.mean());
        let var2: f64 = vec![0.5*0.5, 0.0, 0.5*0.5].mean();
        assert!((var - var2).abs() < 0.0001)
    }

    #[test]
    fn test_pearson() {
        let a: Vec<f64> = vec![1.0,2.0,3.0,4.0,5.0];
        let b: Vec<f64> = vec![1.0,2.0,3.0,4.0,5.0];
        let p = pearson(&a,&b).unwrap();
        assert!((p - 1.0).abs() < 0.0001);

        let c: Vec<f64> = vec![-1.0,-2.0,-3.0,-4.0,-5.0];
        let p = pearson(&a,&c).unwrap();
        assert!((p + 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_spearman() {
        let a: Vec<f64> = vec![1.0,2.0,3.0,4.0,5.0];
        let b: Vec<f64> = vec![1.0,2.0,3.0,4.0,5.0];
        let p = spearman(&a,&b).unwrap();
        assert!((p - 1.0).abs() < 0.0001);

        let c: Vec<f64> = vec![1.0,5.0,6.0,6.5,6.6];
        let p = spearman(&a,&c).unwrap();
        assert!((p - 1.0).abs() < 0.0001);
    }

}
