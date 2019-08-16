use sourcegen;

#[sourcegen::sourcegen(generator = "generate-simple")]
// Generated. All manual edits to the block annotated with #[sourcegen...] will be discarded.
pub struct TestStruct;

/// Nested modules
pub mod nested {
    #[sourcegen::sourcegen(generator = "generate-simple")]
    // Generated. All manual edits to the block annotated with #[sourcegen...] will be discarded.
    pub struct TestStruct;
}

pub mod one;
pub mod two;

#[path = "five_other.rs"]
pub mod five;

mod six {
    mod seven {
        mod eight;
        mod nine;
    }
}
