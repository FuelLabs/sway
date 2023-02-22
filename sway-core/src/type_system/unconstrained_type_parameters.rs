use crate::{Engines, TypeParameter, TypeParameters};

pub(crate) trait UnconstrainedTypeParameters {
    fn type_parameter_is_unconstrained(
        &self,
        engines: Engines<'_>,
        type_parameter: &TypeParameter,
    ) -> bool;

    fn unconstrained_type_parameters<'a>(
        &self,
        engines: Engines<'_>,
        type_parameters: &'a TypeParameters,
    ) -> Vec<&'a TypeParameter> {
        let mut unconstrained = vec![];
        for type_param in type_parameters.iter_excluding_self() {
            if self.type_parameter_is_unconstrained(engines, type_param) {
                unconstrained.push(type_param);
            }
        }
        unconstrained
    }
}
