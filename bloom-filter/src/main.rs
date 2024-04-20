use bloom_filter::BloomFilter;

fn main() {
    let mut bf = BloomFilter::build(4, 4).expect("Sadge");
    let s = String::from("foo");
    bf.add(&s);
    bf.is_present(&s);
}
