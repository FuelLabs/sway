use crate::{
    compute_dom_fronts,
    dominator::{compute_dom_tree, print_dot},
    print_dom_fronts, Context, Function, IrError,
};

pub fn promote_to_registers(context: &mut Context, function: &Function) -> Result<bool, IrError> {
    let dom_tree = compute_dom_tree(context, function);
    let dom_fronts = compute_dom_fronts(context, function, &dom_tree);
    print!(
        "{}\n{}\n{}",
        function.dot_cfg(context),
        print_dot(context, function.get_name(context), &dom_tree),
        print_dom_fronts(context, function.get_name(context), &dom_fronts),
    );
    Ok(true)
}
