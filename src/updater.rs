use crate::config::AppConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::{Arc, OnceLock, RwLock};
use std::time::Duration;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub state: String, // "idle", "checking", "downloading", "applying", "up_to_date", "error"
    pub version: String,
    pub message: String,
}

impl Default for UpdateStatus {
    fn default() -> Self {
        Self {
            state: "idle".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            message: format!("Sistema en v{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

static CURRENT_UPDATE_STATUS: OnceLock<Arc<RwLock<UpdateStatus>>> = OnceLock::new();

fn get_update_status_handle() -> Arc<RwLock<UpdateStatus>> {
    CURRENT_UPDATE_STATUS
        .get_or_init(|| Arc::new(RwLock::new(UpdateStatus::default())))
        .clone()
}

pub fn set_update_status(state: &str, version: &str, message: &str) {
    let handle = get_update_status_handle();
    if let Ok(mut lock) = handle.write() {
        lock.state = state.to_string();
        lock.version = version.to_string();
        lock.message = message.to_string();
    };
}

pub fn get_update_status() -> UpdateStatus {
    let handle = get_update_status_handle();
    let status = match handle.read() {
        Ok(lock) => lock.clone(),
        Err(_) => UpdateStatus::default(),
    };
    status
}



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
    let current_version = env!("CARGO_PKG_VERSION");
    info!("Verificando actualizaciones en GitHub Releases (repo: {}/{})...", config.repo_owner, config.repo_name);

    set_update_status("checking", current_version, "Buscando actualizaciones en GitHub Releases...");

    let client = match reqwest::Client::builder()
        .user_agent("VerifGsigma-AutoUpdater")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("No se pudo crear cliente HTTP: {:?}", e);
            warn!("{}", msg);
            set_update_status("error", current_version, &msg);
            return;
        }
    };

    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        config.repo_owner, config.repo_name
    );

    match client.get(&url).send().await {
        Ok(res) if res.status().is_success() => {
            if let Ok(json) = res.json::<Value>().await {
                let latest_tag = json["tag_name"].as_str().unwrap_or("").trim_start_matches('v');

                info!("Versión instalada: v{}, Versión más reciente en GitHub: v{}", current_version, latest_tag);

                if !latest_tag.is_empty() && latest_tag != current_version {
                    info!("🚀 Nueva versión v{} disponible. Buscando binario...", latest_tag);

                    if let Some(assets) = json["assets"].as_array() {
                        let mut found = false;
                        for asset in assets {
                            let name = asset["name"].as_str().unwrap_or("");
                            if name == "verifgsigma.exe" {
                                if let Some(download_url) = asset["browser_download_url"].as_str() {
                                    found = true;
                                    info!("Binario verifgsigma.exe encontrado. Descargando desde: {}", download_url);
                                    download_and_apply_exe(&client, download_url, latest_tag).await;
                                    break;
                                }
                            }
                        }
                        if !found {
                            let msg = format!("No se encontró el ejecutable verifgsigma.exe en la release v{}", latest_tag);
                            warn!("{}", msg);
                            set_update_status("error", current_version, &msg);
                        }
                    }
                } else {
                    let msg = format!("✅ El sistema ya se encuentra en la versión más reciente (v{})", current_version);
                    info!("{}", msg);
                    set_update_status("up_to_date", current_version, &msg);
                }
            }
        }
        Ok(res) => {
            let msg = format!("Respuesta HTTP no exitosa de GitHub (Status: {})", res.status());
            warn!("{}", msg);
            set_update_status("error", current_version, &msg);
        }
        Err(e) => {
            let msg = format!("Error al conectar con GitHub Releases: {:?}", e);
            warn!("{}", msg);
            set_update_status("error", current_version, &msg);
        }
    }
}

async fn download_and_apply_exe(client: &reqwest::Client, download_url: &str, target_version: &str) {
    info!("Descargando versión v{} desde: {}", target_version, download_url);
    set_update_status("downloading", target_version, &format!("🚀 Descargando versión v{} desde GitHub...", target_version));

    match client.get(download_url).send().await {
        Ok(res) if res.status().is_success() => {
            if let Ok(bytes) = res.bytes().await {
                info!("Descarga completada ({} bytes). Aplicando actualización...", bytes.len());
                set_update_status("applying", target_version, &format!("⚙️ Aplicando actualización v{}. Reiniciando servidor...", target_version));

                if let Ok(current_exe) = std::env::current_exe() {
                    let old_exe = current_exe.with_extension("exe.old");
                    
                    let _ = std::fs::remove_file(&old_exe);
                    if std::fs::rename(&current_exe, &old_exe).is_ok() {
                        if std::fs::write(&current_exe, &bytes).is_ok() {
                            info!("✅ Actualización v{} aplicada exitosamente. Reiniciando...", target_version);
                            let _ = std::process::Command::new(&current_exe).spawn();
                            std::process::exit(0);
                        } else {
                            let msg = "❌ Error al escribir el nuevo ejecutable. Revirtiendo...".to_string();
                            warn!("{}", msg);
                            set_update_status("error", target_version, &msg);
                            let _ = std::fs::rename(&old_exe, &current_exe);
                        }
                    } else {
                        let msg = "❌ No se pudo renombrar el ejecutable actual en uso. Revisa permisos.".to_string();
                        warn!("{}", msg);
                        set_update_status("error", target_version, &msg);
                    }
                }
            }
        }
        Err(e) => {
            let msg = format!("Error al descargar binario de actualización: {:?}", e);
            warn!("{}", msg);
            set_update_status("error", target_version, &msg);
        }
        _ => {
            let msg = "Fallo HTTP al descargar binario de actualización".to_string();
            warn!("{}", msg);
            set_update_status("error", target_version, &msg);
        }
    }
}
