//! Each item may have a list of attributes, each with a name  and a list of zero or more args.
//! Attributes may be specified more than once in which case we use the union of their args.
//!
//! E.g.,
//!
//! ```
//! #[foo(bar)]
//! #[foo(baz, xyzzy)]
//! ```
//!
//! is essentially equivalent to
//!
//! ```ignore
//! #[foo(bar, baz, xyzzy)]
//! ```
//!
//! and duplicates like
//!
//! ```ignore
//! #[foo(bar)]
//! #[foo(bar)]
//! ```
//!
//! are equivalent to
//!
//! ```ignore
//! #[foo(bar, bar)]
//! ```
//!
//! Attribute args can have values:
//!
//! ```ignore
//! #[foo(bar = "some value", baz = true)]
//! ```
//!
//! All attributes have the following common properties:
//! - targets: items that they can annotate. E.g., `#[inline]` can annotate only functions.
//! - multiplicity: if they can be applied multiple time on an item. E.g., `#[inline]` can
//!   be applied only once, but `#[cfg]` multiple times.
//! - arguments multiplicity: how many arguments they can have.
//! - arguments expectance: which arguments are expected and accepted as valid.
//!
//! All attribute arguments have the following common properties:
//! - value expectance: if they must have values specified.
//!
//! Individual arguments might impose their own additional constraints.

use indexmap::IndexMap;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{hash::Hash, sync::Arc};
use sway_ast::{attribute::AttributeHashKind, AttributeDecl, ImplItemParent, ItemImplItem, ItemKind, ItemTraitItem, Literal};
use sway_types::{Ident, Span, Spanned};

use crate::language::{Inline, Purity};

// TODO-IG!: Move these constants on the most appropriate place and remove pub. Review all this code below.
// The valid attribute strings related to storage and purity.
// pub const STORAGE_ATTRIBUTE_NAME: &str = "storage";
pub const STORAGE_ATTRIBUTE_NAME: &str = sway_types::constants::STORAGE_ATTRIBUTE_NAME;
pub const STORAGE_READ_ARG_NAME: &str = "read";
pub const STORAGE_WRITE_ARG_NAME: &str = "write";

// The valid attribute strings related to inline.
pub const INLINE_ATTRIBUTE_NAME: &str = "inline";
pub const INLINE_NEVER_ARG_NAME: &str = "never";
pub const INLINE_ALWAYS_ARG_NAME: &str = "always";

// The valid attribute strings related to documentation control.
pub const DOC_ATTRIBUTE_NAME: &str = "doc";

// The valid attribute strings related to documentation comments.
// Note that because "doc-comment" is not a valid identifier,
// doc-comment attributes cannot be declared in code.
// They are exclusively created by the compiler to denote
// doc comments, `///` and `//!`.
// pub const DOC_COMMENT_ATTRIBUTE_NAME: &str = "doc-comment";
pub const DOC_COMMENT_ATTRIBUTE_NAME: &str = sway_ast::attribute::DOC_COMMENT_ATTRIBUTE_NAME;

// The attribute used for Sway in-language unit tests.
pub const TEST_ATTRIBUTE_NAME: &str = "test";
pub const TEST_SHOULD_REVERT_ARG_NAME: &str = "should_revert";

// The valid attribute string used for payable functions.
pub const PAYABLE_ATTRIBUTE_NAME: &str = "payable";

// The valid attribute strings related to allow.
pub const ALLOW_ATTRIBUTE_NAME: &str = "allow";
pub const ALLOW_DEAD_CODE_ARG_NAME: &str = "dead_code";
pub const ALLOW_DEPRECATED_ARG_NAME: &str = "deprecated";

// The valid attribute strings related to conditional compilation.
pub const CFG_ATTRIBUTE_NAME: &str = "cfg";
pub const CFG_TARGET_ARG_NAME: &str = "target";
pub const CFG_PROGRAM_TYPE_ARG_NAME: &str = "program_type";

pub const DEPRECATED_ATTRIBUTE_NAME: &str = "deprecated";
pub const DEPRECATED_NOTE_ARG_NAME: &str = "note";

pub const FALLBACK_ATTRIBUTE_NAME: &str = "fallback";

// The list of known attributes.
pub const KNOWN_ATTRIBUTE_NAMES: &[&str] = &[
    STORAGE_ATTRIBUTE_NAME,
    DOC_ATTRIBUTE_NAME,
    DOC_COMMENT_ATTRIBUTE_NAME,
    TEST_ATTRIBUTE_NAME,
    INLINE_ATTRIBUTE_NAME,
    PAYABLE_ATTRIBUTE_NAME,
    ALLOW_ATTRIBUTE_NAME,
    CFG_ATTRIBUTE_NAME,
    DEPRECATED_ATTRIBUTE_NAME,
    FALLBACK_ATTRIBUTE_NAME,
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttributeArg {
    pub name: Ident,
    pub value: Option<Literal>,
    pub span: Span,
}

impl AttributeArg {
    pub fn is_allow_dead_code(&self) -> bool {
        self.name.as_str() == ALLOW_DEAD_CODE_ARG_NAME
    }
    pub fn is_allow_deprecated(&self) -> bool {
        self.name.as_str() == ALLOW_DEPRECATED_ARG_NAME
    }
}

impl Spanned for AttributeArg {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

// TODO: Currently we do not support arbitrary inner attributes.
//       Only compiler-generated `doc-comment` attributes for `//!`
//       can currently be inner attributes.
//       All of the below properties assume we are inspecting
//       outer attributes.
//       Extend the infrastructure for attribute properties to
//       support inner attributes, once we fully support them.
//       See: https://github.com/FuelLabs/sway/issues/6924

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    /// Attribute direction, taken from the enclosing [crate::AttributeDecl].
    /// All attributes within the same [crate::AttributeDecl] will have the
    /// same direction.
    pub direction: AttributeDirection,
    pub name: Ident,
    pub args: Vec<AttributeArg>,
    pub span: Span,
    pub kind: AttributeKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum AttributeDirection {
    Inner,
    Outer,
}

impl From<&AttributeHashKind> for AttributeDirection {
    fn from(value: &AttributeHashKind) -> Self {
        match value {
            AttributeHashKind::Inner(_) => Self::Inner,
            AttributeHashKind::Outer(_) => Self::Outer,
        }
    }
}

/// Defines the minimum and the maximum number of [AttributeArg]s
/// that an [Attribute] can have.
pub struct ArgsMultiplicity {
    min: usize,
    max: usize,
}

impl ArgsMultiplicity {
    pub fn zero() -> Self {
        Self { min: 0, max: 0 }
    }
    pub fn arbitrary() -> Self {
        Self { min: 0, max: usize::MAX }
    }
    pub fn exactly(num: usize) -> Self {
        Self { min: num, max: num }
    }
    pub fn at_least(num: usize) -> Self {
        Self { min: num, max: usize::MAX }
    }
    pub fn at_most(num: usize) -> Self {
        Self { min: 0, max: num }
    }
    pub fn between(min: usize, max: usize) -> Self {
        assert!(min <= max, "min must be less than or equal to max; min was {min}, max was {max}");
        Self { min, max }
    }
    pub fn contains(&self, value: usize) -> bool {
        self.min <= value && value <= self.max
    }
}

impl From<&ArgsMultiplicity> for (usize, usize) {
    fn from(value: &ArgsMultiplicity) -> Self {
        (value.min, value.max)
    }
}

/// Defines which [AttributeArg]s an [Attribute] expects.
pub enum ExpectedArgs {
    /// The [Attribute] does not expect any [AttributeArg]s.
    None,
    /// The [Attribute] can accept any argument. The `doc-comment`
    /// attribute is such an attribute - every documentation line
    /// becomes an [AttributeArg] and is accepted as valid.
    Any,
    /// An [AttributeArg::name] **must be** one from the provided list.
    /// If it is not, an error will be emitted.
    MustBeIn(Vec<&'static str>),
    /// An [AttributeArg::name] **should be** one from the provided list.
    /// If it is not, a warning will be emitted.
    ShouldBeIn(Vec<&'static str>),
}

impl ExpectedArgs {
    /// Returns expected argument names, if any specific names are
    /// expected, or an empty [Vec] if no names are expected or
    /// if the [Attribute] can accept any argument name.
    pub(crate) fn args_names(&self) -> Vec<&'static str> {
        match self {
            ExpectedArgs::None
            | ExpectedArgs::Any => vec![],
            ExpectedArgs::MustBeIn(expected_args)
            | ExpectedArgs::ShouldBeIn(expected_args) => expected_args.clone(),
        }
    }
}

/// Defines if [AttributeArg]s within the same [Attribute]
/// can or must have a value specified.
/// 
/// E.g., `#[attribute(arg = <value>)`.
///
/// We consider the expected types of individual values not to be
/// the part of the [AttributeArg]'s metadata. Final consumers of
/// the attribute must always check for the expected type and
/// emit an error if a wrong type is provided.
/// TODO-IG! Refer to exact error.
///
/// E.g., `#[cfg(target = 42)]` must emit an error during the
/// cfg-evaluation.
pub enum ArgsExpectValues {
    /// Each argument, if any, must have a value specified.
    /// Specified values can be of different types.
    ///
    /// E.g.: `#[cfg(target = "fuel", experimental_new_encoding = false)]`.
    Yes,
    /// None of the arguments can never have values specified, or the
    /// [Attribute] does not expect any arguments.
    ///
    /// E.g.: `#[storage(read, write)]`, `#[fallback]`.
    No,
    /// Each argument, if any, can have a value specified, but must not
    /// necessarily have one.
    ///
    /// E.g.: `#[some_attribute(arg_1 = 5, arg_2)]`.
    Maybe,
}

/// Kinds of attributes supported by the compiler.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum AttributeKind {
    /// Represents an [Attribute] unknown to the compiler.
    /// We generate warnings for such attributes but in
    /// general support them and pass them to the typed
    /// tree. This allows third-party static analysis to
    /// utilized proprietary attributes and inspect them
    /// in the typed tree.
    Unknown,
    // TODO-IG!: Remove `doc` attribute.
    Doc,
    DocComment,
    Storage,
    Inline,
    Test,
    Payable,
    Allow,
    Cfg,
    Deprecated,
    Fallback,
}

/// Denotes if an [ItemTraitItem] belongs to an ABI or to a trait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraitItemParent {
    Abi,
    Trait,
}

/// Denotes if a [sway_ast::TypeField] belongs to a struct or to an enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructOrEnumField {
    StructField,
    EnumField,
}

impl AttributeKind {
    pub fn from_attribute_name(name: &str) -> Self {
        match name {
            DOC_ATTRIBUTE_NAME => AttributeKind::Doc,
            DOC_COMMENT_ATTRIBUTE_NAME => AttributeKind::DocComment,
            STORAGE_ATTRIBUTE_NAME => AttributeKind::Storage,
            INLINE_ATTRIBUTE_NAME => AttributeKind::Inline,
            TEST_ATTRIBUTE_NAME => AttributeKind::Test,
            PAYABLE_ATTRIBUTE_NAME => AttributeKind::Payable,
            ALLOW_ATTRIBUTE_NAME => AttributeKind::Allow,
            CFG_ATTRIBUTE_NAME => AttributeKind::Cfg,
            DEPRECATED_ATTRIBUTE_NAME => AttributeKind::Deprecated,
            FALLBACK_ATTRIBUTE_NAME => AttributeKind::Fallback,
            _ => AttributeKind::Unknown,
        }
    }

    /// True if multiple attributes of a this [AttributeKind] can
    /// annotate the same item at the same time. E.g., the `inline`
    /// attribute can be applied only once on an item and does not
    /// allow multiple, while the `cfg` attribute can be applied
    /// arbitrary many times.
    ///
    /// Currently we assume that the multiplicity does not depend
    /// on the annotated item, but is an inherent property of
    /// the [AttributeKind].
    pub fn allows_multiple(&self) -> bool {
        use AttributeKind::*;
        match self {
            Unknown => true,
            Doc => true,
            DocComment => true,
            Storage => false,
            Inline => false,
            Test => false,
            Payable => false,
            Allow => true,
            Cfg => true,
            Deprecated => false,
            Fallback => false,
        }
    }
}

impl Attribute {
    pub fn is_doc_comment(&self) -> bool {
        self.kind == AttributeKind::DocComment
    }

    pub fn is_inner(&self) -> bool {
        self.direction == AttributeDirection::Inner
    }

    pub fn is_outer(&self) -> bool {
        self.direction == AttributeDirection::Outer
    }

    pub(crate) fn args_multiplicity(&self) -> ArgsMultiplicity {
        use AttributeKind::*;
        use ArgsMultiplicity as Multiplicity;
        match self.kind {
            Unknown => Multiplicity::arbitrary(),
            Doc => Multiplicity::exactly(1),
            // Each `doc-comment` attribute contains exactly one argument
            // whose name is the actual documentation text and whose value is `None`.
            // Thus, we expect exactly one argument.
            DocComment => Multiplicity::exactly(1),
            // `storage(read, write)`.
            Storage => Multiplicity::between(1, 2),
            // `inline(always)`.
            Inline => Multiplicity::exactly(1),
            // `test`, `test(should_revert)`.
            Test => Multiplicity::at_most(1),
            Payable => Multiplicity::zero(),
            Allow => Multiplicity::at_least(1),
            Cfg => Multiplicity::exactly(1),
            // `deprecated`, `deprecated(note = "note")`.
            Deprecated => Multiplicity::at_most(1),
            Fallback => Multiplicity::zero(),
        }
    }

    pub(crate) fn can_have_arguments(&self) -> bool {
        let args_multiplicity = self.args_multiplicity();
        args_multiplicity.min != 0 || args_multiplicity.max != 0
    }

    pub(crate) fn expected_args(&self) -> ExpectedArgs {
        use AttributeKind::*;
        use ExpectedArgs::*;
        match self.kind {
            Unknown => Any,
            Doc => Any,
            DocComment => Any,
            Storage => MustBeIn(vec![STORAGE_READ_ARG_NAME, STORAGE_WRITE_ARG_NAME]),
            Inline => MustBeIn(vec![INLINE_ALWAYS_ARG_NAME, INLINE_NEVER_ARG_NAME]),
            Test => MustBeIn(vec![TEST_SHOULD_REVERT_ARG_NAME]),
            Payable => None,
            Allow => ShouldBeIn(vec![ALLOW_DEAD_CODE_ARG_NAME, ALLOW_DEPRECATED_ARG_NAME]),
            Cfg => {
                let mut args = vec![
                    // Arguments, ordered alphabetically.
                    CFG_PROGRAM_TYPE_ARG_NAME,
                    CFG_TARGET_ARG_NAME,
                ];
                args.extend(sway_features::CFG.iter().sorted());
                MustBeIn(args)
            },
            Deprecated => MustBeIn(vec![DEPRECATED_NOTE_ARG_NAME]),
            Fallback => None,
        }
    }

    pub(crate) fn args_expect_values(&self) -> ArgsExpectValues {
        use AttributeKind::*;
        use ArgsExpectValues::*;
        match self.kind {
            Unknown => Maybe,
            Doc => No,
            // The actual documentation line is in the name of the attribute.
            DocComment => No,
            Storage => No,
            Inline => No,
            // `test(should_revert)`, `test(should_revert = "18446744073709486084")`.
            Test => Maybe,
            Payable => No,
            Allow => No,
            Cfg => Yes,
            // `deprecated(note = "note")`.
            Deprecated => Yes,
            Fallback => No,
        }
    }

    pub(crate) fn can_annotate_module_kind(&self) -> bool {
        use AttributeKind::*;
        match self.kind {
            Unknown => false,
            Doc => false,
            DocComment => self.direction == AttributeDirection::Inner,
            Storage => false,
            Inline => false,
            Test => false,
            Payable => false,
            Allow => false,
            Cfg => false,
            Deprecated => false,
            Fallback => false,
        }
    }

    pub(crate) fn can_annotate_item_kind(&self, item_kind: &ItemKind) -> bool {
        // TODO-IG!: Check the comments for inner/outer.
        // TODO: We assume outer annotation here. A separate check that emits not-implemented
        //       error will be done for all inner attributes, as well as the check that
        //       inner doc comments are properly placed. Until we fully support inner
        //       attributes, this approach is sufficient.
        //       See: https://github.com/FuelLabs/sway/issues/6924

        // TODO: Currently we do not support any attributes on `mod`s, including doc comments.
        //       See: https://github.com/FuelLabs/sway/issues/6879
        //       See: https://github.com/FuelLabs/sway/issues/6925

        // We accept all attribute kinds on the `ItemKind::Error`.
        if matches!(item_kind, ItemKind::Error(..)) {
            return true;
        }

        use AttributeKind::*;
        match self.kind {
            Unknown => !matches!(item_kind, ItemKind::Submodule(_)),
            Doc => false,
            // We allow doc comments on all items including `storage` and `configurable`.
            DocComment => self.direction == AttributeDirection::Outer && !matches!(item_kind, ItemKind::Submodule(_)),
            Storage => matches!(item_kind, ItemKind::Fn(_)),
            Inline => matches!(item_kind, ItemKind::Fn(_)),
            Test => matches!(item_kind, ItemKind::Fn(_)),
            Payable => false,
            Allow => !matches!(item_kind, ItemKind::Submodule(_)),
            Cfg => !matches!(item_kind, ItemKind::Submodule(_)),
            // `deprecated` is currently implemented only for structs.
            Deprecated => matches!(item_kind, ItemKind::Struct(_)),
            Fallback => matches!(item_kind, ItemKind::Fn(_)),
        }
    }

    pub(crate) fn can_annotate_struct_or_enum_field(&self, _struct_or_enum_field: StructOrEnumField) -> bool {
        use AttributeKind::*;
        match self.kind {
            Unknown => true,
            Doc => false,
            DocComment => self.direction == AttributeDirection::Outer,
            Storage => false,
            Inline => false,
            Test => false,
            Payable => false,
            Allow => true,
            Cfg => true,
            // `deprecated` is currently implemented only for structs.
            Deprecated => false,
            Fallback => false,
        }
    }

    pub(crate) fn can_annotate_abi_or_trait_item(&self, item: &ItemTraitItem, parent: TraitItemParent) -> bool {
        use AttributeKind::*;
        match self.kind {
            Unknown => true,
            Doc => false,
            DocComment => self.direction == AttributeDirection::Outer,
            Storage => matches!(item, ItemTraitItem::Fn(..)),
            // Functions in the trait or ABI interface surface cannot be marked as inlined
            // because they don't have implementation.
            Inline => false,
            Test => false,
            Payable => parent == TraitItemParent::Abi && matches!(item, ItemTraitItem::Fn(..)),
            Allow => true,
            Cfg => true,
            // `deprecated` is currently implemented only for structs.
            Deprecated => false,
            Fallback => false,
        }
    }

    pub(crate) fn can_annotate_impl_item(&self, item: &ItemImplItem, parent: ImplItemParent) -> bool {
        use AttributeKind::*;
        match self.kind {
            Unknown => true,
            Doc => false,
            DocComment => self.direction == AttributeDirection::Outer,
            Storage => matches!(item, ItemImplItem::Fn(..)),
            Inline => matches!(item, ItemImplItem::Fn(..)),
            Test => false,
            Payable => parent == ImplItemParent::Contract,
            Allow => true,
            Cfg => true,
            // `deprecated` is currently implemented only for structs.
            Deprecated => false,
            Fallback => false,
        }
    }

    pub(crate) fn can_annotate_abi_or_trait_item_fn(&self, abi_or_trait_item: TraitItemParent) -> bool {
        use AttributeKind::*;
        match self.kind {
            Unknown => true,
            Doc => false,
            DocComment => self.direction == AttributeDirection::Outer,
            Storage => true,
            Inline => true,
            Test => false,
            Payable => abi_or_trait_item == TraitItemParent::Abi,
            Allow => true,
            Cfg => true,
            // `deprecated` is currently implemented only for structs.
            Deprecated => false,
            Fallback => false,
        }
    }

    pub(crate) fn can_annotate_storage_entry(&self) -> bool {
        use AttributeKind::*;
        match self.kind {
            Unknown => true,
            Doc => false,
            DocComment => self.direction == AttributeDirection::Outer,
            Storage => false,
            Inline => false,
            Test => false,
            Payable => false,
            Allow => true,
            Cfg => true,
            // `deprecated` is currently implemented only for structs.
            Deprecated => false,
            Fallback => false,
        }
    }

    pub(crate) fn can_annotate_configurable_field(&self) -> bool {
        use AttributeKind::*;
        match self.kind {
            Unknown => true,
            Doc => false,
            DocComment => self.direction == AttributeDirection::Outer,
            Storage => false,
            Inline => false,
            Test => false,
            Payable => false,
            Allow => true,
            Cfg => true,
            // `deprecated` is currently implemented only for structs.
            Deprecated => false,
            Fallback => false,
        }
    }

    // TODO-IG!: Comment.
    pub(crate) fn can_only_annotate_help(&self, target_friendly_name: &str) -> Vec<&'static str> {
        // Using strings to identify targets is not ideal, but there
        // is no real need for a more complex and type-safe identification here.
        use AttributeKind::*;
        let help = match self.kind {
            Unknown => vec![],
            Doc => vec![],
            DocComment => match self.direction {
                AttributeDirection::Inner => vec![
                    "Inner doc comments (`//!`) can only document modules and must be",
                    "at the beginning of the module file, before the module kind.",
                ],
                AttributeDirection::Outer => if target_friendly_name.starts_with("module kind") {
                    vec![
                        "To document modules, use inner doc comments (`//!`). E.g.:",
                        "//! This doc comment documents a module.",
                    ]
                } else {
                    vec![]
                },
            },
            Storage => {
                if target_friendly_name == "function signature" {
                    vec![
                        "\"storage\" attribute can only annotate functions that have an implementation.",
                        "Function signatures in ABI and trait declarations do not have implementations.",
                    ]
                } else {
                    vec![
                        "\"storage\" attribute can only annotate functions.",
                    ]
                }
            },
            Inline => vec!["\"inline\" attribute can only annotate functions."],
            Test => vec!["\"test\" attribute can only annotate module functions."],
            Payable => vec![
                "\"payable\" attribute can only annotate:",
                "  - ABI function signatures and their implementations in contracts,",
                "  - provided ABI functions.",
            ],
            Allow => vec![],
            Cfg => vec![],
            Deprecated => vec!["\"deprecated\" attribute is currently implemented only for struct declarations."],
            Fallback => vec!["\"fallback\" attribute can only annotate module functions in a contract module."],
        };

        if help.is_empty() && target_friendly_name.starts_with("module kind") {
            vec!["Annotating module kinds (contract, script, predicate, or library) is currently not implemented."]
        } else {
            help
        }
    }
}

// TODO-IG!: Rename to Attributes and document. Losing the info about enclosing [AttributeDecl].
/// Stores the attributes associated with the type.
// TODO-IG!: Comment why not map.
// TODO-IG!: Fast access to deprecated.
// TODO-IG!: Comment only once for last-wins approach.
#[derive(Default, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttributesMap(Arc<Vec<Attribute>>);

impl AttributesMap {
    /// Creates a new [AttributesMap].
    pub fn new(attribute_decls: &[AttributeDecl]) -> AttributesMap {
        let mut attributes: Vec<Attribute> = vec![];
        for attr_decl in attribute_decls {
            let attrs = attr_decl.attribute.get().into_iter();
            for attr in attrs {
                let name = attr.name.as_str();
                let args = attr
                    .args
                    .as_ref()
                    .map(|parens| {
                        parens
                            .get()
                            .into_iter()
                            .cloned()
                            .map(|arg| AttributeArg {
                                name: arg.name.clone(),
                                value: arg.value.clone(),
                                span: arg.span(),
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                let attribute = Attribute {
                    direction: (&attr_decl.hash_kind).into(),
                    name: attr.name.clone(),
                    args,
                    span: attr_decl.span(),
                    kind: AttributeKind::from_attribute_name(name),
                };

                attributes.push(attribute);
            }
        }
        AttributesMap(Arc::new(attributes))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the first attribute, ordered by span, or None if there are no attributes.
    pub fn first(&self) -> Option<&Attribute> {
        self.0.first()
    }

    pub fn known_attribute_names(&self) -> &'static [&'static str] {
        KNOWN_ATTRIBUTE_NAMES
    }

    pub fn all(&self) -> impl Iterator<Item = &Attribute> {
        self.0.iter()
    }

    pub fn all_by_kind<F>(&self, predicate: F) -> IndexMap<AttributeKind, Vec<&Attribute>>
    where
        F: Fn(&&Attribute) -> bool
    {
        let mut result = IndexMap::<_, Vec<&Attribute>>::new();
        for attr in self.0.iter().filter(predicate) {
            result.entry(attr.kind).or_default().push(attr);
        }
        result
    }

    pub fn of_kind(&self, kind: AttributeKind) -> impl Iterator<Item = &Attribute> {
        self.0.iter().filter(move |attr| attr.kind == kind)
    }

    pub fn has_any_of_kind(&self, kind: AttributeKind) -> bool {
        self.of_kind(kind).any(|_| true)
    }

    pub fn unknown(&self) -> impl Iterator<Item = &Attribute> {
        self.0.iter().filter(|attr| attr.kind == AttributeKind::Unknown)
    }

    /// Returns true if the [AttributesMap] contains any `#[allow]` [Attribute]
    /// containing `dead_code` [AttributeArg].
    pub fn has_allow_dead_code(&self) -> bool {
        self.has_allow(|arg| arg.is_allow_dead_code())
    }

    pub fn has_allow_deprecated(&self) -> bool {
        self.has_allow(|arg| arg.is_allow_deprecated())
    }

    fn has_allow(&self, arg_filter: impl Fn(&AttributeArg) -> bool) -> bool {
        self
            .of_kind(AttributeKind::Allow)
            .flat_map(|attribute| &attribute.args)
            .any(arg_filter)
    }

    /// Returns the value of the `#[inline]` [Attribute], or `None` if the
    /// [AttributesMap] does not contain any `#[inline]` attributes.
    pub fn inline(&self) -> Option<Inline> {
        // `inline` attribute can be applied only once (`AttributeMultiplicity::Single`),
        // and can have exactly one argument, otherwise an error is emitted.
        // Last-wins approach.
        match self
            .of_kind(AttributeKind::Inline)
            .last()?
            .args
            .last()?
            .name
            .as_str()
        {
            INLINE_NEVER_ARG_NAME => Some(Inline::Never),
            INLINE_ALWAYS_ARG_NAME => Some(Inline::Always),
            _ => None,
        }
    }

    /// Returns the value of the `#[storage]` [Attribute], or [Purity::Pure] if the
    /// [AttributesMap] does not contain any `#[storage]` attributes.
    pub fn purity(&self) -> Purity {
        // `storage` attribute can be applied only once (`AttributeMultiplicity::Single`).
        // Last-wins approach.
        let Some(storage_attr) = self.of_kind(AttributeKind::Storage).last() else {
            return Purity::Pure;
        };

        let mut purity = Purity::Pure;

        let mut add_impurity = |new_impurity, counter_impurity| {
            if purity == Purity::Pure {
                purity = new_impurity;
            } else if purity == counter_impurity {
                purity = Purity::ReadsWrites;
            }
        };

        for arg in storage_attr.args.iter()
        {
            match arg.name.as_str() {
                STORAGE_READ_ARG_NAME => add_impurity(Purity::Reads, Purity::Writes),
                STORAGE_WRITE_ARG_NAME => add_impurity(Purity::Writes, Purity::Reads),
                _ => {}
            }
        }

        purity
    }

    /// Returns the `#[deprecated]` [Attribute], or `None` if the
    /// [AttributesMap] does not contain any `#[deprecated]` attributes.
    pub fn deprecated(&self) -> Option<&Attribute> {
        // Last-wins approach.
        self
            .of_kind(AttributeKind::Deprecated)
            .last()
    }
}

pub struct AllowDeprecatedEnterToken {
    diff: i32,
}

#[derive(Default)]
pub struct AllowDeprecatedState {
    allowed: u32,
}
impl AllowDeprecatedState {
    pub(crate) fn enter(&mut self, attributes: AttributesMap) -> AllowDeprecatedEnterToken {
        if attributes.has_allow_deprecated() {
            self.allowed += 1;

            AllowDeprecatedEnterToken { diff: -1 }
        } else {
            AllowDeprecatedEnterToken { diff: 0 }
        }
    }

    pub(crate) fn exit(&mut self, token: AllowDeprecatedEnterToken) {
        self.allowed = self.allowed.saturating_add_signed(token.diff);
    }

    pub(crate) fn is_allowed(&self) -> bool {
        self.allowed > 0
    }
}
