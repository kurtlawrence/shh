(Rust) Silence stderr and stdout, optionally rerouting it.

Stdout Gagging

```rust
println!("STDOUT GAGGING", );
println!("you will see this");
let shh = shh::stdout().unwrap();
println!("but not this");
drop(shh);
println!("and this");
```

Stderr Gagging

```rust
println!("STDERR GAGGING", );
eprintln!("you will see this");
let shh = shh::stderr().unwrap();
eprintln!("but not this");
drop(shh);
eprintln!("and this");
```

Redirecting Example

```rust
println!("REDIRECTING", );
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
```

# Scoping

The struct `Shh` implements the `Drop` trait. Upon going out of scope, the redirection is reset and resources are cleaned up. A `Shh` will only last for the scope, and where no local variable is used, the silencing will not work.

## Example - Silencing Dropped Early
```rust
println!("you will see this");
shh::stdout().unwrap();        // Shh struct is created, and dropped, here
println!("and expect not to see this, but you will");
```

To fix this, just assign a local variable
```rust
println!("you will see this");
let shh = shh::stdout().unwrap();        // Shh struct is created here
println!("and expect not to see this");
drop(shh);    // and dropped here
println!("now it works!");
```

