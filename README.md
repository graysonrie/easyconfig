### Easy Config

A simple library for managing config files in Rust.

Usage:

```rust
#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
struct MyConfig {
    pub name: String,
    pub age: u32,
    pub favorite_numbers: Vec<u32>,
}

let holder = ConfigHolder::<MyConfig>::new(SaveTo::AppData, "config");
let config = holder.get_or_create().unwrap();
```
