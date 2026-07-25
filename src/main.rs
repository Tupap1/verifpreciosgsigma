mod api;
mod config;
mod db;
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
    tracing_subscriber::fmt::init();

    info!("Iniciando Servidor Verificador de Precios Local para Gsigma...");

    let config = AppConfig::load();
    info!("Configuración cargada: {:?}", config);

    // Inicializar pool de conexiones MySQL
    let db_pool = db::init_db_pool(&config.db_url).await.map_err(|e| {
        tracing::error!("Error al conectar a MySQL ({}): {:?}", config.db_url, e);
        e
    })?;

    info!("Conexión exitosa a la base de datos MySQL (pv)");

    let shared_state = Arc::new(AppState {
        db_pool,
        config: config.clone(),
    });

    // Iniciar worker de auto-actualización en segundo plano
    let config_clone = config.clone();
    tokio::spawn(async move {
        updater::start_update_checker(config_clone).await;
    });

    // Configurar middleware CORS para permitir peticiones desde cualquier tablet en la LAN
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Configurar rutas API y Servidor de Archivos Estáticos (Frontend Embebido)
    let app = Router::new()
        .route("/api/producto", get(api::get_producto))
        .route("/api/health", get(api::health_check))
        .route("/api/config", get(api::get_config))
        .fallback(static_handler)
        .layer(cors)
        .with_state(shared_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.puerto));
    info!("Servidor escuchando en http://0.0.0.0:{}", config.puerto);

    let listener = tokio::net::TcpListener::bind(addr).await?;
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
            Response::builder()
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(mime.as_ref()).unwrap(),
                )
                .body(Body::from(content.data))
                .unwrap()
        }
        None => {
            // Fallback a index.html para soporte de SPA routing
            if let Some(content) = Assets::get("index.html") {
                Response::builder()
                    .header(header::CONTENT_TYPE, HeaderValue::from_static("text/html"))
                    .body(Body::from(content.data))
                    .unwrap()
            } else {
                Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(Body::from("404 Not Found"))
                    .unwrap()
            }
        }
    }
}
