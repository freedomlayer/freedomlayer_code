extern crate num;
use self::num::Complex;
use std::f64;

/// Convert network coordinate to chord value in [0,1) 
/// by projection to a plane.
pub fn old_coord_to_ring(coord: &Vec<usize>) -> f64 {
    let fcoord: Vec<f64> = coord.iter().map(|&a| a as f64).collect();

    let k: f64 = fcoord.len() as f64;
    let S_a:f64 = fcoord.iter().sum();
    let normalize = |a| (a/S_a) - (1.0/k);
    let L_a: f64 = fcoord.iter().
        map(|&a| normalize(a).powi(2) as f64).sum::<f64>().sqrt();

    let numerator: f64 = 
        normalize(fcoord[0]) - 
            (1.0/(k-1.0)) * fcoord.iter().skip(1).map(|&a| normalize(a)).sum::<f64>();

    let denominator: f64 = L_a * ((k/(k-1.0))).sqrt();

    (numerator/denominator).acos() / (f64::consts::PI)
}

pub fn coord_to_ring_all_pairs(coord: &Vec<usize>) -> f64 {
    assert!(coord.len() > 1);
    let fcoord: Vec<f64> = coord.iter().map(|&a| a as f64).collect();

    let k: f64 = fcoord.len() as f64;
    let S_a:f64 = fcoord.iter().sum();
    let normalize = |a| (a/S_a) - (1.0/k);
    let L_a: f64 = fcoord.iter().
        map(|&a| normalize(a).powi(2) as f64).sum::<f64>().sqrt();

    let scoord: Vec<f64> = fcoord.into_iter().map(|a| normalize(a) / L_a).collect();


    let mut sum: f64 = 0.0;
    for i in 0..scoord.len() {
        for j in i+1..scoord.len() {
            let x = scoord[i];
            let y = scoord[j];
            let addition = 0.5 + (y.atan2(x) / (2.0 * f64::consts::PI));
            // println!("Addition = {}",addition);
            sum += addition;
        }
    }

    // let pairs: f64 = k * (k-1.0) / 2.0;
    let f = (sum).fract();
    assert!(f >= 0.0);

    f
}

pub fn coord_to_ring_adj_pairs(coord: &Vec<usize>) -> f64 {
    assert!(coord.len() > 1);
    let fcoord: Vec<f64> = coord.iter().map(|&a| a as f64).collect();

    let k: f64 = fcoord.len() as f64;
    let S_a:f64 = fcoord.iter().sum();
    let normalize = |a| (a/S_a) - (1.0/k);
    let L_a: f64 = fcoord.iter().
        map(|&a| normalize(a).powi(2) as f64).sum::<f64>().sqrt();

    let scoord: Vec<f64> = fcoord.into_iter().map(|a| normalize(a) / L_a).collect();

    let mut sum: f64 = 0.0;
    for i in 0..scoord.len() {
        let x = scoord[i];
        let y = scoord[(i + 1) % scoord.len()];
        let addition = 0.5 + (y.atan2(x) / (2.0 * f64::consts::PI));
        // println!("Addition = {}",addition);
        sum += addition;
    }

    let f = (sum).fract();
    assert!(f >= 0.0);
    f
}

pub fn coord_to_ring(coord: &Vec<usize>) -> f64 {
    let k: f64 = coord.len() as f64;
    let ang_part = (2.0 * f64::consts::PI) / k;

    let sum: Complex<f64> = 
        coord.iter().map(|&a| a as f64).enumerate()
            .fold(Complex::new(0.0,0.0), |acc, (i,x)|
                acc + Complex::from_polar(&((-x*2.0).exp()),&(ang_part * (i as f64))));

    (sum.arg() + f64::consts::PI) / (2.0 * f64::consts::PI)
}
