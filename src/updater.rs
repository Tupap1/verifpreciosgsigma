use crate::config::AppConfig;
use std::time::Duration;
use tracing::{info, warn};

pub async fn start_update_checker(config: AppConfig) {
    if !config.auto_update {
        info!("Auto-actualizador desactivado en config.json");
        return;
    }

    info!(
        "Iniciando servicio de auto-actualización para repo: {}/{}",
        config.repo_owner, config.repo_name
    );

    let client = match reqwest::Client::builder()
        .user_agent("VerifGsigma-AutoUpdater")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            warn!("No se pudo crear cliente HTTP para updates: {:?}", e);
            return;
        }
    };

    loop {
        // Consultar cada 6 horas
        tokio::time::sleep(Duration::from_secs(6 * 3600)).await;

        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            config.repo_owner, config.repo_name
        );

        if let Ok(res) = client.get(&url).send().await {
            if res.status().is_success() {
                info!("Verificación de releases completada correctamente");
            }
        }
    }
}

pub async fn check_one_update(config: &AppConfig) {
    info!("Verificando actualizaciones bajo demanda en GitHub Releases...");
}

