//! Contains [`WhoAmI`] JSON from the [`winvoice_server::api`] which is a proper HTTP [`Response`].

mod from;

use crate::api::response::WhoAmI;

crate::new_response!(WhoAmIResponse(WhoAmI): Clone, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd);
