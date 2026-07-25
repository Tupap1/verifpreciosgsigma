use serde::{Deserialize, Serialize};
use sqlx::{MySqlPool, Row};
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

pub async fn init_db_pool(db_url: &str) -> Result<MySqlPool, sqlx::Error> {
    info!("Conectando a base de datos MySQL en: {}", db_url);
    MySqlPool::connect(db_url).await
}

pub async fn buscar_producto(
    pool: &MySqlPool,
    sucursal: &str,
    codigo: &str,
) -> Result<Option<ProductoDto>, sqlx::Error> {
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

    if let Some(r) = row {
        let raw_cod: String = r.try_get("ARTCOD").unwrap_or_default();
        let raw_nom: String = r.try_get("ARTNOM").unwrap_or_default();
        let raw_pre: f64 = r.try_get("ARTPRE").unwrap_or(0.0);
        let raw_exi: f64 = r.try_get("ARTEXI").unwrap_or(0.0);
        let raw_uni: String = r.try_get("ARTUNIDAD").unwrap_or_else(|_| "UND".to_string());

        let producto = ProductoDto {
            codigo: raw_cod.trim().to_string(),
            nombre: raw_nom.trim().to_string(),
            precio: raw_pre,
            existencia: raw_exi,
            unidad: raw_uni.trim().to_string(),
            encontrado: true,
        };

        Ok(Some(producto))
    } else {
        Ok(None)
    }
}
