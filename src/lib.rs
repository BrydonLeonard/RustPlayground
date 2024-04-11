use std::error::Error;
use std::hash::{Hash, Hasher};
use sha2::{Sha512, Digest};
use std::fmt;

struct BloomFilter {
    bytes: Vec<u8>,
    hasher_count: u8,
    byte_count: usize,
}

const MAX_BYTES: usize = 64;

impl BloomFilter { 
    fn build(byte_count: usize, hasher_count: u8) -> Result<BloomFilter, &'static str> {
        if byte_count > MAX_BYTES / usize::from(hasher_count) {
            return Err("The bloom filter is too large for the underlying hashers");
        }

        let mut bytes = Vec::new();

        for _ in 0..byte_count {
            bytes.push(0);
        }

        Ok(
            BloomFilter { 
                bytes,
                hasher_count, 
                byte_count,
            }
        )
    }

    // Adds the given string to the bloom filter
    fn add<T: AsRef<[u8]>>(&mut self, t: &T) {
        let t_hash = self.hash(t);

        for i in 0..t_hash.len() {
            self.bytes[i] = self.bytes[i] | t_hash[i]
        }
    }

    fn is_present<T: AsRef<[u8]>>(&self, t: &T) -> BloomCheckResult {
        let t_hash = self.hash(t);

        for i in 0..t_hash.len() {
            if self.bytes[i] & t_hash[i] != t_hash[i] {
                return BloomCheckResult::No;
            }
        }

        BloomCheckResult::Maybe
    }

    fn hash<T: AsRef<[u8]>>(&self, t: &T) -> Vec<u8> {
        let mut hasher = Sha512::new();
        hasher.update(&t);
        let full_hash = hasher.finalize();

        let mut computed_hash: Vec<u8> = vec![0; self.byte_count];

        // This pointer will step through the full hash so that we can wrap it back around
        let mut full_hash_index = 0;
        for _ in 0..self.hasher_count {
            for byte_index in 0..self.byte_count {
                // work backwards and bitwise or each byte with the relevant 
                // byte of the bloom filter
                computed_hash[byte_index] = &computed_hash[self.byte_count - byte_index - 1] | full_hash[full_hash.len() - full_hash_index - 1];
    
                full_hash_index += 1;
            }
        }
        
        computed_hash
    }
}

impl fmt::Debug for BloomFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut s = String::new();

        for b in &self.bytes {
            s.push_str(&format!("{:b} ", b));
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
        if let Ok(_) = BloomFilter::build(10, 7) {
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
        let mut bf = BloomFilter::build(30, 1)
            .expect("Failed to build bloom filter");

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

