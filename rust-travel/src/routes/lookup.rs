//! travelers, bookings. Implemented by its work stream.

use crate::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
}
