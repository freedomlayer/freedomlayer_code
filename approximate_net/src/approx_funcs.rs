use std;

/// Calculate mean of values.
fn mean(vals: &[f64]) -> f64 {
    let amount = vals.len() as f64;
    vals.iter()
        .map(|&x| x / amount)
        .sum()
}

/// Calculate harmonic mean of given values
fn harmonic_mean(vals: &[f64]) -> f64 {
    let fsum: f64 = vals.iter()
        .map(|&x| 1.0 / x)
        .sum();

    (vals.len() as f64) / fsum
}

pub fn approx_size_harmonic_before(mins: &[u64]) -> usize {

    let fmeans = mins.iter()
        .map(|&m| m as f64)
        .collect::<Vec<f64>>();
    let hmean_min = harmonic_mean(&fmeans);
    (((u64::max_value() as f64)/ hmean_min) as usize) - 1
}

pub fn approx_size_harmonic_after(mins: &[u64]) -> usize {
    let trans = mins.iter()
        .map(|&m| (u64::max_value() / m) - 1)
        .map(|x| x as f64)
        .collect::<Vec<f64>>();

    harmonic_mean(&trans) as usize
}

pub fn approx_size_mean_before(mins: &[u64]) -> usize {
    let fmeans = mins.iter()
        .map(|&m| m as f64)
        .collect::<Vec<f64>>();
    let mean_min = mean(&fmeans);
    (((u64::max_value() as f64)/ mean_min) as usize) - 1

}

pub fn approx_size_mean_after(mins: &[u64]) -> usize {
    let trans = mins.iter()
        .map(|&m| (u64::max_value() / m) - 1)
        .map(|x| x as f64)
        .collect::<Vec<f64>>();

    mean(&trans) as usize
}

pub type ApproxFunc = (Fn(&[u64]) -> usize +  std::marker::Sync);

pub static APPROX_FUNCS_NAMED: &[(&ApproxFunc, &'static str)] = 
    &[
        (&approx_size_harmonic_before, "approx_size_harmonic_before"), 
        (&approx_size_harmonic_after, "approx_size_harmonic_after"),
        (&approx_size_mean_before, "approx_size_mean_before"),
        (&approx_size_mean_after, "approx_size_mean_after"),
    ];


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_approx_size_harmonic() {
        let mins = &[111,222,333,4,555];
        approx_size_harmonic_before(mins);
    }

}
