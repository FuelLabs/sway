library;

#[deprecated]
pub struct Ok1 { }

#[deprecated()]
pub struct Ok2 { }

#[deprecated(note = "note")]
pub struct Ok3 { }

#[deprecated(note = "note", note = "other note")]
pub struct NotOk1 { }