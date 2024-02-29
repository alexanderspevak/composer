# Composer

## About

Transforms all inputs to sandardized version: `x.x.x`. 

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
composer = { path = "/path/to/composer" }
```
## Usage
Instantiate:
```rust
use composer::ComposerVersion;

let version = ComposerVersion::new("2023-023-29-v1");

```

Bump:
```rust
version.bump_major();
version.bump_minor();
version.bump_patch();

```

ComposerVersion implements display trait:
```rust
println!("{}",version);
```
To obtain original string:
```rust
version.get_original();
```