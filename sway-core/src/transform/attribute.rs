//! Each item may have a list of attributes, each with a name and a list of zero or more args.
//! Attributes may be specified more than once in which case we use the union of their args.
//!
//! E.g.,
//!
//! ```ignore
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
use sway_ast::{
    attribute::*, AttributeDecl, ImplItemParent, ItemImplItem, ItemKind, ItemTraitItem, Literal,
};
use sway_error::{
    convert_parse_tree_error::ConvertParseTreeError,
    handler::{ErrorEmitted, Handler},
};
use sway_features::Feature;
use sway_types::{Ident, Span, Spanned};

use crate::language::{Inline, Purity, Trace};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AttributeArg {
    pub name: Ident,
    pub value: Option<Literal>,
    pub span: Span,
}

impl AttributeArg {
    /// Returns a mandatory [String] value from `self`,
    /// or an error if the value does not exist or is not of type [String].
    ///
    /// `attribute` is the the parent [Attribute] of `self`.
    pub fn get_string(
        &self,
        handler: &Handler,
        attribute: &Attribute,
    ) -> Result<&String, ErrorEmitted> {
        match &self.value {
            Some(literal) => match literal {
                Literal::String(lit_string) => Ok(&lit_string.parsed),
                _ => Err(handler.emit_err(
                    ConvertParseTreeError::InvalidAttributeArgValueType {
                        span: literal.span(),
                        arg: self.name.clone(),
                        expected_type: "str",
                        received_type: literal.friendly_type_name(),
                    }
                    .into(),
                )),
            },
            None => Err(handler.emit_err(
                ConvertParseTreeError::InvalidAttributeArgExpectsValue {
                    attribute: attribute.name.clone(),
                    arg: (&self.name).into(),
                    value_span: None,
                }
                .into(),
            )),
        }
    }

    /// Returns an optional [String] value from `self`,
    /// or an error if the value exists but is not of type [String].
    pub fn get_string_opt(&self, handler: &Handler) -> Result<Option<&String>, ErrorEmitted> {
        match &self.value {
            Some(literal) => match literal {
                Literal::String(lit_string) => Ok(Some(&lit_string.parsed)),
                _ => Err(handler.emit_err(
                    ConvertParseTreeError::InvalidAttributeArgValueType {
                        span: literal.span(),
                        arg: self.name.clone(),
                        expected_type: "str",
                        received_type: literal.friendly_type_name(),
                    }
                    .into(),
                )),
            },
            None => Ok(None),
        }
    }

    /// Returns a mandatory `bool` value from `self`,
    /// or an error if the value does not exist or is not of type `bool`.
    ///
    /// `attribute` is the the parent [Attribute] of `self`.
    pub fn get_bool(&self, handler: &Handler, attribute: &Attribute) -> Result<bool, ErrorEmitted> {
        match &self.value {
            Some(literal) => match literal {
                Literal::Bool(lit_bool) => Ok(lit_bool.kind.into()),
                _ => Err(handler.emit_err(
                    ConvertParseTreeError::InvalidAttributeArgValueType {
                        span: literal.span(),
                        arg: self.name.clone(),
                        expected_type: "bool",
                        received_type: literal.friendly_type_name(),
                    }
                    .into(),
                )),
            },
            None => Err(handler.emit_err(
                ConvertParseTreeError::InvalidAttributeArgExpectsValue {
                    attribute: attribute.name.clone(),
                    arg: (&self.name).into(),
                    value_span: None,
                }
                .into(),
            )),
        }
    }

    pub fn is_allow_dead_code(&self) -> bool {
        self.name.as_str() == ALLOW_DEAD_CODE_ARG_NAME
    }

    pub fn is_allow_deprecated(&self) -> bool {
        self.name.as_str() == ALLOW_DEPRECATED_ARG_NAME
    }

    pub fn is_cfg_target(&self) -> bool {
        self.name.as_str() == CFG_TARGET_ARG_NAME
    }

    pub fn is_cfg_program_type(&self) -> bool {
        self.name.as_str() == CFG_PROGRAM_TYPE_ARG_NAME
    }

    pub fn is_cfg_experimental(&self) -> bool {
        Feature::CFG.contains(&self.name.as_str())
    }

    pub fn is_deprecated_note(&self) -> bool {
        self.name.as_str() == DEPRECATED_NOTE_ARG_NAME
    }

    pub fn is_test_should_revert(&self) -> bool {
        self.name.as_str() == TEST_SHOULD_REVERT_ARG_NAME
    }

    pub fn is_error_message(&self) -> bool {
        self.name.as_str() == ERROR_M_ARG_NAME
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
        Self {
            min: 0,
            max: usize::MAX,
        }
    }
    pub fn exactly(num: usize) -> Self {
        Self { min: num, max: num }
    }
    pub fn at_least(num: usize) -> Self {
        Self {
            min: num,
            max: usize::MAX,
        }
    }
    pub fn at_most(num: usize) -> Self {
        Self { min: 0, max: num }
    }
    pub fn between(min: usize, max: usize) -> Self {
        assert!(
            min <= max,
            "min must be less than or equal to max; min was {min}, max was {max}"
        );
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
            ExpectedArgs::None | ExpectedArgs::Any => vec![],
            ExpectedArgs::MustBeIn(expected_args) | ExpectedArgs::ShouldBeIn(expected_args) => {
                expected_args.clone()
            }
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
/// the attribute will check for the expected type and emit an error
/// if a wrong type is provided.
///
/// E.g., `#[cfg(target = 42)]` will emit an error during the
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
    DocComment,
    Storage,
    Inline,
    Test,
    Payable,
    Allow,
    Cfg,
    Deprecated,
    Fallback,
    ErrorType,
    Error,
    Trace,
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
            DOC_COMMENT_ATTRIBUTE_NAME => AttributeKind::DocComment,
            STORAGE_ATTRIBUTE_NAME => AttributeKind::Storage,
            INLINE_ATTRIBUTE_NAME => AttributeKind::Inline,
            TEST_ATTRIBUTE_NAME => AttributeKind::Test,
            PAYABLE_ATTRIBUTE_NAME => AttributeKind::Payable,
            ALLOW_ATTRIBUTE_NAME => AttributeKind::Allow,
            CFG_ATTRIBUTE_NAME => AttributeKind::Cfg,
            DEPRECATED_ATTRIBUTE_NAME => AttributeKind::Deprecated,
            FALLBACK_ATTRIBUTE_NAME => AttributeKind::Fallback,
            ERROR_TYPE_ATTRIBUTE_NAME => AttributeKind::ErrorType,
            ERROR_ATTRIBUTE_NAME => AttributeKind::Error,
            TRACE_ATTRIBUTE_NAME => AttributeKind::Trace,
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
            DocComment => true,
            Storage => false,
            Inline => false,
            Test => false,
            Payable => false,
            Allow => true,
            Cfg => true,
            Deprecated => false,
            Fallback => false,
            ErrorType => false,
            Error => false,
            Trace => false,
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
        use ArgsMultiplicity as Multiplicity;
        use AttributeKind::*;
        match self.kind {
            Unknown => Multiplicity::arbitrary(),
            // Each `doc-comment` attribute contains exactly one argument
            // whose name is the actual documentation text and whose value is `None`.
            // Thus, we expect exactly one argument.
            DocComment => Multiplicity::exactly(1),
            // `storage(read, write)`.
            Storage => Multiplicity::between(1, 2),
            // `inline(never)` or `inline(always)`.
            Inline => Multiplicity::exactly(1),
            // `test`, `test(should_revert)`.
            Test => Multiplicity::at_most(1),
            Payable => Multiplicity::zero(),
            Allow => Multiplicity::at_least(1),
            Cfg => Multiplicity::exactly(1),
            // `deprecated`, `deprecated(note = "note")`.
            Deprecated => Multiplicity::at_most(1),
            Fallback => Multiplicity::zero(),
            ErrorType => Multiplicity::zero(),
            Error => Multiplicity::exactly(1),
            // `trace(never)` or `trace(always)`.
            Trace => Multiplicity::exactly(1),
        }
    }

    pub(crate) fn check_args_multiplicity(&self, handler: &Handler) -> Result<(), ErrorEmitted> {
        if !self.args_multiplicity().contains(self.args.len()) {
            Err(handler.emit_err(
                ConvertParseTreeError::InvalidAttributeArgsMultiplicity {
                    span: if self.args.is_empty() {
                        self.name.span()
                    } else {
                        Span::join(
                            self.args.first().unwrap().span(),
                            &self.args.last().unwrap().span,
                        )
                    },
                    attribute: self.name.clone(),
                    args_multiplicity: (&self.args_multiplicity()).into(),
                    num_of_args: self.args.len(),
                }
                .into(),
            ))
        } else {
            Ok(())
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
                args.extend(Feature::CFG.iter().sorted());
                MustBeIn(args)
            }
            Deprecated => MustBeIn(vec![DEPRECATED_NOTE_ARG_NAME]),
            Fallback => None,
            ErrorType => None,
            Error => MustBeIn(vec![ERROR_M_ARG_NAME]),
            Trace => MustBeIn(vec![TRACE_ALWAYS_ARG_NAME, TRACE_NEVER_ARG_NAME]),
        }
    }

    pub(crate) fn args_expect_values(&self) -> ArgsExpectValues {
        use ArgsExpectValues::*;
        use AttributeKind::*;
        match self.kind {
            Unknown => Maybe,
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
            ErrorType => No,
            // `error(msg = "msg")`.
            Error => Yes,
            Trace => No,
        }
    }

    pub(crate) fn can_annotate_module_kind(&self) -> bool {
        use AttributeKind::*;
        match self.kind {
            Unknown => false,
            DocComment => self.direction == AttributeDirection::Inner,
            Storage => false,
            Inline => false,
            Test => false,
            Payable => false,
            Allow => false,
            Cfg => false,
            // TODO: Change to true once https://github.com/FuelLabs/sway/issues/6942 is implemented.
            //       Deprecating the module kind will mean deprecating all its items.
            Deprecated => false,
            Fallback => false,
            ErrorType => false,
            Error => false,
            Trace => false,
        }
    }

    pub(crate) fn can_annotate_item_kind(&self, item_kind: &ItemKind) -> bool {
        // TODO: Except for `DocComment`, we assume outer annotation here.
        //       A separate check emits not-implemented error for all inner attributes.
        //       Until we fully support inner attributes, this approach is sufficient.
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
            // We allow doc comments on all items including `storage` and `configurable`.
            DocComment => {
                self.direction == AttributeDirection::Outer
                    && !matches!(item_kind, ItemKind::Submodule(_))
            }
            Storage => matches!(item_kind, ItemKind::Fn(_)),
            Inline => matches!(item_kind, ItemKind::Fn(_)),
            Test => matches!(item_kind, ItemKind::Fn(_)),
            Payable => false,
            Allow => !matches!(item_kind, ItemKind::Submodule(_)),
            Cfg => !matches!(item_kind, ItemKind::Submodule(_)),
            // TODO: Adapt once https://github.com/FuelLabs/sway/issues/6942 is implemented.
            Deprecated => match item_kind {
                ItemKind::Submodule(_) => false,
                ItemKind::Use(_) => false,
                ItemKind::Struct(_) => true,
                ItemKind::Enum(_) => true,
                ItemKind::Fn(_) => true,
                ItemKind::Trait(_) => false,
                ItemKind::Impl(_) => false,
                ItemKind::Abi(_) => false,
                ItemKind::Const(_) => true,
                ItemKind::Storage(_) => false,
                // TODO: Currently, only single configurables can be deprecated.
                //       Change to true once https://github.com/FuelLabs/sway/issues/6942 is implemented.
                ItemKind::Configurable(_) => false,
                ItemKind::TypeAlias(_) => false,
                ItemKind::Error(_, _) => true,
            },
            Fallback => matches!(item_kind, ItemKind::Fn(_)),
            ErrorType => matches!(item_kind, ItemKind::Enum(_)),
            Error => false,
            Trace => matches!(item_kind, ItemKind::Fn(_)),
        }
    }

    // TODO: Add `can_annotated_nested_item_kind`, once we properly support nested items.
    //       E.g., the `#[test]` attribute can annotate module functions (`ItemKind::Fn`),
    //       but will not be allowed on nested functions.

    pub(crate) fn can_annotate_struct_or_enum_field(
        &self,
        struct_or_enum_field: StructOrEnumField,
    ) -> bool {
        use AttributeKind::*;
        match self.kind {
            Unknown => true,
            DocComment => self.direction == AttributeDirection::Outer,
            Storage => false,
            Inline => false,
            Test => false,
            Payable => false,
            Allow => true,
            Cfg => true,
            Deprecated => true,
            Fallback => false,
            ErrorType => false,
            Error => struct_or_enum_field == StructOrEnumField::EnumField,
            Trace => false,
        }
    }

    pub(crate) fn can_annotate_abi_or_trait_item(
        &self,
        item: &ItemTraitItem,
        parent: TraitItemParent,
    ) -> bool {
        use AttributeKind::*;
        match self.kind {
            Unknown => true,
            DocComment => self.direction == AttributeDirection::Outer,
            Storage => matches!(item, ItemTraitItem::Fn(..)),
            // Functions in the trait or ABI interface surface cannot be marked as inlined
            // because they don't have implementation.
            Inline => false,
            Test => false,
            Payable => parent == TraitItemParent::Abi && matches!(item, ItemTraitItem::Fn(..)),
            Allow => true,
            Cfg => true,
            // TODO: Change to true once https://github.com/FuelLabs/sway/issues/6942 is implemented.
            Deprecated => false,
            Fallback => false,
            ErrorType => false,
            Error => false,
            // Functions in the trait or ABI interface surface cannot be marked as traced
            // because they don't have implementation.
            Trace => false,
        }
    }

    pub(crate) fn can_annotate_impl_item(
        &self,
        item: &ItemImplItem,
        parent: ImplItemParent,
    ) -> bool {
        use AttributeKind::*;
        match self.kind {
            Unknown => true,
            DocComment => self.direction == AttributeDirection::Outer,
            Storage => matches!(item, ItemImplItem::Fn(..)),
            Inline => matches!(item, ItemImplItem::Fn(..)),
            Test => false,
            Payable => parent == ImplItemParent::Contract,
            Allow => true,
            Cfg => true,
            Deprecated => !matches!(item, ItemImplItem::Type(_)),
            Fallback => false,
            ErrorType => false,
            Error => false,
            Trace => matches!(item, ItemImplItem::Fn(..)),
        }
    }

    pub(crate) fn can_annotate_abi_or_trait_item_fn(
        &self,
        abi_or_trait_item: TraitItemParent,
    ) -> bool {
        use AttributeKind::*;
        match self.kind {
            Unknown => true,
            DocComment => self.direction == AttributeDirection::Outer,
            Storage => true,
            Inline => true,
            Test => false,
            Payable => abi_or_trait_item == TraitItemParent::Abi,
            Allow => true,
            Cfg => true,
            Deprecated => true,
            Fallback => false,
            ErrorType => false,
            Error => false,
            Trace => true,
        }
    }

    pub(crate) fn can_annotate_storage_entry(&self) -> bool {
        use AttributeKind::*;
        match self.kind {
            Unknown => true,
            DocComment => self.direction == AttributeDirection::Outer,
            Storage => false,
            Inline => false,
            Test => false,
            Payable => false,
            Allow => true,
            Cfg => true,
            // TODO: Change to true once https://github.com/FuelLabs/sway/issues/6942 is implemented.
            Deprecated => false,
            Fallback => false,
            ErrorType => false,
            Error => false,
            Trace => false,
        }
    }

    pub(crate) fn can_annotate_configurable_field(&self) -> bool {
        use AttributeKind::*;
        match self.kind {
            Unknown => true,
            DocComment => self.direction == AttributeDirection::Outer,
            Storage => false,
            Inline => false,
            Test => false,
            Payable => false,
            Allow => true,
            Cfg => true,
            Deprecated => true,
            Fallback => false,
            ErrorType => false,
            Error => false,
            Trace => false,
        }
    }

    pub(crate) fn can_only_annotate_help(&self, target_friendly_name: &str) -> Vec<&'static str> {
        // Using strings to identify targets is not ideal, but there
        // is no real need for a more complex and type-safe identification here.
        use AttributeKind::*;
        let help = match self.kind {
            Unknown => vec![],
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
            // TODO: Remove this help lines once https://github.com/FuelLabs/sway/issues/6942 is implemented.
            Deprecated => vec![
                "\"deprecated\" attribute is currently not implemented for all elements that could be deprecated.",
            ],
            Fallback => vec!["\"fallback\" attribute can only annotate module functions in a contract module."],
            ErrorType => vec!["\"error_type\" attribute can only annotate enums."],
            Error => vec!["\"error\" attribute can only annotate enum variants of enums annotated with the \"error_type\" attribute."],
            Trace => vec!["\"trace\" attribute can only annotate functions that can panic."],
        };

        if help.is_empty() && target_friendly_name.starts_with("module kind") {
            vec!["Annotating module kinds (contract, script, predicate, or library) is currently not implemented."]
        } else {
            help
        }
    }
}

/// Stores the [Attribute]s that annotate an element.
///
/// Note that once stored in the [Attributes], the [Attribute]s lose
/// the information about their enclosing [AttributeDecl].
///
/// The map can contain erroneous attributes. A typical example s containing
/// several attributes of an [AttributeKind] that allows only a single attribute
/// to be applied, like, e.g., `#[deprecated]`, or `#[test]`.
///
/// When retrieving such attributes, we follow the last-wins approach
/// and return the last attribute in the order of declaration.
#[derive(Default, Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Attributes {
    // Note that we don't need a map here, to store attributes because:
    //  - Attributes will mostly be empty.
    //  - Per `AttributeKind` there will usually be just one element.
    //  - The only exception are comments, that anyhow need to be traversed sequentially
    //    and will dominate in the list of attributes or mostly be the only attributes.
    //  - Most of the analysis requires traversing all attributes regardless of the `AttributeKind`.
    //  - Analysis that is interested in `AttributeKind` anyhow needs to apply a filter first.
    //  - Attributes are accessed only once, when checking the declaration of the annotated element.
    /// [Attribute]s, in the order of their declaration.
    attributes: Arc<Vec<Attribute>>,
    // `#[deprecated]` is the only attribute requested on call sites,
    // and we provide a O(1) access to it.
    /// The index of the last `#[deprecated]` attribute, if any.
    deprecated_attr_index: Option<usize>,
}

impl Attributes {
    pub fn new(attribute_decls: &[AttributeDecl]) -> Attributes {
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

        Attributes {
            deprecated_attr_index: attributes
                .iter()
                .rposition(|attr| attr.kind == AttributeKind::Deprecated),
            attributes: Arc::new(attributes),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    /// Returns the first attribute, ordered by span, or None if there are no attributes.
    pub fn first(&self) -> Option<&Attribute> {
        self.attributes.first()
    }

    pub fn known_attribute_names(&self) -> &'static [&'static str] {
        KNOWN_ATTRIBUTE_NAMES
    }

    pub fn all(&self) -> impl Iterator<Item = &Attribute> {
        self.attributes.iter()
    }

    pub fn all_as_slice(&self) -> &[Attribute] {
        self.attributes.as_slice()
    }

    pub fn all_by_kind<F>(&self, predicate: F) -> IndexMap<AttributeKind, Vec<&Attribute>>
    where
        F: Fn(&&Attribute) -> bool,
    {
        let mut result = IndexMap::<_, Vec<&Attribute>>::new();
        for attr in self.attributes.iter().filter(predicate) {
            result.entry(attr.kind).or_default().push(attr);
        }
        result
    }

    pub fn of_kind(&self, kind: AttributeKind) -> impl Iterator<Item = &Attribute> {
        self.attributes.iter().filter(move |attr| attr.kind == kind)
    }

    pub fn has_any_of_kind(&self, kind: AttributeKind) -> bool {
        self.of_kind(kind).any(|_| true)
    }

    pub fn unknown(&self) -> impl Iterator<Item = &Attribute> {
        self.attributes
            .iter()
            .filter(|attr| attr.kind == AttributeKind::Unknown)
    }

    pub fn has_allow_dead_code(&self) -> bool {
        self.has_allow(|arg| arg.is_allow_dead_code())
    }

    pub fn has_allow_deprecated(&self) -> bool {
        self.has_allow(|arg| arg.is_allow_deprecated())
    }

    fn has_allow(&self, arg_filter: impl Fn(&AttributeArg) -> bool) -> bool {
        self.of_kind(AttributeKind::Allow)
            .flat_map(|attribute| &attribute.args)
            .any(arg_filter)
    }

    pub fn has_error_type(&self) -> bool {
        self.of_kind(AttributeKind::ErrorType).any(|_| true)
    }

    pub fn has_error(&self) -> bool {
        self.of_kind(AttributeKind::Error).any(|_| true)
    }

    /// Returns the value of the `#[inline]` [Attribute], or `None` if the
    /// [Attributes] does not contain any `#[inline]` attributes.
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

    /// Returns the value of the `#[trace]` [Attribute], or `None` if the
    /// [Attributes] does not contain any `#[trace]` attributes.
    pub fn trace(&self) -> Option<Trace> {
        // `trace` attribute can be applied only once (`AttributeMultiplicity::Single`),
        // and can have exactly one argument, otherwise an error is emitted.
        // Last-wins approach.
        match self
            .of_kind(AttributeKind::Trace)
            .last()?
            .args
            .last()?
            .name
            .as_str()
        {
            TRACE_NEVER_ARG_NAME => Some(Trace::Never),
            TRACE_ALWAYS_ARG_NAME => Some(Trace::Always),
            _ => None,
        }
    }

    /// Returns the value of the `#[storage]` [Attribute], or [Purity::Pure] if the
    /// [Attributes] does not contain any `#[storage]` attributes.
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

        for arg in storage_attr.args.iter() {
            match arg.name.as_str() {
                STORAGE_READ_ARG_NAME => add_impurity(Purity::Reads, Purity::Writes),
                STORAGE_WRITE_ARG_NAME => add_impurity(Purity::Writes, Purity::Reads),
                _ => {}
            }
        }

        purity
    }

    /// Returns the `#[deprecated]` [Attribute], or `None` if the
    /// [Attributes] does not contain any `#[deprecated]` attributes.
    pub fn deprecated(&self) -> Option<&Attribute> {
        self.deprecated_attr_index
            .map(|index| &self.attributes[index])
    }

    /// Returns the `#[test]` [Attribute], or `None` if the
    /// [Attributes] does not contain any `#[test]` attributes.
    pub fn test(&self) -> Option<&Attribute> {
        // Last-wins approach.
        self.of_kind(AttributeKind::Test).last()
    }

    /// Returns the `#[error]` [Attribute], or `None` if the
    /// [Attributes] does not contain any `#[error]` attributes.
    pub fn error(&self) -> Option<&Attribute> {
        // Last-wins approach.
        self.of_kind(AttributeKind::Error).last()
    }

    /// Returns the error message of the `#[error]` [Attribute],
    /// or `None` if the [Attributes] does not contain any
    /// `#[error]` attributes, or if the attribute does not
    /// contain a message argument (`m`), or if the message
    /// argument is not a string.
    pub fn error_message(&self) -> Option<&String> {
        // Last-wins approach.
        self.error().and_then(|error_attr| {
            error_attr
                .args
                .iter()
                .filter(|arg| arg.is_error_message())
                .next_back()
                .and_then(|arg| arg.get_string_opt(&Handler::default()).ok().flatten())
        })
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
    pub(crate) fn enter(&mut self, attributes: Attributes) -> AllowDeprecatedEnterToken {
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
