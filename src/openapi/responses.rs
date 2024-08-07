// Everything in here should be self-explanatory, and doc comments would end up as descriptions in
// the OpenAPI spec, which we don't want.
#![allow(missing_docs, clippy::missing_docs_in_private_items)]

use std::collections::BTreeMap;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use utoipa::openapi::response::Response as ResponseSchema;
use utoipa::openapi::schema::Schema;
use utoipa::openapi::{ObjectBuilder, RefOr, SchemaType};
use utoipa::{IntoResponses, ToSchema};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginationResponse<T>
where
	T: ToSchema<'static>,
{
	pub total: u64,

	#[schema(inline)]
	pub results: Vec<T>,
}

#[derive(Debug, Serialize, IntoResponses)]
#[response(status = 200)]
pub struct Ok<T>(#[to_schema] pub T)
where
	T: ToSchema<'static>;

#[derive(Debug, Serialize)]
pub struct Created<T = ()>(pub T);

impl<T> IntoResponse for Created<T>
where
	T: IntoResponse,
{
	fn into_response(self) -> Response {
		(StatusCode::CREATED, self.0).into_response()
	}
}

impl<T> IntoResponses for Created<T>
where
	T: ToSchema<'static>,
{
	#[allow(clippy::missing_docs_in_private_items)]
	fn responses() -> BTreeMap<String, RefOr<ResponseSchema>> {
		#[derive(IntoResponses)]
		#[response(status = 201)]
		struct Helper<T>(#[to_schema] T)
		where
			T: ToSchema<'static>;

		Helper::<T>::responses()
	}
}

#[derive(Debug, Clone, Copy, Serialize, IntoResponses)]
#[response(status = 204)]
pub struct NoContent;

impl IntoResponse for NoContent {
	fn into_response(self) -> Response {
		StatusCode::NO_CONTENT.into_response()
	}
}

#[derive(Debug, Clone, Copy, Serialize, IntoResponses)]
#[response(status = 303)]
pub struct SeeOther;

#[derive(Debug, Clone, Copy, Serialize, IntoResponses)]
#[response(status = 400)]
pub struct BadRequest;

#[derive(Debug, Clone, Copy, Serialize, IntoResponses)]
#[response(status = 401)]
pub struct Unauthorized;

#[derive(Debug, Clone, Copy, Serialize, IntoResponses)]
#[response(status = 404)]
pub struct NotFound;

#[derive(Debug, Clone, Copy, Serialize, IntoResponses)]
#[response(status = 409)]
pub struct Conflict;

#[derive(Debug, Clone, Copy, Serialize, IntoResponses)]
#[response(status = 422)]
pub struct UnprocessableEntity;

#[derive(Debug, Clone, Copy, Serialize, IntoResponses)]
#[response(status = 502)]
pub struct BadGateway;

#[derive(Debug, Clone, Copy)]
pub struct Object;

impl<'s> ToSchema<'s> for Object {
	fn schema() -> (&'s str, RefOr<Schema>) {
		(
			"Object",
			ObjectBuilder::new()
				.title(Some("Object"))
				.description(Some("arbitrary key-value pairs"))
				.schema_type(SchemaType::Object)
				.into(),
		)
	}
}
