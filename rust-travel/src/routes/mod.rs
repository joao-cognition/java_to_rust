//! `/travel/api` routes, one module per resource so each endpoint track owns its own file.

pub mod bookings;
pub mod flights;
pub mod offers;
pub mod reference;
pub mod smoke;
pub mod travelers;

use crate::AppState;
use axum::Router;

pub fn api() -> Router<AppState> {
    Router::new()
        .merge(reference::router())
        .merge(flights::router())
        .merge(travelers::router())
        .merge(bookings::router())
        .merge(offers::router())
        .merge(smoke::router())
}
