#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod api;
mod config;
mod db;
mod tray;
mod updater;

use api::AppState;
use axum::{
    body::Body,
    http::{header, HeaderValue, StatusCode, Uri},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use config::AppConfig;
use rust_embed::RustEmbed;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

#[derive(RustEmbed)]
#[folder = "frontend/build/"]
struct Assets;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let log_dir = exe_dir.join("logs");

    if let Err(e) = std::fs::create_dir_all(&log_dir) {
        eprintln!("Advertencia: No se pudo crear el directorio de logs en {:?}: {}", log_dir, e);
    }

    let file_appender = tracing_appender::rolling::daily(log_dir, "verifgsigma.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .init();

    info!("Iniciando Servidor Verificador de Precios Local para Gsigma...");

    let config = AppConfig::load();
    info!("Configuración cargada: {:?}", config);

    // Inicializar pool de conexiones MySQL (lazy)
    let db_pool = match db::init_db_pool(&config.db_url).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Error al inicializar pool MySQL ({}): {:?}", config.db_url, e);
            db::init_db_pool("mysql://root@localhost:3306/pv").await.unwrap()
        }
    };
    info!("Pool MySQL configurado exitosamente");

    let shared_state = Arc::new(AppState {
        db_pool,
        config: config.clone(),
        cache: db::ProductCache::default(),
    });

    let config_clone = config.clone();
    tokio::spawn(async move {
        updater::start_update_checker(config_clone).await;
    });

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/api/producto", get(api::get_producto))
        .route("/api/productos/sync", get(api::sync_productos))
        .route("/api/health", get(api::health_check))
        .route("/api/config", get(api::get_config))
        .route("/api/update", axum::routing::post(api::trigger_update))
        .route("/api/update/status", get(api::get_update_status))

        .route("/api/admin/verify-pin", axum::routing::post(api::verify_admin_pin))
        .route("/api/admin/logs", get(api::get_admin_logs))
        .route("/apk", get(api::download_apk))
        .fallback(static_handler)
        .layer(cors)
        .with_state(shared_state.clone());

    let cache_clone = shared_state.cache.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(600)).await;
            cache_clone.evict_expired();
        }
    });

    let candidate_ports = [config.puerto, 8080, 8085, 8090, 8081, 8888];
    let mut bound_listener = None;
    let mut actual_port = config.puerto;

    for &port in &candidate_ports {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => {
                bound_listener = Some(listener);
                actual_port = port;
                break;
            }
            Err(_) => {
                tracing::warn!("Puerto {} ocupado. Intentando puerto alternativo...", port);
            }
        }
    }

    let listener = match bound_listener {
        Some(l) => l,
        None => {
            let addr = SocketAddr::from(([0, 0, 0, 0], 0));
            let l = tokio::net::TcpListener::bind(addr).await?;
            actual_port = l.local_addr()?.port();
            l
        }
    };

    info!("Servidor escuchando exitosamente en http://0.0.0.0:{}", actual_port);

    #[cfg(target_os = "windows")]
    tray::start_system_tray(actual_port);

    axum::serve(listener, app).await?;

    Ok(())
}


async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    if path.is_empty() {
        path = "index.html".to_string();
    }

    match Assets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            let mime_val = match HeaderValue::from_str(mime.as_ref()) {
                Ok(v) => v,
                Err(_) => HeaderValue::from_static("application/octet-stream"),
            };
            Response::builder()
                .header(header::CONTENT_TYPE, mime_val)
                .body(Body::from(content.data))
                .unwrap_or_else(|_| Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty()).unwrap())
        }
        None => {
            // Fallback a index.html para soporte de SPA routing
            if let Some(content) = Assets::get("index.html") {
                Response::builder()
                    .header(header::CONTENT_TYPE, HeaderValue::from_static("text/html"))
                    .body(Body::from(content.data))
                    .unwrap_or_else(|_| Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty()).unwrap())
            } else {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("404 Not Found"))
                    .unwrap_or_else(|_| Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty()).unwrap())
            }
        }
    }
}
