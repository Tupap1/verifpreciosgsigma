#[cfg(windows)]
fn main() {
    let version = std::fs::read_to_string("VERSION").unwrap_or_else(|_| "1.4.7".to_string());
    let version = version.trim().to_string();

    let mut res = winres::WindowsResource::new();
    res.set("ProductName", "verifGsigma");
    res.set("FileDescription", "verifGsigma Servidor Local Verificador de Precios (BTW-One)");
    res.set("CompanyName", "BTW-One");
    res.set("LegalCopyright", "Copyright © 2026 BTW-One");
    res.set("OriginalFilename", "verifgsigma.exe");
    res.set("FileVersion", &format!("{}.0", version));
    res.set("ProductVersion", &format!("{}.0", version));
    if let Err(e) = res.compile() {
        eprintln!("Error al compilar recursos de Windows: {}", e);
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn main() {}
