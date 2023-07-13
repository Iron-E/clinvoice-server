//! Contains method extensions for the [`Result`] type.

/// Method extensions for [`Result`]s.
pub trait ResultExt
{
	type Ok;
	type Err;

	/// Map both sides of the [`Result`], returning a [`Result`] with different variant types.
	fn map_all<Ok2, OkFn, Err2, ErrFn>(self, ok: OkFn, err: ErrFn) -> Result<Ok2, Err2>
	where
		OkFn: FnOnce(Self::Ok) -> Ok2,
		ErrFn: FnOnce(Self::Err) -> Err2;
}

impl<O, E> ResultExt for Result<O, E>
{
	type Err = E;
	type Ok = O;

	#[inline]
	fn map_all<Ok2, OkFn, Err2, ErrFn>(self, ok: OkFn, err: ErrFn) -> Result<Ok2, Err2>
	where
		OkFn: FnOnce(O) -> Ok2,
		ErrFn: FnOnce(E) -> Err2,
	{
		match self
		{
			Ok(o) => Ok(ok(o)),
			Err(e) => Err(err(e)),
		}
	}
}
