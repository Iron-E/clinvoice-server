//! Contains helpers for [`Cookie`](axum_extra::extract::cookie::Cookie)s.

use std::borrow::Cow;

use axum_extra::extract::cookie::{Cookie, SameSite};
use time::OffsetDateTime;

pub fn new<'cookie, D, N, V>(
	name: N,
	value: V,
	domain: D,
	expires: OffsetDateTime,
) -> Cookie<'cookie>
where
	D: Into<Cow<'cookie, str>>,
	N: Into<Cow<'cookie, str>>,
	V: Into<Cow<'cookie, str>>,
{
	let mut cookie = Cookie::new(name, value);
	cookie.set_domain(domain);
	cookie.set_expires(expires);
	cookie.set_http_only(true);
	cookie.set_same_site(SameSite::Strict);
	cookie.set_secure(true);
	cookie
}
