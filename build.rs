use std::io;

fn main() -> io::Result<()> {
    #[cfg(windows)] {
        use winres::WindowsResource;
        WindowsResource::new()
            .set_icon("icon.ico")
            .compile()?;
    }
    Ok(())
}