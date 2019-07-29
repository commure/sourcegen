use sourcegen::sourcegen;

pub mod four;

// Included as `<mod>/mod.rs`
#[sourcegen(generator = "generate-enum", count = 3)]
/// This comment is generated
pub enum Second {
    Literal0,
    Literal1,
    Literal2,
}
