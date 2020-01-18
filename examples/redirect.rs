extern crate shh;
use std::io::Read;

fn main() {
    let mut shh = shh::stdout().unwrap();
    println!("hello, world!",);
    let mut s = String::new();
    shh.read_to_string(&mut s).unwrap();
    assert_eq!(&s, "hello, world!\n"); // notice the new line!
}
