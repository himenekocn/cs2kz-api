use std::num::NonZeroU32;

use axum::Json;

use crate::auth::sessions::Admin;
use crate::auth::{Role, Session};
use crate::responses::Created;
use crate::servers::{CreatedServer, NewServer};
use crate::sqlx::SqlErrorExt;
use crate::{audit, responses, AppState, Error, Result};

/// Register a new CS2 server.
#[tracing::instrument(skip(state))]
#[utoipa::path(
  post,
  tag = "Servers",
  path = "/servers",
  request_body = NewServer,
  responses(
    responses::Created<CreatedServer>,
    responses::BadRequest,
    responses::Unauthorized,
    responses::UnprocessableEntity,
    responses::InternalServerError,
  ),
  security(
    ("Steam Session" = ["servers"]),
  ),
)]
pub async fn create(
	state: AppState,
	session: Session<Admin<{ Role::Servers as u32 }>>,
	Json(server): Json<NewServer>,
) -> Result<Created<Json<CreatedServer>>> {
	let mut transaction = state.begin_transaction().await?;
	let api_key = rand::random::<NonZeroU32>();

	sqlx::query! {
		r#"
		INSERT INTO
		  Servers (name, ip_address, port, owned_by, api_key)
		VALUES
		  (?, ?, ?, ?, ?)
		"#,
		server.name,
		server.ip_address.ip().to_string(),
		server.ip_address.port(),
		server.owned_by,
		api_key.get(),
	}
	.execute(transaction.as_mut())
	.await
	.map_err(|err| {
		if err.is_foreign_key_violation() {
			Error::unknown("SteamID").with_detail(server.owned_by)
		} else {
			Error::from(err)
		}
	})?;

	let server_id = sqlx::query!("SELECT LAST_INSERT_ID() id")
		.fetch_one(transaction.as_mut())
		.await
		.map(|row| row.id as u16)?;

	audit! {
		"created server",
		id = %server_id,
		owner = %server.owned_by,
		approved_by = %session.user.steam_id
	};

	transaction.commit().await?;

	Ok(Created(Json(CreatedServer { server_id, api_key })))
}
