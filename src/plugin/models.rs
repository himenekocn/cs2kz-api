//! Types for representing CS2KZ plugin versions.

use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};
use sqlx::mysql::MySqlRow;
use sqlx::{FromRow, Row};
use utoipa::ToSchema;

use crate::make_id;

make_id!(PluginVersionID as u16);

/// A CS2KZ plugin version.
#[derive(Debug, Serialize, ToSchema)]
pub struct PluginVersion {
	/// The version's ID.
	pub id: PluginVersionID,

	/// The semver representation.
	#[schema(value_type = String)]
	pub semver: Version,

	/// The corresponding git revision (commit hash).
	pub git_revision: String,

	/// When this version was published.
	pub created_on: DateTime<Utc>,
}

impl FromRow<'_, MySqlRow> for PluginVersion {
	fn from_row(row: &MySqlRow) -> sqlx::Result<Self> {
		Ok(Self {
			id: row.try_get("id")?,
			semver: row
				.try_get::<&str, _>("semver")?
				.parse::<Version>()
				.map_err(|err| sqlx::Error::ColumnDecode {
					index: String::from("semver"),
					source: Box::new(err),
				})?,
			git_revision: row.try_get("git_revision")?,
			created_on: row.try_get("created_on")?,
		})
	}
}

/// Request body for submitting new plugin versions.
#[derive(Debug, Deserialize, ToSchema)]
pub struct NewPluginVersion {
	/// The semver representation.
	#[serde(deserialize_with = "crate::serde::semver::deserialize_plugin_version")]
	#[schema(value_type = String)]
	pub semver: Version,

	/// The corresponding git revision (commit hash).
	pub git_revision: String,
}

/// A newly created plugin version.
#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
pub struct CreatedPluginVersion {
	/// The version's ID.
	pub plugin_version_id: PluginVersionID,
}
