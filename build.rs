#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set("ProductName", "BTW-One Verificador de Precios");
    res.set("FileDescription", "BTW-One Servidor Local Verificador de Precios");
    res.set("CompanyName", "BTW-One");
    res.set("LegalCopyright", "Copyright © 2026 BTW-One");
    res.set("OriginalFilename", "verifgsigma.exe");
    res.set("FileVersion", "1.0.0.0");
    res.set("ProductVersion", "1.0.0.0");
    if let Err(e) = res.compile() {
        eprintln!("Error al compilar recursos de Windows: {}", e);
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn main() {}
