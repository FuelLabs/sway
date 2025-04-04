library;

mod empty_module;

mod nested_annotations;

mod trailing_single_inner_attribute;
mod trailing_single_inner_doc_comment;

mod trailing_single_outer_attribute;

mod trailing_multiple_inner_doc_comments;
mod trailing_multiple_inner_attributes;
mod trailing_multiple_outer_attributes;
mod trailing_multiple_mixed_attributes_and_comments;

pub struct S1 {
    #[allow(dead_code)]
}

pub struct S2 {
    /// Outer doc comment.
}

pub struct S3 {
    //! Inner doc comment.
}

impl S1 {
    #[allow(dead_code)]
}

impl S2 {
    /// Outer doc comment.
}

impl S3 {
    //! Inner doc comment.
}

pub enum E {
    #[allow(dead_code)]
}

#[allow(dead_code)]
trait T1 {
    #[allow(dead_code)]
}

#[allow(dead_code)]
trait T2 {
    /// Outer doc comment.
}

#[allow(dead_code)]
trait T3 {
    //! Inner doc comment.
}

impl T1 for S1 {
    #[allow(dead_code)]
}

impl T2 for S2 {
    /// Outer doc comment.
}

impl T3 for S3 {
    //! Inner doc comment.
}

/// This is an outer doc comment.
/// It has several lines.

/// And even an empty line in-between them.
/// But it doesn't have an item following the comment :-(