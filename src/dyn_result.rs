use std::error::Error;

/// An [`Error`] type which can be anything.
pub type DynError = Box<dyn Error + Send + Sync>;

/// A [`Result`] which can contain any [`Error`].
pub type DynResult<TOk> = Result<TOk, DynError>;
