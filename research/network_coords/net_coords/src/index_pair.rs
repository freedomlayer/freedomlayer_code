
pub enum Pair<T> {
    Two(T, T),
    One(T),
    None,
}

/// Get two mutable cells out of a slice, safely.
pub fn index_pair<T>(slc: &mut [T], a: usize, b: usize) -> Pair<&mut T> {
    if a == b {
        slc.get_mut(a).map_or(Pair::None, Pair::One)
    } else if a >= slc.len() || b >= slc.len() {
        Pair:: None
    } else if a > b {
        let (low, high) = slc.split_at_mut(a);
        Pair::Two(high.get_mut(0).unwrap(), low.get_mut(b).unwrap())
    } else {
        // a < b
        let (low, high) = slc.split_at_mut(b);
        Pair::Two(low.get_mut(a).unwrap(), high.get_mut(0).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_pair() {
        let mut v = vec![0,1,2,3,4,5,6];
        match index_pair(&mut v, 2, 3) {
            Pair::Two(a,_) => *a += 1,
            _ => {},
        };

        assert!(v[2] == 3);
    }

}
