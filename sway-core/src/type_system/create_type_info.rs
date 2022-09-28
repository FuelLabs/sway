use crate::TypeInfo;

pub(crate) trait CreateTypeInfo {
    fn create_type_info(&self) -> TypeInfo;
}
