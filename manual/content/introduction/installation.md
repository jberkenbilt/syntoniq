+++
title = "Installation"
weight = 10
sort_by = "weight"
+++

# TODO

* Package with [dist](https://github.com/axodotdev/cargo-dist)
  * Decide which other files, such as examples, should go there; probably examples should be a separate download
* Embed examples/microtonal-hello.stq -- see docs/TODO
* `cargo install --locked --git https://github.com/jberkenbilt/syntoniq`
* Link to https://github.com/jberkenbilt/syntoniq/

Cover:
* Download and install using whatever distribution method
* Build from source including disabling csound feature and how build.rs works
* Run `syntoniq demo` (see TODO) and play resulting files
* If hardware is available, run the keyboard with a demo file. Do we want an embedded keyboard demo? If so, also provide a way to output the file.

Remember: Windows:
* Install https://www.tobias-erichsen.de/software/loopmidi.html
* Create a loop port called `syntoniq-loop`
```
cargo build --config .cargo/windows-cross.toml --no-default-features
```
