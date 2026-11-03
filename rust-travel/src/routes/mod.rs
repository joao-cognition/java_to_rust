//! Routes under `/travel/api`, one module per work stream.

pub mod lookup;
pub mod offers;
pub mod reference;
pub mod schedule;

use crate::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(reference::router())
        .merge(schedule::router())
        .merge(lookup::router())
        .merge(offers::router())
}
