use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlPoolOptions, MySqlPool, Row};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductoDto {
    pub codigo: String,
    pub nombre: String,
    pub precio: f64,
    pub existencia: f64,
    pub unidad: String,
    pub encontrado: bool,
}

#[derive(Clone)]
pub struct CacheItem {
    pub producto: Option<ProductoDto>,
    pub fetched_at: Instant,
}

#[derive(Clone, Default)]
pub struct ProductCache {
    // Cache key: "sucursal:codigo", TTL: 5 minutes
    items: Arc<RwLock<HashMap<String, CacheItem>>>,
}

impl ProductCache {
    pub fn get(&self, sucursal: &str, codigo: &str) -> Option<Option<ProductoDto>> {
        let key = format!("{}:{}", sucursal, codigo);
        let guard = self.items.read().ok()?;
        if let Some(item) = guard.get(&key) {
            // Cache TTL: 5 minutes (300 seconds)
            if item.fetched_at.elapsed() < Duration::from_secs(300) {
                return Some(item.producto.clone());
            }
        }
        None
    }

    pub fn set(&self, sucursal: &str, codigo: &str, producto: Option<ProductoDto>) {
        let key = format!("{}:{}", sucursal, codigo);
        if let Ok(mut guard) = self.items.write() {
            guard.insert(
                key,
                CacheItem {
                    producto,
                    fetched_at: Instant::now(),
                },
            );
        }
    }

    pub fn evict_expired(&self) {
        if let Ok(mut guard) = self.items.write() {
            let ttl = Duration::from_secs(300);
            guard.retain(|_, item| item.fetched_at.elapsed() < ttl);
        }
    }
}

pub async fn init_db_pool(db_url: &str) -> Result<MySqlPool, sqlx::Error> {
    info!("Conectando y pre-calentando pool MySQL en: {}", db_url);
    
    MySqlPoolOptions::new()
        .max_connections(20)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(600))
        .connect(db_url)
        .await
}

pub async fn buscar_producto_cached(
    pool: &MySqlPool,
    cache: &ProductCache,
    sucursal: &str,
    codigo: &str,
) -> Result<Option<ProductoDto>, sqlx::Error> {
    // 1. Revisar caché en memoria (Respuesta ultra-rápida en 0.05ms)
    if let Some(cached_result) = cache.get(sucursal, codigo) {
        return Ok(cached_result);
    }

    // 2. Si no está en caché, consultar MySQL
    let query = "
        SELECT 
            a.ARTCOD, 
            p.PASNOM AS ARTNOM, 
            a.ARTPRE, 
            a.ARTEXI,
            p.PASUNIMED AS ARTUNIDAD
        FROM artic a 
        JOIN pas p ON a.ARTCOD = p.PAS 
        LEFT JOIN pascodal ca ON a.ARTCOD = ca.PASCOD 
        WHERE a.SUCCOD = ? AND (a.ARTCOD = ? OR ca.PASCODALT = ?) 
        LIMIT 1;
    ";

    let row = sqlx::query(query)
        .bind(sucursal)
        .bind(codigo)
        .bind(codigo)
        .fetch_optional(pool)
        .await?;

    let resultado = if let Some(r) = row {
        let raw_cod: String = r.try_get("ARTCOD").unwrap_or_default();
        let raw_nom: String = r.try_get("ARTNOM").unwrap_or_default();
        let raw_pre: f64 = r.try_get("ARTPRE").unwrap_or(0.0);
        let raw_exi: f64 = r.try_get("ARTEXI").unwrap_or(0.0);
        let raw_uni: String = r.try_get("ARTUNIDAD").unwrap_or_else(|_| "UND".to_string());

        Some(ProductoDto {
            codigo: raw_cod.trim().to_string(),
            nombre: raw_nom.trim().to_string(),
            precio: raw_pre,
            existencia: raw_exi,
            unidad: raw_uni.trim().to_string(),
            encontrado: true,
        })
    } else {
        None
    };

    // Guardar en caché para futuras consultas instantáneas
    cache.set(sucursal, codigo, resultado.clone());

    Ok(resultado)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductoSyncDto {
    pub c: String, // codigo
    pub n: String, // nombre
    pub p: f64,    // precio
    pub e: f64,    // existencia
    pub u: String, // unidad
}

pub async fn obtener_todos_productos_sync(
    pool: &MySqlPool,
    sucursal: &str,
) -> Result<Vec<ProductoSyncDto>, sqlx::Error> {
    let query = "
        SELECT 
            a.ARTCOD, 
            p.PASNOM AS ARTNOM, 
            a.ARTPRE, 
            a.ARTEXI,
            p.PASUNIMED AS ARTUNIDAD
        FROM artic a 
        JOIN pas p ON a.ARTCOD = p.PAS 
        WHERE a.SUCCOD = ?;
    ";

    let rows = sqlx::query(query)
        .bind(sucursal)
        .fetch_all(pool)
        .await?;

    let mut lista = Vec::new();
    for r in rows {
        let raw_cod: String = r.try_get("ARTCOD").unwrap_or_default();
        let raw_nom: String = r.try_get("ARTNOM").unwrap_or_default();
        let raw_pre: f64 = r.try_get("ARTPRE").unwrap_or(0.0);
        let raw_exi: f64 = r.try_get("ARTEXI").unwrap_or(0.0);
        let raw_uni: String = r.try_get("ARTUNIDAD").unwrap_or_else(|_| "UND".to_string());

        lista.push(ProductoSyncDto {
            c: raw_cod.trim().to_string(),
            n: raw_nom.trim().to_string(),
            p: raw_pre,
            e: raw_exi,
            u: raw_uni.trim().to_string(),
        });
    }

    Ok(lista)
}

