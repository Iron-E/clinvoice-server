//! Extensions for [`bool`]s.

pub trait BoolExt
{
	/// Calls and returns `r#true` if `true`, or returns `r#false` otherwise. `r#false` is eagerly
	/// evaluated— see [`BoolExt::then_or_else`] for a lazy alternative.
	fn then_or<T, TrueFn>(self, r#false: T, r#true: TrueFn) -> T
	where
		TrueFn: FnOnce() -> T;

	/// Calls `r#false` if `false`, and `r#true` otherwise. Both sides are lazily evaluated.
	fn then_or_else<T, FalseFn, TrueFn>(self, r#false: FalseFn, r#true: TrueFn) -> T
	where
		FalseFn: FnOnce() -> T,
		TrueFn: FnOnce() -> T;

	/// Returns `r#true` if `true`, and `r#false` otherwise. Both sides are eagerly
	/// evaluated— see [`BoolExt::then_or_else`] for a lazy alternative.
	fn then_some_or<T>(self, r#false: T, r#true: T) -> T;

	/// Calls and returns `r#false` if `false`, or returns `r#true`. `r#true` is eagerly
	/// evaluated— see [`BoolExt::then_or_else`] for a lazy alternative.
	fn then_some_or_else<T, FalseFn>(self, r#false: FalseFn, r#true: T) -> T
	where
		FalseFn: FnOnce() -> T;
}

impl BoolExt for bool
{
	#[inline]
	fn then_or<T, TrueFn>(self, r#false: T, r#true: TrueFn) -> T
	where
		TrueFn: FnOnce() -> T,
	{
		match self
		{
			true => r#true(),
			false => r#false,
		}
	}

	#[inline]
	fn then_or_else<T, FalseFn, TrueFn>(self, r#false: FalseFn, r#true: TrueFn) -> T
	where
		FalseFn: FnOnce() -> T,
		TrueFn: FnOnce() -> T,
	{
		match self
		{
			true => r#true(),
			false => r#false(),
		}
	}

	#[inline]
	fn then_some_or<T>(self, r#false: T, r#true: T) -> T
	{
		match self
		{
			true => r#true,
			false => r#false,
		}
	}

	#[inline]
	fn then_some_or_else<T, FalseFn>(self, r#false: FalseFn, r#true: T) -> T
	where
		FalseFn: FnOnce() -> T,
	{
		match self
		{
			true => r#true,
			false => r#false(),
		}
	}
}
