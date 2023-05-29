//! Contains extensions to [`Adapter`](BaseAdapter) for the [`winvoice_server`].

use winvoice_adapter::{schema::Adapter as BaseAdapter, Deletable};

use super::{RoleAdapter, UserAdapter};

pub trait Adapter: BaseAdapter
{
	/// The adapter for [`Role`](super::Role)s
	type Role: Deletable<Db = Self::Db> + RoleAdapter;

	/// The adapter for [`User`](super::User)s
	type User: Deletable<Db = Self::Db> + UserAdapter;
}
