use sourcegen::sourcegen;

#[sourcegen(generator = "generate-enum", count = 3)]
/// This comment is generated
pub enum Third {
    Literal0,
    Literal1,
    Literal2,
}
