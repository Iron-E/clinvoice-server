//! The API endpoints for the [`winvoice_server`](crate)
//!
//! All endpoints accept `DELETE`, `PATCH`, `POST`, and `PUT` unless otherwise specified.

/// The endpoint for [`winvoice_schema::Contact`]s
pub const CONTACT: &str = "/contact";

/// The endpoint for [`winvoice_schema::Contact`]s
pub const DEPARTMENT: &str = "/department";

/// The API endpoint for [`winvoice_schema::Employee`]
pub const EMPLOYEE: &str = "/employee";

/// The API endpoint for [`winvoice_schema::Expense`]
pub const EXPENSE: &str = "/expense";

/// The API endpoint for exporting [`winvoice_schema::Job`]s
///
/// Accepts a `POST` request with a JSON [`Export`](super::request::Export) body only.
///
/// The heading of the exported document is controlled by the [`Contact`](winvoice_schema::Contact) with the label
/// 'Name' (case-sensitive).
pub const EXPORT: &str = "/job/export";

/// The API endpoint for [`winvoice_schema::Job`]
pub const JOB: &str = "/job";

/// The API endpoint for [`winvoice_schema::Location`]
pub const LOCATION: &str = "/location";

/// The API endpoint for logging in.
///
/// Unlike other endpoints, takes a `POST` request with a [basic authorization
/// header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization#basic_authentication)
pub const LOGIN: &str = "/login";

/// The API endpoint for logging out
///
/// Unlike other endpoints, takes a `POST` request with no body.
pub const LOGOUT: &str = "/logout";

/// The API endpoint for [`winvoice_schema::Organization`]
pub const ORGANIZATION: &str = "/organization";

/// The API endpoint for [`Role`](crate::schema::Role)
pub const ROLE: &str = "/role";

/// The API endpoint for [`winvoice_schema::Timesheet`]
pub const TIMESHEET: &str = "/timesheet";

/// The API endpoint for [`User`](crate::schema::User)
pub const USER: &str = "/user";

/// The API endpoint for retrieving the currently logged in [`User`](crate::schema::User)'s information.
///
/// Like [`LOGOUT`], takes a `POST` request with no body.
pub const WHO_AM_I: &str = "/whoami";
