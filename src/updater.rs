use crate::config::AppConfig;
use serde_json::Value;
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

    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await; // Verificar cada hora
        check_one_update(&config).await;
    }
}

pub async fn check_one_update(config: &AppConfig) {
    info!("Verificando actualizaciones bajo demanda en GitHub Releases...");

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

    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        config.repo_owner, config.repo_name
    );

    if let Ok(res) = client.get(&url).send().await {
        if res.status().is_success() {
            if let Ok(json) = res.json::<Value>().await {
                let latest_tag = json["tag_name"].as_str().unwrap_or("").trim_start_matches('v');
                let current_version = env!("CARGO_PKG_VERSION");

                if !latest_tag.is_empty() && latest_tag != current_version {
                    info!("Nueva versión encontrada en GitHub: v{}. Iniciando descarga...", latest_tag);

                    if let Some(assets) = json["assets"].as_array() {
                        for asset in assets {
                            let name = asset["name"].as_str().unwrap_or("");
                            if name == "verifgsigma.exe" {
                                if let Some(download_url) = asset["browser_download_url"].as_str() {
                                    download_and_apply_exe(&client, download_url).await;
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    info!("El servidor está en la versión más reciente (v{})", current_version);
                }
            }
        }
    }
}

async fn download_and_apply_exe(client: &reqwest::Client, download_url: &str) {
    match client.get(download_url).send().await {
        Ok(res) if res.status().is_success() => {
            if let Ok(bytes) = res.bytes().await {
                if let Ok(current_exe) = std::env::current_exe() {
                    let old_exe = current_exe.with_extension("exe.old");
                    
                    // Renombrar ejecutable en ejecución
                    let _ = std::fs::remove_file(&old_exe);
                    if std::fs::rename(&current_exe, &old_exe).is_ok() {
                        if std::fs::write(&current_exe, &bytes).is_ok() {
                            info!("✅ Actualización descargada exitosamente. Reiniciando servidor...");
                            let _ = std::process::Command::new(&current_exe).spawn();
                            std::process::exit(0);
                        } else {
                            // Revertir si falló la escritura
                            let _ = std::fs::rename(&old_exe, &current_exe);
                        }
                    }
                }
            }
        }
        _ => {
            warn!("Fallo al descargar ejecutable de actualización desde GitHub");
        }
    }
}
