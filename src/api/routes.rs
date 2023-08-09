//! The API endpoints for the [`winvoice_server`](crate)

/// The endpoint for [`winvoice_schema::Contact`]s
pub const CONTACT: &str = "/contact";

/// The endpoint for [`winvoice_schema::Contact`]s
pub const DEPARTMENT: &str = "/department";

/// The API endpoint for [`winvoice_schema::Employee`]
pub const EMPLOYEE: &str = "/employee";

/// The API endpoint for [`winvoice_schema::Expense`]
pub const EXPENSE: &str = "/expense";

/// The API endpoint for exporting [`winvoice_schema::Job`]s
pub const EXPORT: &str = "/job/export";

/// The API endpoint for [`winvoice_schema::Job`]
pub const JOB: &str = "/job";

/// The API endpoint for [`winvoice_schema::Location`]
pub const LOCATION: &str = "/location";

/// The API endpoint for logging in
pub const LOGIN: &str = "/login";

/// The API endpoint for logging out
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
pub const WHO_AM_I: &str = "/whoami";
