//! Contains [`Export`] JSON from the [`winvoice_server::api`] which is a proper HTTP [`Response`].

mod from;

use std::collections::HashMap;

use super::{Response, StatusCode};
use crate::api::{response::Export, Status};

crate::new_response!(ExportResponse(Export): Clone, Debug, Default, Eq, PartialEq);

impl ExportResponse
{
	/// Create a new [`ExportResponse`].
	pub const fn new(code: StatusCode, exported: HashMap<String, String>, status: Status) -> Self
	{
		Self(Response::new(code, Export::new(exported, status)))
	}
}
