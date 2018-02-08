# Animation of Yves Lafont's Interaction Combinators

[Interaction combinators][article] are a universal model of distributed
computation devised by Yves Lafont in 1997. They are a particular instance of
[Interaction nets][wikipedia]. Interaction nets are a graphical model of
computation also devised by Yves Lafont in 1990.

This project renders the computation of interaction combinators in a simplified
physical 3D world where agents are spheres and edges are invisible.

This is not an official Google product.

## How to install

You can install the latest version released on https://crates.io/crates/lafont with:

    cargo install lafont

Or you can install the latest commit from https://github.com/ia0/lafont with:

    cargo install --git=https://github.com/ia0/lafont.git lafont

The binary will be installed as `~/.cargo/bin/lafont` by default.

If you don't have `cargo` (the Rust package manager), install it through
https://rustup.rs/ or through your package manager (e.g. `apt install cargo` on
Debian-like machines).

[article]: https://dl.acm.org/citation.cfm?id=264415
[wikipedia]: https://en.wikipedia.org/wiki/Interaction_nets
