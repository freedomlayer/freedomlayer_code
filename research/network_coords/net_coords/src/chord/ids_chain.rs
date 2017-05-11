use chord::{RingKey};

pub struct IdsChain {
    cur_id: Option<RingKey>, // Current id
    dst_id: RingKey, // Destination id
}

/// Find the msb bit index of a given number.
fn get_msb(mut x: RingKey) -> Option<usize> {
    match x {
        0 => None,
        _ => {
            let mut index: usize = 0;
            while x > 0 {
                x >>= 1;
                index += 1;
            }
            Some(index - 1)
        }
    }
}

fn advance_id(cur_id: RingKey, dst_id: RingKey) -> RingKey {
    // Find the most significant different bit between cur_id and dst_id:
    let msb_diff: usize = get_msb(cur_id ^ dst_id).unwrap();

    // Check if we need to add or to subtract:
    let pow_diff: RingKey = 2_u64.pow(msb_diff as u32);
    match (cur_id >> msb_diff) & 1 {
        0 => cur_id + pow_diff,
        _ => cur_id - pow_diff,
    }
}

///
/// Iterator for a chain of ids between some source id and a destination id.
/// Every two adjacent produced ids have a difference which is an exact
/// power of 2.
/// This iterator is guaranteed to be deterministic. (It will return the same
/// chain for the same source and destination ids every time).
impl Iterator for IdsChain {
    type Item = RingKey;
    fn next(&mut self) -> Option<RingKey> {
        match self.cur_id {
            None => None,
            Some(cur_id) => {
                if cur_id == self.dst_id {
                    self.cur_id = None
                } else {
                    self.cur_id = Some(advance_id(cur_id, self.dst_id));
                }
                Some(cur_id)
            }
        }
    }
}

fn ids_chain(src_id: RingKey, dst_id: RingKey) -> IdsChain {
    IdsChain {
        cur_id: Some(src_id),
        dst_id: dst_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_msb() {
        assert!(get_msb(0) == None);
        assert!(get_msb(1) == Some(0));
        assert!(get_msb(2) == Some(1));
        assert!(get_msb(3) == Some(1));
        assert!(get_msb(4) == Some(2));
        assert!(get_msb(5) == Some(2));
        assert!(get_msb(6) == Some(2));
        assert!(get_msb(7) == Some(2));
        assert!(get_msb(8) == Some(3));
        assert!(get_msb(9) == Some(3));
    }

    #[test]
    fn test_ids_chain_trivial() {
        let ic = ids_chain(0,1).collect::<Vec<_>>();
        println!("{:?}",ic);
        assert!(ic[0] == 0);
        assert!(ic[1] == 1);
        assert!(ic.len() == 2);
    }

    fn is_power2(x: u64) -> bool {
        match x {
            0 => false, 
            1 => true,
            _ => x & (x - 1) == 0,
        }
    }
    
    /// Check if two ids are adjacent
    fn is_adjacent(id_a: RingKey, id_b: RingKey) -> bool {
        match id_a >= id_b {
            true => is_power2(id_a - id_b),
            _ => is_power2(id_b - id_a),
        }
    }

    #[test]
    fn test_ids_chain_long() {
        let ic = ids_chain(0xdeadbeef,0x12345678).collect::<Vec<_>>();
        let ic0 = ic.iter();
        let ic1 = ic.iter().skip(1);

        assert!(ic1.zip(ic0).all(|(&next_id, &id)| is_adjacent(next_id, id)));
        assert!(ic.len() > 2);
    }
}
