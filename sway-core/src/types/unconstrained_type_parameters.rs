use crate::{Engines, TypeParameter};

pub(crate) trait UnconstrainedTypeParameters {
    fn type_parameter_is_unconstrained(
        &self,
        engines: &Engines,
        type_parameter: &TypeParameter,
    ) -> bool;

    fn type_parameter_is_constrained(
        &self,
        engines: &Engines,
        type_parameter: &TypeParameter,
    ) -> bool {
        !self.type_parameter_is_unconstrained(engines, type_parameter)
    }

    fn unconstrained_type_parameters<'a>(
        &self,
        engines: &Engines,
        type_parameters: &'a [TypeParameter],
    ) -> Vec<&'a TypeParameter> {
        let mut unconstrained = vec![];
        for type_param in type_parameters.iter() {
            if self.type_parameter_is_unconstrained(engines, type_param) {
                unconstrained.push(type_param);
            }
        }
        unconstrained
    }

    fn constrained_type_parameters<'a>(
        &self,
        engines: &Engines,
        type_parameters: &'a [TypeParameter],
    ) -> Vec<&'a TypeParameter> {
        let mut constrained = vec![];
        for type_param in type_parameters.iter() {
            if self.type_parameter_is_constrained(engines, type_param) {
                constrained.push(type_param);
            }
        }
        constrained
    }
}
