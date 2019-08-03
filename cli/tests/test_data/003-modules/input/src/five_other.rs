use sourcegen::sourcegen;

#[sourcegen(generator = "generate-simple")]
// Generated. All manual edits to the block annotated with #[sourcegen...] will be discarded.
pub struct TestStruct;
