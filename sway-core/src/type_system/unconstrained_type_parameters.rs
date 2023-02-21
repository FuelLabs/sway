use crate::{Engines, TypeParam};

pub(crate) trait UnconstrainedTypeParameters {
    fn type_parameter_is_unconstrained(
        &self,
        engines: Engines<'_>,
        type_parameter: &TypeParam,
    ) -> bool;

    fn unconstrained_type_parameters<'a>(
        &self,
        engines: Engines<'_>,
        type_parameters: &'a [TypeParam],
    ) -> Vec<&'a TypeParam> {
        let mut unconstrained = vec![];
        for type_param in type_parameters.iter() {
            if self.type_parameter_is_unconstrained(engines, type_param) {
                unconstrained.push(type_param);
            }
        }
        unconstrained
    }
}
