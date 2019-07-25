# Sourcegen

**Source gen**erator

[![crates.io][Crate Logo]][Crate]
[![Documentation][Doc Logo]][Doc]

Sourcegen is a toolkit for in-place source code generation in Rust.

In-place source code generation is like a procedural macro in Rust, but with a difference that it is expanded before
Rust code is compiled. For example, one use-case could be generating data types based on an external definition.

You start with the following code:

```rust
#[sourcegen::sourcegen(generator = "json-schema", schema = "widget.json")]
struct Widget;
```

Then, you run a special tool that is built on top of the `sourcegen-cli`:

```sh
cargo run --package json-schema-sourcegen
``` 

Which expands the code above into something like (this assumes that `widget.json` schema defines a data type with
two fields, `name` and `weight`:

```rust
#[sourcegen::sourcegen(generator = "json-schema", schema = "widget.json")]
struct Widget {
    /// Name of the widget
    name: String,
    /// Weight of the widget
    weight: usize,
}
```

Next time you run the tool, it would not change the code as it is already in it's correct form.

## Rationale

<!-- work in progress... -->

## Creating a Tool

In the current form, you build your own tool on top of the `sourcegen_cli::run_tool` entry point. This function takes
a number of input parameters and a list of source generators implementations.

Source generators are similar to procedural macros, they take syntax as an input and return token stream as an output.
Input to source generators use [`syn`](https://crates.io./syn) crate for representing syntax trees. Returned tokens are
rendered by generators into the source code and formatted via `rustfmt`. 

[Crate]: https://crates.io/crates/sourcegen-cli
[Crate Logo]: https://img.shields.io/crates/v/sourcegen-cli.svg

[Doc]: https://docs.rs/sourcegen
[Doc Logo]: https://docs.rs/sourcegen/badge.svg
