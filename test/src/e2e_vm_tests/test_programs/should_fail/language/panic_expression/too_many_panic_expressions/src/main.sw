script;

#[allow(dead_code)]
fn main() {
    let t = true;
    // 10 x 25 = 250 `panic` expressions.
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    // 6 more `panic` expressions to reach the limit of 256.
    if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}if t {panic;}
    if t {panic "This is the 257th `panic` expression.";}
    if t {panic "This is the 258th `panic` expression.";}
    if t {panic "This is the 259th `panic` expression.";}
    if t {panic "This is the 260th `panic` expression.";}
    if t {panic "This is the 261st `panic` expression.";}
    // TODO: Error should be shown on all of the above `panic` expressions, not just the last one.
    //       This is very likely a common issue because of the `//TODO return all errors` in:
    //       sway-core/src/ir_generation/function.rs after the `self.compile_code_block` call.
}
