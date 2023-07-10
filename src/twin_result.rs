//! Contains a [result type](TwinResult) which has the same [`Ok`] and [`Error`] variants. Only useful because the
//! [`FromResidual`](core::ops::FromResidual) trait is unstable.

//! Has the same [`Ok`] and [`Error`] variants. Only useful because the [`FromResidual`](core::ops::FromResidual) trait
//! is unstable.
pub type TwinResult<T> = Result<T, T>;
