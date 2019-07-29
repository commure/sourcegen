use sourcegen::sourcegen;

/// The contents of the following enum will be replaced with the enum rendered with the given amount
/// of literals.
#[sourcegen(generator = "generate-enum", count = 3)]
/// This comment is generated
pub enum TestEnum {
    Literal0,
    Literal1,
    Literal2,
}

/// The contents of the following enum will be replaced with the enum rendered with the given amount
/// of literals.
#[sourcegen(generator = "generate-struct", count = 3)]
/// This comment is generated
pub struct TestStruct {
    pub field0: usize,
    pub field1: usize,
    pub field2: usize,
}

/// Nesting works!
pub mod nested {
    #[sourcegen::sourcegen(generator = "generate-enum", count = 3)]
    /// This comment is generated
    pub enum TestEnum {
        Literal0,
        Literal1,
        Literal2,
    }
}

pub mod one;
pub mod two;

#[path = "five_other.rs"]
pub mod five;

#[sourcegen::sourcegen(generator = "generate-mod", count = 3)]
/// This comment is generated
pub mod generated {
    pub struct Struct0;
    pub struct Struct1;
    pub struct Struct2;
}
