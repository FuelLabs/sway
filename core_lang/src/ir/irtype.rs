use super::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Type {
    Unit,
    Bool,
    Uint(u8),
    B256,
    String(u64),
    Struct(Aggregate),
    Enum(Aggregate),
}

impl Type {
    pub(crate) fn as_string(&self, context: &Context) -> String {
        let comma_sep_types_str = |agg_content: &AggregateContent| {
            agg_content
                .field_types
                .iter()
                .map(|ty| ty.as_string(context))
                .collect::<Vec<_>>()
                .join(", ")
        };

        match *self {
            Type::Unit => "()".into(),
            Type::Bool => "bool".into(),
            Type::Uint(nbits) => format!("u{}", nbits),
            Type::B256 => "b256".into(),
            Type::String(n) => format!("string<{}>", n),
            Type::Struct(agg) => {
                let agg_content = &context.aggregates[agg.0];
                format!("{{ {} }}", comma_sep_types_str(agg_content))
            }
            Type::Enum(agg) => {
                let agg_content = &context.aggregates[agg.0];
                format!("({})", comma_sep_types_str(agg_content))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub(crate) struct Aggregate(pub(crate) generational_arena::Index);

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct AggregateContent {
    pub(crate) field_types: Vec<Type>,
}

impl Aggregate {
    pub(crate) fn new(context: &mut Context, name: Option<String>, field_types: Vec<Type>) -> Self {
        let aggregate = Aggregate(context.aggregates.insert(AggregateContent { field_types }));
        if let Some(name) = name {
            context.aggregate_names.insert(name, aggregate);
        };
        aggregate
    }

    pub(crate) fn get_field_type(&self, context: &Context, indices: &[u64]) -> Option<Type> {
        indices.iter().fold(Some(Type::Struct(*self)), |ty, idx| {
            ty.map(|ty| match ty {
                Type::Struct(agg) => context.aggregates[agg.0]
                    .field_types
                    .get(*idx as usize)
                    .copied(),

                // Trying to index a non-aggregate.
                _otherwise => None,
            })
            .flatten()
        })
    }
}
