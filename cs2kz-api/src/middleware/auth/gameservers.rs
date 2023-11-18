use {
	super::jwt::GameServerInfo,
	crate::{middleware, Error, Result, State},
	axum::{
		body::Body,
		extract::ConnectInfo,
		headers::{authorization::Bearer, Authorization},
		http::Request,
		middleware::Next,
		response::Response,
		TypedHeader,
	},
	jsonwebtoken as jwt,
	serde::Deserialize,
	std::net::SocketAddr,
};

#[derive(Debug, Deserialize)]
struct ServerMetadata {
	plugin_version: u16,
}

#[derive(Debug, Clone)]
pub struct AuthenticatedServer {
	pub id: u16,
	pub plugin_version: u16,
}

#[tracing::instrument(level = "DEBUG")]
pub async fn auth_server(
	state: State,
	ConnectInfo(addr): ConnectInfo<SocketAddr>,
	TypedHeader(api_token): TypedHeader<Authorization<Bearer>>,
	request: Request<Body>,
	next: Next<Body>,
) -> Result<Response> {
	let server_info = jwt::decode::<GameServerInfo>(
		api_token.token(),
		&state.jwt().decode,
		&state.jwt().validation,
	)?
	.claims;

	let server = sqlx::query! {
		r#"
		SELECT
			id
		FROM
			Servers
		WHERE
			id = ?
			AND api_key IS NOT NULL
		"#,
		server_info.id,
	}
	.fetch_one(state.database())
	.await
	.map_err(|_| Error::Unauthorized)?;

	if server_info.exp < jwt::get_current_timestamp() {
		return Err(Error::Unauthorized);
	}

	let (metadata, mut request) = middleware::deserialize_body::<ServerMetadata>(request).await?;

	request
		.extensions_mut()
		.insert(Some(AuthenticatedServer {
			id: server.id,
			plugin_version: metadata.plugin_version,
		}));

	Ok(next.run(request).await)
}