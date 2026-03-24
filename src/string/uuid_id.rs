//! An ID with UUID…

use uuid::Uuid;

/// A trait for anything that wants to represent self with UUID tagged to end.
pub trait AsUuidId {
    /// Self as `self`+UUID.
    /// 
    /// # Returns
    /// E.g. with "`foobar`" as `self`, something akin to `foobar-488aeca2-8191-49f3-bc34-845de208a23d`.
    fn uuided(&self) -> String;
}

/// Generate an ID with UUID tagged to it.
fn uuid(str: &str) -> String { format!("{str}-{}", Uuid::new_v4())}

impl AsUuidId for String {
    fn uuided(&self) -> String { uuid(self) }
}

impl AsUuidId for &String {
    fn uuided(&self) -> String { uuid(self) }
}

impl AsUuidId for &str {
    fn uuided(&self) -> String { uuid(self) }
}
