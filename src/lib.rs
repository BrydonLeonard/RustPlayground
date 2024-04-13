use std::error::Error;
use std::hash::Hash;
use sha2::{Sha512, Digest};
use std::fmt;
use bit_vec::BitVec;

struct BloomFilter {
    bits: BitVec, // the bits that actually make up the bloom filter
    hasher_count: usize, // the number of hashers
    hasher_range_in_bits: u32, // the number of bits for each hash value. bits is effectively 2 ^ this value long
}

const FULL_HASH_BYTES: u32 = 512;

impl BloomFilter { 
    fn build(hasher_range_in_bits: u32, hasher_count: usize) -> Result<BloomFilter, &'static str> {
        if hasher_range_in_bits * (hasher_count as u32) > FULL_HASH_BYTES {
            return Err("The bloom filter is too large for the underlying hashers");
        }

        let mut bits = BitVec::from_elem(2_usize.pow(hasher_range_in_bits), false);

        Ok(
            BloomFilter { 
                bits,
                hasher_count, 
                hasher_range_in_bits, // TODO make this variable
            }
        )
    }

    // Adds the given string to the bloom filter
    fn add<T: AsRef<[u8]>>(&mut self, t: &T) {
        let t_hash = self.hash(t);

        for i in t_hash {
            self.bits.set(i, true);
        }
    }

    fn is_present<T: AsRef<[u8]>>(&self, t: &T) -> BloomCheckResult {
        let t_hash = self.hash(t);

        for i in t_hash {
            if !self.bits.get(i)
                .expect("the values produced by the hashers should be in the bounds of the bit array") {
                return BloomCheckResult::No;
            }
        }

        BloomCheckResult::Maybe
    }

    // Each bloom filter has [hasher_count] hashers, each of which hash a given value
    // to a single position in a bit vector. This method calculates those positions
    // for each of the hashers. In reality, this method is implemented by computing a 
    // single SHA512 hash value and using the necessary number of bits of the resulting
    // hash for each hasher. 
    //
    // As an example, for a bloom filter consisting of a bit vector with length 8, 3 bits
    // of the SHA512 hash will be used for each "hasher" because 2 ^ 3 == 8. The number in
    // [0 - 7] represented by each of those slices of three bits is the position of a 1 
    // in the final hash.
    //
    // The Vector returned from this method is a list of the positions of the 1s in the 
    // final hash for this value.
    fn hash<T: AsRef<[u8]>>(&self, t: &T) -> Vec<usize> {
        let mut hasher = Sha512::new();
        hasher.update(&t);
        let full_hash = hasher.finalize();

        let mut computed_hash: Vec<usize> = vec![0; self.hasher_count];
        // This moves along the full hash, keeping track of the bit we're working on
        let mut full_hash_ptr = 0;

        for bloom_hasher_index in 0..self.hasher_count {
            // The position of the 1 for this hasher
            let mut hasher_value: usize = 0;
            
            for _ in 0..self.hasher_range_in_bits {
                // The SHA512 hashes are grouped into bytes, so find the byte and bit
                // within that byte that we're considering.
                let byte_index: usize = (full_hash_ptr / 8).try_into().unwrap();
                let bit_in_byte = full_hash_ptr % 8;

                // Check the bit under consideration.
                let bit_mask: u8 = 2_u8.pow(bit_in_byte);
                let bit: bool = full_hash[full_hash.len() - byte_index - 1] & bit_mask != 0;

                // Add the bit to the hasher's value.
                hasher_value = (hasher_value << 1) + (bit as usize);
                full_hash_ptr += 1;
            }

            computed_hash[bloom_hasher_index] = hasher_value;
        }
        
        computed_hash
    }
}

impl fmt::Debug for BloomFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();

        for b in &self.bits {
            let v = if b {
                1
            } else {
                0
            };

            s.push_str(&format!("{} ", v));
        }

        write!(f, "{}", s)
    }
}

#[derive(PartialEq, Debug)]
enum BloomCheckResult {
    No,
    Maybe
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn rejects_invalid_size_and_hasher_count() {
        if let Ok(_) = BloomFilter::build(200, 7) {
            panic!("Should have rejected invalid input");
        }
    }

    #[test]
    fn accepts_valid_size_and_hasher_count() {
        if let Err(_) = BloomFilter::build(4, 6) { 
            panic!("Should have accepted valid input");
        }
    }

    #[test]
    fn no_false_negatives() {
        let mut bf = BloomFilter::build(4, 2)
            .expect("should have built a bloom filte");

        bf.add(&String::from("foo"));
        bf.add(&String::from("bar"));
        bf.add(&String::from("baz"));
        bf.add(&String::from("Green eggs and ham"));

        dbg!("{:?}", &bf);

        assert_eq!(bf.is_present(&String::from("foo")), BloomCheckResult::Maybe);
        assert_eq!(bf.is_present(&String::from("bar")), BloomCheckResult::Maybe);
        assert_eq!(bf.is_present(&String::from("baz")), BloomCheckResult::Maybe);
        assert_eq!(bf.is_present(&String::from("Green eggs and ham")), BloomCheckResult::Maybe);

        assert_eq!(bf.is_present(&String::from("not present")), BloomCheckResult::No);
        assert_eq!(bf.is_present(&String::from("nor I")), BloomCheckResult::No);
        assert_eq!(bf.is_present(&String::from("Green eggs and jam")), BloomCheckResult::No);
    }
}

