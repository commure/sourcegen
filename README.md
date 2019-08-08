# Sourcegen

**Source gen**erator

[![crates.io][Crate Logo]][Crate]
[![Documentation][Doc Logo]][Doc]
[![Build Status][CI Logo]][CI]

Sourcegen is a toolkit for in-place source code generation in Rust.

In-place source code generation is like a procedural macro in Rust, but with a difference that it is expanded before
Rust code is compiled. For example, one use-case could be generating data types based on an external definition.

You start with the following code:

```rust
#[sourcegen::sourcegen(generator = "json-schema", schema = "widget.json")]
struct Widget;
```

Then, you run a special tool that is built on top of the `sourcegen-cli` crate:

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

## Creating a Tool

In the current form, you build your own tool on top of the `sourcegen_cli::run_tool` entry point. This function takes
a number of input parameters and a list of source generators implementations.

Source generators are similar to procedural macros, they take syntax as an input and return token stream as an output.
Input to source generators use [`syn`](https://crates.io./crates/syn) crate for representing syntax trees. Returned tokens are
rendered by generators into the source code and formatted via `rustfmt`. 

## Rationale

What are the benefits of generating source code this way compared to using procedural macros or generating code during
the build via `build.rs`?

Advantages over procedural macros:

1. Does not take compilation time.
2. Compilation does not depend on original metadata used for generation.
3. You have source code to look at. This is especially useful when generated code are data types of some sort.
4. The generator code can depend on the generated types (bootstrapping). 

Advantages over `build.rs` source generation:

1. Does not take compilation time.
2. Compilation does not depend on original metadata used for generation.
3. More flexible setup: can generate code piecemeal directly where it is used.
4. No need to include generated code via `include!` or other means (`build.rs` cannot write to sources).

However, there are also some disadvantages:

1. Potential desynchronization between source of truth metadata and source code.
2. If source generators depend on their own output, harder to work on the source generators themselves (if they generate
invalid code, recovering might require reverting the generated code via source control). 
3. Too magical.

<!-- work in progress... -->

[Crate]: https://crates.io/crates/sourcegen-cli
[Crate Logo]: https://img.shields.io/crates/v/sourcegen-cli.svg

[Doc]: https://docs.rs/sourcegen-cli
[Doc Logo]: https://docs.rs/sourcegen-cli/badge.svg

[CI]: https://dev.azure.com/commure/sourcegen/_build/latest?definitionId=1&branchName=master
[CI Logo]: https://dev.azure.com/commure/sourcegen/_apis/build/status/commure.sourcegen?branchName=master