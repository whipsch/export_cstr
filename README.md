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


## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
