use sourcegen::sourcegen;

pub mod three;

#[sourcegen(generator = "generate-enum", count = 3)]
// Generated. All manual edits to the block annotated with #[sourcegen...] will be discarded.
/// This comment is generated
pub enum First {
    Literal0,
    Literal1,
    Literal2,
}
