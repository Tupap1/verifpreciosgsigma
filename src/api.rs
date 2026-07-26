use crate::config::AppConfig;
use crate::db::{self, ProductCache};
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use sqlx::MySqlPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: MySqlPool,
    pub config: AppConfig,
    pub cache: ProductCache,
}

#[derive(Deserialize)]
pub struct ProductoQuery {
    pub codigo: String,
    pub sucursal: Option<String>,
}

pub async fn get_producto(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ProductoQuery>,
) -> impl IntoResponse {
    let sucursal = params
        .sucursal
        .unwrap_or_else(|| state.config.sucursal.clone());

    if params.codigo.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "El código de producto no puede estar vacío",
                "encontrado": false
            })),
        );
    }

    match db::buscar_producto_cached(&state.db_pool, &state.cache, &sucursal, params.codigo.trim()).await {
        Ok(Some(prod)) => (StatusCode::OK, Json(serde_json::to_value(prod).unwrap())),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Producto no encontrado",
                "encontrado": false
            })),
        ),
        Err(err) => {
            tracing::error!("Error al consultar la base de datos: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "Error interno al consultar la base de datos",
                    "encontrado": false
                })),
            )
        }
    }
}

pub async fn health_check(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db_ok = sqlx::query("SELECT 1").execute(&state.db_pool).await.is_ok();
    
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "online",
            "db_connected": db_ok,
            "version": env!("CARGO_PKG_VERSION")
        })),
    )
}

pub async fn get_config(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    (StatusCode::OK, Json(state.config.clone()))
}
