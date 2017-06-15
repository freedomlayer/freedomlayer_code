
use std::mem;

struct SmallestK<'a,T: 'a> {
    pub smallest_k: Vec<T>,
    k: usize,
    lt: &'a Fn(&T, &T) -> bool,
}

impl<'a, T: Ord + Clone> SmallestK<'a, T> {
    /// Create a new BestK
    pub fn new(k: usize, lt: &'a Fn(&T, &T) -> bool) -> SmallestK<T> {
        SmallestK { 
            smallest_k: Vec::new(),
            k,
            lt,
        }
    }

    /// Update with element T, possibly better than 
    /// one of the other elements.
    pub fn update(&mut self, candidate: &T) -> bool {
        let found_index_opt: Option<usize> = {
            let find_res = self.smallest_k
                .iter()
                .enumerate()
                .find(|&(_,elem)| (self.lt)(candidate,elem));

            match find_res {
                None => None,
                Some((i, _)) => Some(i),
            }
        };

        match found_index_opt {
            None => {
                if self.smallest_k.len() < self.k {
                    self.smallest_k.push(candidate.clone());
                    true
                } else {
                    false
                }
            },
            Some(found_index) => {
                let mut cur_val = candidate.clone();
                for j in found_index .. self.smallest_k.len() {
                    mem::swap(&mut cur_val, &mut self.smallest_k[j]);
                }
                // If we have extra room, put the largest value at 
                // the end of the vector:
                if self.smallest_k.len() < self.k {
                    self.smallest_k.push(cur_val);
                }
                true
            }
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smallest_k_basic() {
        let lt = |a: &u64,b: &u64| *a < *b;
        let mut bk = SmallestK::<u64>::new(3, &lt);
        assert!(bk.smallest_k == vec![]);
        bk.update(&6);
        assert!(bk.smallest_k == vec![6]);
        bk.update(&5);
        assert!(bk.smallest_k == vec![5,6]);
        bk.update(&5);
        assert!(bk.smallest_k == vec![5,5,6]);
        bk.update(&7);
        assert!(bk.smallest_k == vec![5,5,6]);
        bk.update(&3);
        assert!(bk.smallest_k == vec![3,5,5]);
        bk.update(&4);
        assert!(bk.smallest_k == vec![3,4,5]);
        bk.update(&4);
        assert!(bk.smallest_k == vec![3,4,4]);
        bk.update(&2);
        assert!(bk.smallest_k == vec![2,3,4]);
        bk.update(&1);
        assert!(bk.smallest_k == vec![1,2,3]);
    }
}
