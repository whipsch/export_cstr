# This crate is no longer useful.
See [PR #18480](https://github.com/rust-lang/rust/pull/18480)

---


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
#![feature(plugin)]
#[plugin] #[macro_use] extern crate export_cstr;

// implicit #[allow(dead_code, non_upper_case_globals)]
export_cstr!(foo, "this becomes an exported symbol 'foo' which points to a constant, null-terminated, C string");

// ...
```

