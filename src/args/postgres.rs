use core::time::Duration;
use std::{net::SocketAddr, path::PathBuf};

use axum_server::tls_rustls::RustlsConfig;
use casbin::Enforcer;
use clap::Args;
use sqlx::{
	pool::PoolOptions,
	postgres::{PgConnectOptions, PgSslMode},
};
use winvoice_adapter_postgres::PgSchema;

use crate::{
	lock::Lock,
	server::{Server, ServerState},
	DynResult,
};

/// Spawn a Winvoice Server which interacts which a Postgres Database.
#[derive(Args, Clone, Debug)]
pub struct Postgres
{
	/// The name of the database where Winvoice should perform its operations.
	#[arg(env = "PGDATABASE", long, short)]
	database: String,

	/// This changes the default precision of floating-point values.
	#[arg(long, short)]
	float_digits: Option<i8>,

	/// Sets the name of the host to connect to.
	///
	/// If a host name begins with a slash, it specifies Unix-domain communication rather than
	/// TCP/IP communication; the value is the name of the directory in which the socket file is
	/// stored.
	///
	/// The default behavior when host is not specified, or is empty, is to connect to a
	/// Unix-domain socket
	#[arg(default_value_t, env = "PGHOST", long, short)]
	host: String,

	/// The password used to establish a master connection with the database.
	#[arg(default_value_t, env = "PGPASSWORD", long, short)]
	password: String,

	/// Sets the port to connect to at the server host.
	#[arg(env = "PGPORT", long, short, value_name = "NUMBER")]
	port: Option<u16>,

	/// Determines whether or with what priority a secure SSL TCP/IP connection will be negotiated.
	#[arg(default_value = "prefer", env = "PGSSLMODE", long, short = 'm')]
	ssl_mode: PgSslMode,

	/// Sets the name of a file containing a list of trusted SSL Certificate Authorities.
	#[arg(env = "PGSSLROOTCERT", long, short = 'r', value_name = "FILE")]
	ssl_root_cert: Option<PathBuf>,

	/// Sets the capacity of the connection’s statement cache in a number of stored distinct
	/// statements.
	///
	/// Caching is handled using LRU, meaning when the amount of queries hits the defined limit,
	/// the oldest statement will get dropped.
	#[arg(long, short = 'c', value_name = "COUNT")]
	statement_cache_capacity: Option<usize>,

	/// The username used to establish a master connection with the database.
	#[arg(default_value_t, env = "PGUSER", long, short)]
	username: String,
}

impl Postgres
{
	/// Run the Winvoice postgres server.
	#[allow(clippy::too_many_arguments)]
	pub async fn run(
		self,
		address: SocketAddr,
		connection_idle: Duration,
		cookie_domain: Option<String>,
		cookie_secret: Vec<u8>,
		permissions: Lock<Enforcer>,
		session_ttl: Duration,
		timeout: Option<Duration>,
		tls: RustlsConfig,
	) -> DynResult<()>
	{
		let mut connect_options = PgConnectOptions::new()
			.application_name("winvoice-server")
			.database(&self.database)
			.host(&self.host)
			.ssl_mode(self.ssl_mode);

		if let Some(d) = self.float_digits
		{
			connect_options = connect_options.extra_float_digits(d);
		}

		if let Some(p) = self.port
		{
			connect_options = connect_options.port(p);
		}

		if let Some(c) = self.ssl_root_cert
		{
			connect_options = connect_options.ssl_root_cert(c);
		}

		if let Some(c) = self.statement_cache_capacity
		{
			connect_options = connect_options.statement_cache_capacity(c);
		}

		let pool = PoolOptions::<sqlx::Postgres>::new()
			.idle_timeout(connection_idle)
			.connect_with(connect_options)
			.await?;

		Server::<PgSchema>::new(address, tls)
			.serve(
				cookie_domain,
				cookie_secret,
				ServerState::new(permissions, pool),
				session_ttl,
				timeout,
			)
			.await
	}
}
