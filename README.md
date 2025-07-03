# AhkO3

This project aims to provide Rust binding generation for AutoHotkey, similar to how PyO3 works 
for Python.

Everything is in early phases and not yet published to crates.io

## Usage

See the `examples` directory for working examples.

The idea is that you can write a Rust crate (a `cdylib` crate-type) like so:

```rust
use ahko3::prelude::*;

#[ahkfunction]
pub fn concatenate(a: String, b: String) -> String {
    format!("{}{}", a, b)
}

#[ahkfunction]
pub fn add(left: i64, right: i64) -> i64 {
    left + right
}
```


Then (not yet implemented) an AutoHotkey file will also be generated like `<crate_name>.ahk`

So the typical usage will then be in AHK something like:

```ahk
; include the generated file
#Include adder.ahk

; use the functions to call your rust code:

result := concatenate("foo", "bar")

MsgBox(result) ; "foobar"
```
