mod r#async {
    #[sourcegen::sourcegen(generator = "generate-simple")]
    // Generated. All manual edits to the block annotated with #[sourcegen...] will be discarded.
    struct Test {
        pub hello: String,
    }
}

mod r#await;
