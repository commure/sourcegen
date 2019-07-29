use sourcegen::sourcegen;

// Included as `<mod>/mod.rs`
#[sourcegen(generator = "generate-enum", count = 3)]
/// This comment is generated
pub enum Five {
    Literal0,
    Literal1,
    Literal2,
}
