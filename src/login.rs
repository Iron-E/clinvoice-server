pub trait Login
{
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
