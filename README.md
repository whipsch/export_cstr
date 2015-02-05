### Example usage

Cargo.toml:

```toml
[lib]
name = "foo"
crate-type = ["dylib"]

[dependencies]
export_cstr = "*"
```

lib.rs:

```rust
#![feature(plugin, libc)]
#[plugin] #[macro_use] extern crate export_cstr;

extern crate libc;
use libc::c_char;

export_cstr!(FOO, "this becomes an exported symbol 'FOO' which points to a constant, null-terminated, C string");

// ...
```

