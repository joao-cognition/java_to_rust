//! `/flights?origin&destination&date`, `/flights/{id}`

use crate::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
}
