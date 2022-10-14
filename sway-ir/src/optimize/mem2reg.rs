use crate::{
    dominator::{compute_dom_tree, print_dot},
    Context, Function, IrError,
};

pub fn promote_to_registers(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    let dom_tree = compute_dom_tree(context, function);
    print!(
        "{}\n{}",
        function.dot_cfg(context),
        print_dot(context, function.get_name(context), &dom_tree)
    );
    Ok(true)
}
