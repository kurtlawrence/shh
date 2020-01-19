extern crate shh;
use std::io::Read;

fn main() {
    // if this blocks then it is failing.
    println!("this will be printed");
    let mut shh = shh::stdout().unwrap();
    let mut s = String::new();
    assert_eq!(shh.read_to_string(&mut s).unwrap(), 0); // should exit immediately and return empty string
    assert_eq!(&s, "");

    // if this blocks then it is failing.
    eprintln!("this will be printed");
    let mut shh = shh::stderr().unwrap();
    let mut s = String::new();
    assert_eq!(shh.read_to_string(&mut s).unwrap(), 0); // should exit immediately and return empty string
    assert_eq!(&s, "");
}
