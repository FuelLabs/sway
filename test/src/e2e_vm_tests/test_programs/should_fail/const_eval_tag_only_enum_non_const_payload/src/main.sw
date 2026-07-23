library;

struct EmptyStruct {}

// All variants are zero-sized, so `SomeEnum` is lowered to just its tag and the variant
// payloads are dropped in both const-eval and codegen.
enum SomeEnum {
    A: EmptyStruct,
    B: (),
}

fn non_const_eval_empty_struct() -> EmptyStruct {
    // ASM block to ensure the function is not const-evaluable.
    asm() {
        noop;
    };
    EmptyStruct {}
}

// NOK: Even though `SomeEnum` is tag-only and the payload carries no data, the payload
// initializer must still be const-evaluated. It is not const-evaluable, so this must be
// rejected at compile time.
const SOME_ENUM: SomeEnum = SomeEnum::A(non_const_eval_empty_struct());
