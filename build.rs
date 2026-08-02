#[cfg(windows)]
fn main() {
    let mut res = winres::WindowsResource::new();
    res.set("ProductName", "verifGsigma");
    res.set("FileDescription", "verifGsigma Servidor Local Verificador de Precios (BTW-One)");

    res.set("CompanyName", "BTW-One");
    res.set("LegalCopyright", "Copyright © 2026 BTW-One");
    res.set("OriginalFilename", "verifgsigma.exe");
    res.set("FileVersion", "1.4.6.0");
    res.set("ProductVersion", "1.4.6.0");

    if let Err(e) = res.compile() {
        eprintln!("Error al compilar recursos de Windows: {}", e);
        std::process::exit(1);
    }
}

#[cfg(not(windows))]
fn main() {}
