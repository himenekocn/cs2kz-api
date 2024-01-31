use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;

use crate::auth::Session;

/// Middleware for authenticating users who logged in with Steam.
#[tracing::instrument(skip(request, next))]
pub async fn layer<const REQUIRED_FLAGS: u32>(
	session: Session<REQUIRED_FLAGS>,
	mut request: Request,
	next: Next,
) -> (Session<REQUIRED_FLAGS>, Response) {
	request.extensions_mut().insert(session.clone());

	(session, next.run(request).await)
}
