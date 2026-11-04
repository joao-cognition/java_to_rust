//! `/airports`, `/airports/{iata}`, `/airlines`, `/airlines/{iata}`

use crate::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
}
