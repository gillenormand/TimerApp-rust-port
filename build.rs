fn main() {
    slint_build::compile("ui/appwindow.slint").unwrap();

    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=assets/icon.ico");

        let res = winresource::WindowsResource::new()
            .set_icon("assets/icon.ico")
            .set("ProductName", "appTimer")
            .set("FileDescription", "appTimer 0.8.5a")
            .compile();

        match res {
            Ok(()) => println!("cargo:warning=Windows resources embedded OK"),
            Err(e) => {
                // Print debug so we see the real windres stderr
                println!("cargo:warning=embed error: {:?}", e);
            }
        }
    }
}
