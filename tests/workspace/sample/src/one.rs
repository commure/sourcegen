use sourcegen::sourcegen;

pub mod three;

#[sourcegen(generator = "generate-enum", count = 3)]
/// This comment is generated by source expander
pub enum First {
    Literal0,
    Literal1,
    Literal2,
}