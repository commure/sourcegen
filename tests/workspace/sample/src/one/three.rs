use sourcegen::sourcegen;

#[sourcegen(generator = "generate-enum", count = 3)]
// Generated. All manual edits to the block annotated with #[sourcegen...] will be discarded.
/// This comment is generated
pub enum Third {
    Literal0,
    Literal1,
    Literal2,
}
