extern crate shh;

#[cfg(unix)]
fn main() {
    /////////////////////////////////////////////////////////////
    // Stdout gagging
    println!("STDOUT GAGGING",);
    println!("you will see this");
    let shh = shh::stdout().unwrap();
    println!("but not this");
    drop(shh);
    println!("and this");
    /////////////////////////////////////////////////////////////

    /////////////////////////////////////////////////////////////
    // Stderr gagging
    println!("STDERR GAGGING",);
    eprintln!("you will see this");
    let shh = shh::stderr().unwrap();
    eprintln!("but not this");
    drop(shh);
    eprintln!("and this");
    /////////////////////////////////////////////////////////////

    /////////////////////////////////////////////////////////////
    // Redirecting
    println!("REDIRECTING",);
    use std::io::{Read, Write};

    std::thread::spawn(move || {
        let mut shh = shh::stdout().unwrap();
        let mut stderr = std::io::stderr();
        loop {
            let mut buf = Vec::new();
            shh.read_to_end(&mut buf).unwrap();
            stderr.write_all(&buf).unwrap();
        }
    });

    println!("This should be printed on stderr");
    eprintln!("This will be printed on stderr as well");

    // This will exit and close the spawned thread.
    // In most cases you will want to setup a channel and send a break signal to the loop,
    // and then join the thread back into it once you are finished.

    /////////////////////////////////////////////////////////////
}

#[cfg(windows)]
fn main() {}
