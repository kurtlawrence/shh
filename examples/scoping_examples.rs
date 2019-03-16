extern crate shh;

use std::thread;

fn main() {
    // if no local variable assigned, output won't be silenced.
    println!("you will see this");
    shh::stdout().unwrap(); // Shh struct is created, and dropped, here
    println!("and expect not to see this, but you will");

    // To fix this, just assign a local variable
    println!("you will see this");
    let shh = shh::stdout().unwrap(); // Shh struct is created here
    println!("and expect not to see this");
    drop(shh); // and dropped here
    println!("now it works!");

    // threading example
    let shh = shh::stdout().unwrap();
    println!("you won't see this",);
    thread::spawn(move || {
        let _move_it = shh;
        println!("nor this",);
    })
    .join()
    .unwrap();
    println!("you will see this thread though",);

    // even if you don't move it!
    let shh = shh::stdout().unwrap();
    println!("you won't see this",);
    thread::spawn(move || {
        println!("nor this",);
    })
    .join()
    .unwrap();
    println!("you also won't see this",);
    drop(shh);
    println!("you only see this!");
}
