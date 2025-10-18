# clap_complete_clink

Generate [Clink](https://chrisant996.github.io/clink/clink.html) completion scripts for [`clap`](https://github.com/clap-rs/clap) command-line apps.

```rust
use clap_complete::generate;
use clap_complete_clink::Clink; // < (1)

let mut buf = Vec::new();
generate(Clink, clap_command, "awesome_app", &mut buf)?;
//   (2) ^^^^^
```
