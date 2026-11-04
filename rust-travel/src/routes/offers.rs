//! `/offers?origin&destination&date&adults&cabin` — fare search, the CPU hot path (see `crate::pricing`)

use crate::AppState;
use axum::Router;

pub fn router() -> Router<AppState> {
    Router::new()
}
