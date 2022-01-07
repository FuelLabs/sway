use crate::context::Context;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Type {
    Unit,
    Bool,
    Uint(u8),
    B256,
    String(u64),
    Array(Aggregate),
    Union(Aggregate),
    Struct(Aggregate),

    Contract,
    ContractCaller(AbiInstance),
}

// XXX I've added Array as using Aggregate in the hope ExtractValue could be used just like with
// struct aggregates, but it turns out we need ExtractElement (which takes an index Value).  So
// Aggregate can be a 'struct' or 'array' but you only ever use them with Struct and Array types
// and with ExtractValue and ExtractElement... so they're orthogonal and we can simplify aggregate
// again to be only for structs.
//
// But also to keep Type as Copy we need to put the Array meta into another copy type (rather than
// recursing with Box<Type>, effectively a different Aggregate.  This could be OK though, still
// simpler that what we have here.

impl Type {
    pub fn as_string(&self, context: &Context) -> String {
        let sep_types_str = |agg_content: &AggregateContent, sep: &str| {
            agg_content
                .field_types()
                .iter()
                .map(|ty| ty.as_string(context))
                .collect::<Vec<_>>()
                .join(sep)
        };

        match self {
            Type::Unit => "()".into(),
            Type::Bool => "bool".into(),
            Type::Uint(nbits) => format!("u{}", nbits),
            Type::B256 => "b256".into(),
            Type::String(n) => format!("string<{}>", n),
            Type::Array(agg) => {
                let (ty, cnt) = &context.aggregates[agg.0].array_type();
                format!("[{}; {}]", ty.as_string(context), cnt)
            }
            Type::Union(agg) => {
                let agg_content = &context.aggregates[agg.0];
                format!("{{ {} }}", sep_types_str(agg_content, " | "))
            }
            Type::Struct(agg) => {
                let agg_content = &context.aggregates[agg.0];
                format!("{{ {} }}", sep_types_str(agg_content, ", "))
            }
            Type::Contract => "contract".into(),
            Type::ContractCaller(_) => "TODO CONTRACT CALLER".into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Aggregate(pub generational_arena::Index);

#[derive(Debug, Clone, PartialEq)]
pub enum AggregateContent {
    ArrayType(Type, u64),
    FieldTypes(Vec<Type>),
}

impl Aggregate {
    pub fn new_struct(context: &mut Context, name: Option<String>, field_types: Vec<Type>) -> Self {
        let aggregate = Aggregate(
            context
                .aggregates
                .insert(AggregateContent::FieldTypes(field_types)),
        );
        if let Some(name) = name {
            context.aggregate_names.insert(name, aggregate);
        };
        aggregate
    }

    pub fn new_array(context: &mut Context, element_type: Type, count: u64) -> Self {
        Aggregate(
            context
                .aggregates
                .insert(AggregateContent::ArrayType(element_type, count)),
        )
    }

    pub fn get_field_type(&self, context: &Context, indices: &[u64]) -> Option<Type> {
        indices.iter().fold(Some(Type::Struct(*self)), |ty, idx| {
            ty.map(|ty| match ty {
                Type::Struct(agg) => context.aggregates[agg.0]
                    .field_types()
                    .get(*idx as usize)
                    .cloned(),

                // Trying to index a non-aggregate.
                _otherwise => None,
            })
            .flatten()
        })
    }

    pub fn get_elem_type(&self, context: &Context) -> Option<Type> {
        if let AggregateContent::ArrayType(ty, _) = context.aggregates[self.0] {
            Some(ty)
        } else {
            None
        }
    }
}

impl AggregateContent {
    pub fn field_types(&self) -> &Vec<Type> {
        match self {
            AggregateContent::FieldTypes(types) => types,
            AggregateContent::ArrayType(..) => panic!("Getting field types from array aggregate."),
        }
    }

    pub fn array_type(&self) -> (&Type, &u64) {
        match self {
            AggregateContent::FieldTypes(..) => panic!("Getting array type from fields aggregate."),
            AggregateContent::ArrayType(ty, cnt) => (ty, cnt),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AbiInstance(pub generational_arena::Index);

#[derive(Debug, Clone, PartialEq)]
pub struct AbiInstanceContent {
    pub name: Vec<String>,
    pub address: String,
}

impl AbiInstance {
    pub fn new(
        context: &mut Context,
        mut name_prefixes: Vec<String>,
        name_suffix: String,
        address: String,
    ) -> Self {
        name_prefixes.push(name_suffix);
        AbiInstance(context.abi_instances.insert(AbiInstanceContent {
            name: name_prefixes,
            address,
        }))
    }
}
