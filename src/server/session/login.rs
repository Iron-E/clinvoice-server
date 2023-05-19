/// An entity which is capable of setting login options.
pub trait Login
{
	/// Pass a `username` and `password` to some entity, in order to log in.
	fn login(self, username: &str, password: &str) -> Self;
}

#[cfg(feature = "postgres")]
impl Login for sqlx::postgres::PgConnectOptions
{
	fn login(self, username: &str, password: &str) -> Self
	{
		self.username(username).password(password)
	}
}
