//! `/bookings/{recordLocator}`, `/bookings?lastName=` (read-only; `POST`/`DELETE` stay on Java)

use crate::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
}
