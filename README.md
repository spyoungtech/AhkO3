# AhkO3

This project aims to provide Rust binding generation for AutoHotkey, similar to how PyO3 works 
for Python.

Everything is in early phases and not yet published to crates.io

## Usage

The idea is that you can write a Rust crate (a `cdylib` crate-type) like so:

```rust
// example for crate named "adder"
// adder/src/lib.rs

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


Then (not yet implemented) an AutoHotkey file will also be generated like so:

```ahk
; adder.ahk -- automatically generated

#DllLoad "*i adder.dll"
if !DllCall("GetModuleHandle", "str", "adder") {
    throw Error("Cannot load adder.dll -- please ensure it is on PATH or use #DllLoad to load it in your script before your #Inlude of adder.ahk")
}



concatenate(a, b) {
    res := DllCall("adder\gen_concatenate", "Str", a, "Str", b, "Ptr")
    s := StrGet(res)
    DllCall("adder\ahko3_free_string_ptr", "Ptr", res, "Int64")
    return s
}


add(a, b) {
    res := DllCall("adder\gen_add", "Int64", a, "Int64", b, "Int64")
    return res
}
```

So the typical usage will then be in AHK something like:

```ahk
; include the generated file
#Include adder.ahk

; use the functions to call your rust code:

result := concatenate("foo", "bar")

MsgBox(result) ; "foobar"
```
