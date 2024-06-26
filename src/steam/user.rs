//! Steam Users.

use std::result::Result as StdResult;

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request;
use axum_extra::extract::cookie::Cookie;
use cs2kz::SteamID;
use derive_more::Debug;
use serde::{Deserialize, Deserializer, Serialize};
use tracing::error;
use url::Url;
use utoipa::ToSchema;

use crate::{Error, Result, State};

/// Steam WebAPI URL for fetching information about players.
const API_URL: &str = "https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002";

/// The cookie name used to store the user information.
const COOKIE_NAME: &str = "kz-player";

/// Information about a Steam user.
///
/// This will be serialized as JSON and put into a cookie so frontends can use it.
#[derive(Debug, Serialize, ToSchema)]
pub struct User {
	/// The user's SteamID.
	pub steam_id: SteamID,

	/// Also the user's SteamID, but encoded as a stringified 64-bit integer, because
	/// JavaScript.
	pub steam_id64: String,

	/// The user's username.
	pub username: String,

	/// The user's "real" name.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub realname: Option<String>,

	/// The user's country.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub country: Option<String>,

	/// URL to the user's profile.
	pub profile_url: Url,

	/// URL to the user's avatar.
	pub avatar_url: Url,
}

impl User {
	/// Fetch this user from Steam's API.
	pub async fn fetch(
		steam_id: SteamID,
		http_client: &reqwest::Client,
		config: &crate::Config,
	) -> Result<Self> {
		let url = Url::parse_with_params(API_URL, [
			("key", config.steam_api_key.clone()),
			("steamids", steam_id.as_u64().to_string()),
		])
		.map_err(|err| {
			error!(target: "audit_log", %err, "failed to parse url");
			Error::internal_server_error("failed to parse url").with_source(err)
		})?;

		let response = http_client.get(url).send().await?;

		if let Err(error) = response.error_for_status_ref() {
			let error = Error::bad_gateway("failed to fetch user information from Steam")
				.with_source(error);

			let response_body = response.text().await.ok();

			error!(?error, ?response_body, "failed to fetch steam user");

			return Err(error);
		}

		let user = response.json::<Self>().await?;

		Ok(user)
	}

	/// Creates a [`Cookie`] containing this [`User`] as a JSON value.
	pub fn to_cookie<'c>(&self, config: &'c crate::Config) -> Cookie<'c> {
		let json = serde_json::to_string(self).expect("this is valid json");

		Cookie::build((COOKIE_NAME, json))
			.domain(config.cookie_domain.clone())
			.path("/")
			.secure(cfg!(feature = "production"))
			.http_only(false)
			.permanent()
			.build()
	}
}

impl<'de> Deserialize<'de> for User {
	#[allow(clippy::missing_docs_in_private_items)]
	fn deserialize<D>(deserializer: D) -> StdResult<Self, D::Error>
	where
		D: Deserializer<'de>,
	{
		#[derive(Deserialize)]
		struct Helper1 {
			response: Helper2,
		}

		#[derive(Deserialize)]
		struct Helper2 {
			players: [Helper3; 1],
		}

		#[derive(Deserialize)]
		struct Helper3 {
			steamid: SteamID,
			personaname: String,
			realname: Option<String>,
			loccountrycode: Option<String>,
			profileurl: Url,
			avatar: Url,
		}

		Helper1::deserialize(deserializer).map(|x| x.response).map(
			|Helper2 { players: [player] }| Self {
				steam_id: player.steamid,
				steam_id64: player.steamid.as_u64().to_string(),
				username: player.personaname,
				realname: player.realname,
				country: player.loccountrycode,
				profile_url: player.profileurl,
				avatar_url: player.avatar,
			},
		)
	}
}

#[async_trait]
impl FromRequestParts<&'static State> for User {
	type Rejection = Error;

	async fn from_request_parts(
		parts: &mut request::Parts,
		state: &&'static State,
	) -> Result<Self> {
		let steam_id = parts
			.extensions
			.get::<SteamID>()
			.copied()
			.expect("`SteamLoginResponse` extractor should have inserted this");

		Self::fetch(steam_id, &state.http_client, &state.config).await
	}
}
