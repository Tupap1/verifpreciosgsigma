use crate::config::AppConfig;
use crate::{
    db::{self, ProductCache},
    updater,
};
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
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

#[derive(Deserialize)]
pub struct SyncQuery {
    pub sucursal: Option<String>,
}

pub async fn sync_productos(
    State(state): State<Arc<AppState>>,
    Query(params): Query<SyncQuery>,
) -> impl IntoResponse {
    let sucursal = params
        .sucursal
        .unwrap_or_else(|| state.config.sucursal.clone());

    match db::obtener_todos_productos_sync(&state.db_pool, &sucursal).await {
        Ok(lista) => (StatusCode::OK, Json(serde_json::to_value(lista).unwrap())),
        Err(err) => {
            tracing::error!("Error al realizar sync masivo: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Error interno en sync" })),
            )
        }
    }
}

pub async fn download_apk() -> impl IntoResponse {
    let path = "VerificadorGsigmaKiosk.apk";
    match tokio::fs::read(path).await {
        Ok(bytes) => (
            [
                (header::CONTENT_TYPE, "application/vnd.android.package-archive"),
                (
                    header::CONTENT_DISPOSITION,
                    "attachment; filename=\"VerificadorGsigmaKiosk.apk\"",
                ),
            ],
            bytes,
        )
            .into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "APK no encontrado en el servidor" })),
        )
            .into_response(),
    }
}

pub async fn trigger_update(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let config = state.config.clone();
    tokio::spawn(async move {
        updater::check_one_update(&config).await;
    });
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "buscando",
            "message": "Buscando actualizaciones en GitHub Releases..."
        })),
    )
}

#[derive(serde::Deserialize)]
pub struct PinRequest {
    pub pin: String,
}

pub async fn verify_admin_pin(Json(payload): Json<PinRequest>) -> impl IntoResponse {
    let valid_pin = "7612";
    if payload.pin.trim() == valid_pin {
        (
            StatusCode::OK,
            Json(serde_json::json!({ "success": true, "message": "PIN Correcto" })),
        )
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "success": false, "message": "PIN Incorrecto" })),
        )
    }
}

pub async fn get_admin_logs() -> impl IntoResponse {
    let log_dir = std::path::Path::new("logs");
    let mut log_content = String::new();

    if let Ok(entries) = std::fs::read_dir(log_dir) {
        let mut files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .collect();
        files.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());

        if let Some(latest_file) = files.last() {
            if let Ok(content) = std::fs::read_to_string(latest_file.path()) {
                let lines: Vec<&str> = content.lines().collect();
                let start = if lines.len() > 200 { lines.len() - 200 } else { 0 };
                log_content = lines[start..].join("\n");
            }
        }
    }

    if log_content.is_empty() {
        log_content = "No se encontraron logs recientes.".to_string();
    }

    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        log_content,
    )
}





