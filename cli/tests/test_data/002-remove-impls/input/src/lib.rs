#[sourcegen::sourcegen(generator = "generate-impls")]
// Generated. All manual edits to the block annotated with #[sourcegen...] will be discarded.
struct Hello;
#[sourcegen::generated]
impl Hello {}

struct Irrelevant;

#[sourcegen::sourcegen(generator = "generate-impls")]
// Generated. All manual edits to the block annotated with #[sourcegen...] will be discarded.
struct Hello2;
#[sourcegen::generated]
impl Hello2 {}
