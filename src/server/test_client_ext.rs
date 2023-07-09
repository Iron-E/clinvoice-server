//! Contains [extensions](TestClientExt) for [`TestClient`].

use core::{
	fmt::Debug,
	hash::Hash,
	marker::{Send, Sync},
};
use std::{collections::HashSet, sync::OnceLock};

use axum::http::header;
use axum_login::axum_sessions::async_session::base64;
use axum_test_helper::{RequestBuilder, TestClient};
use pretty_assertions::assert_eq;
use serde::{de::DeserializeOwned, Serialize};
use sqlx::{Database, Pool};
use winvoice_adapter::Retrievable;

use super::response::{LoginResponse, LogoutResponse};
use crate::{
	api::{
		self,
		request,
		response::{Get, Login, Logout, Post},
		routes,
		Code,
		Status,
	},
	schema::User,
	server::response::Response,
};

/// Get the default version requirement for tests.
fn version_req() -> &'static str
{
	static VERSION_REQ: OnceLock<String> = OnceLock::new();
	VERSION_REQ.get_or_init(|| format!("={}", api::version()))
}

/// Extensions for [`TestClient`].
#[async_trait::async_trait]
pub trait TestClientExt
{
	/// Make a DELETE [`RequestBuilder`] on the given `route`.
	fn delete_builder(&self, route: &str) -> RequestBuilder;

	/// Make a GET [`RequestBuilder`] on the given `route`.
	fn get_builder(&self, route: &str) -> RequestBuilder;

	/// Log in a [`User`](crate::schema::User) into the [`TestClient`].
	async fn login(&self, username: &str, password: &str);

	/// Log out a [`User`](crate::schema::User) into the [`TestClient`].
	async fn logout(&self);

	/// Make a PATCH [`RequestBuilder`] on the given `route`.
	fn patch_builder(&self, route: &str) -> RequestBuilder;

	/// Make a POST [`RequestBuilder`] on the given `route`.
	fn post_builder(&self, route: &str) -> RequestBuilder;

	/// assert logged in user GET with permissions is accepted
	async fn test_get_success<'ent, M, E, Iter>(
		&self,
		route: &str,
		user: &User,
		password: &str,
		condition: M,
		entities: Iter,
		code: Option<Code>,
	) where
		E: 'ent + Clone + Debug + DeserializeOwned + Eq + Hash + PartialEq + Send + Serialize,
		Iter: Debug + Iterator<Item = &'ent E> + Send,
		M: Debug + Serialize + Send + Sync;

	/// assert logged in user GET with permissions is rejected
	async fn test_get_unauthorized<M>(&self, route: &str, user: &User, password: &str)
	where
		M: Debug + Default + Serialize + Send + Sync;

	/// assert logged in user POST with permissions is accepted
	async fn test_post_success<R, A>(
		&self,
		pool: &Pool<R::Db>,
		route: &str,
		user: &User,
		password: &str,
		args: A,
		code: Option<Code>,
	) -> R::Entity
	where
		A: Debug + Send + Serialize + Sync,
		R: Retrievable,
		R::Entity: Clone + Debug + DeserializeOwned + PartialEq + Send,
		R::Match: Debug + From<R::Entity> + Send;

	/// assert logged in user POST with permissions is rejected
	async fn test_post_unauthorized<Db, A>(&self, pool: &Pool<Db>, route: &str, user: &User, password: &str, args: A)
	where
		A: Debug + Send + Serialize + Sync,
		Db: Database;
}

#[async_trait::async_trait]
impl TestClientExt for TestClient
{
	fn delete_builder(&self, route: &str) -> RequestBuilder
	{
		self.delete(route).header(api::HEADER, version_req())
	}

	fn get_builder(&self, route: &str) -> RequestBuilder
	{
		self.get(route).header(api::HEADER, version_req())
	}

	#[tracing::instrument(skip(self))]
	async fn login(&self, username: &str, password: &str)
	{
		let response = self
			.get(routes::LOGIN)
			.header(api::HEADER, version_req())
			.header(header::AUTHORIZATION, format!("Basic {}", base64::encode(format!("{username}:{password}"))))
			.send()
			.await;

		let expected = LoginResponse::from(Code::Success);
		assert_eq!(response.status(), expected.status());
		assert_eq!(&response.json::<Login>().await, expected.content());
	}

	#[tracing::instrument(skip(self))]
	async fn logout(&self)
	{
		let response = self.get(routes::LOGOUT).header(api::HEADER, version_req()).send().await;
		let expected = LogoutResponse::from(Code::Success);
		assert_eq!(response.status(), expected.status());
		assert_eq!(&response.json::<Logout>().await, expected.content());
	}

	fn patch_builder(&self, route: &str) -> RequestBuilder
	{
		self.patch(route).header(api::HEADER, version_req())
	}

	fn post_builder(&self, route: &str) -> RequestBuilder
	{
		self.post(route).header(api::HEADER, version_req())
	}

	#[tracing::instrument(skip(self))]
	async fn test_get_success<'ent, M, E, Iter>(
		&self,
		route: &str,
		user: &User,
		password: &str,
		condition: M,
		entities: Iter,
		code: Option<Code>,
	) where
		E: 'ent + Clone + Debug + DeserializeOwned + Eq + Hash + PartialEq + Send + Serialize,
		Iter: Debug + Iterator<Item = &'ent E> + Send,
		M: Debug + Serialize + Send + Sync,
	{
		// HACK: `tracing` doesn't work correctly with asyn cso I have to annotate this function
		// like       this or else this function's span is skipped.
		tracing::trace!(parent: None, "\n");
		tracing::trace!("\n");

		self.login(user.username(), password).await;
		let response = self.get_builder(route).json(&request::Get::new(condition)).send().await;
		let status = response.status();

		let actual = Response::new(status, response.json::<Get<E>>().await);
		let expected = Response::from(Get::<E>::new(
			entities.into_iter().cloned().collect(),
			code.unwrap_or(Code::Success).into(),
		));

		assert_eq!(
			actual.content().entities().into_iter().collect::<HashSet<_>>(),
			expected.content().entities().into_iter().collect::<HashSet<_>>()
		);
		assert_eq!(actual.content().status(), expected.content().status());
		assert_eq!(actual.status(), expected.status());
		self.logout().await;
	}

	#[tracing::instrument(skip(self))]
	async fn test_get_unauthorized<M>(&self, route: &str, user: &User, password: &str)
	where
		M: Debug + Default + Serialize + Send + Sync,
	{
		// HACK: `tracing` doesn't work correctly with asyn cso I have to annotate this function
		// like       this or else this function's span is skipped.
		tracing::trace!(parent: None, "\n");
		tracing::trace!("\n");

		self.login(user.username(), password).await;
		let response = self.get_builder(route).json(&request::Get::new(M::default())).send().await;

		let actual = Response::new(response.status(), response.json::<Get<()>>().await);
		let expected = Response::from(Get::<()>::from(Status::from(Code::Unauthorized)));

		assert_eq!(actual.status(), expected.status());
		assert_eq!(actual.content().entities(), expected.content().entities());
		assert_eq!(actual.content().status().code(), expected.content().status().code());
		self.logout().await;
	}

	#[tracing::instrument(skip(self))]
	async fn test_post_success<R, A>(
		&self,
		pool: &Pool<R::Db>,
		route: &str,
		user: &User,
		password: &str,
		args: A,
		code: Option<Code>,
	) -> R::Entity
	where
		A: Debug + Send + Serialize + Sync,
		R: Retrievable,
		R::Entity: Clone + Debug + DeserializeOwned + PartialEq + Send,
		R::Match: Debug + From<R::Entity> + Send,
	{
		// HACK: `tracing` doesn't work correctly with async so I have to annotate this function
		// like       this or else this function's span is skipped.
		tracing::trace!(parent: None, "\n");
		tracing::trace!("\n");

		self.login(user.username(), password).await;
		let response = self.post_builder(route).json(&request::Post::new(args)).send().await;
		let status = response.status();

		let actual = Response::new(status, response.json::<Post<R::Entity>>().await);
		let expected = {
			let entity = actual.content().entity().unwrap().clone();
			let row = R::retrieve(pool, R::Match::from(entity)).await.map(|mut v| v.remove(0)).unwrap();
			Response::from(Post::new(row.into(), code.unwrap_or(Code::Success).into()))
		};

		assert_eq!(actual, expected);
		self.logout().await;

		actual.into_content().into_entity().unwrap()
	}

	#[tracing::instrument(skip(self))]
	async fn test_post_unauthorized<Db, A>(&self, pool: &Pool<Db>, route: &str, user: &User, password: &str, args: A)
	where
		A: Debug + Send + Serialize + Sync,
		Db: Database,
	{
		// HACK: `tracing` doesn't work correctly with async so I have to annotate this function
		// like       this or else this function's span is skipped.
		tracing::trace!(parent: None, "\n");
		tracing::trace!("\n");

		self.login(user.username(), password).await;
		let response = self.post_builder(route).json(&request::Post::new(args)).send().await;
		let status = response.status();

		let actual = Response::new(status, response.json::<Post<()>>().await);
		let expected = Response::from(Post::new(None, Code::Unauthorized.into()));

		assert_eq!(actual.status(), expected.status());
		assert_eq!(actual.content().entity(), expected.content().entity());
		assert_eq!(actual.content().status().code(), expected.content().status().code());
		self.logout().await;
	}
}
