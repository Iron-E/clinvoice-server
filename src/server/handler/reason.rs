mod display;

/// Reasons that a permission may be lacking.
pub enum Reason
{
	/// No [`Employee`](winvoice_schema::Employee) record, no [`Department`](winvoice_schema::Department)
	NoDepartment,

	/// No [`Employee`](winvoice_schema::Employee) record, no [`Department`](winvoice_schema::Department)
	NoEmployee,

	/// The specified resource does not exist.
	NoResourceExists,

	/// Another resource depends on it.
	ResourceConstraint,

	/// The specified resource already exists.
	ResourceExists,
}
