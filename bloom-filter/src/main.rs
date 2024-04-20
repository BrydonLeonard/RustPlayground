use std::hash::{DefaultHasher, Hasher, Hash};

fn main() {
    let mut hasher = DefaultHasher::new();
    String::from("foo bar").hash(&mut hasher);
    println!("{}", hasher.finish());
}
