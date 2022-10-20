use crate::TypeParameter;

pub(crate) trait UnconstrainedTypeParameters {
    fn type_parameter_is_unconstrained(&self, type_parameter: &TypeParameter) -> bool;
    // fn type_parameter_is_constrained(&self, type_parameter: &TypeParameter) -> bool;

    fn unconstrained_type_parameters<'a>(
        &self,
        type_parameters: &'a [TypeParameter],
    ) -> Vec<&'a TypeParameter> {
        let mut unconstrained = vec![];
        for type_param in type_parameters.iter() {
            if self.type_parameter_is_unconstrained(type_param) {
                unconstrained.push(type_param);
            }
        }
        unconstrained
    }

    // fn type_parameter_is_constrained(&self, type_parameter: &TypeParameter) -> bool {
    //     !self.type_parameter_is_unconstrained(type_parameter)
    // }

    // fn constrained_type_parameters<'a>(
    //     &self,
    //     type_parameters: &'a [TypeParameter],
    // ) -> Vec<&'a TypeParameter> {
    //     let mut constrained = vec![];
    //     for type_param in type_parameters.iter() {
    //         if self.type_parameter_is_constrained(type_param) {
    //             constrained.push(type_param);
    //         }
    //     }
    //     constrained
    // }
}
