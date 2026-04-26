fn main() {
    // Only embed the .exe icon resource when building on a Windows host
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icons/mail.ico");
        if let Err(e) = res.compile() {
            eprintln!("cargo:warning=winres: {e}");
        }
    }
}
