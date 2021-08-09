#[cfg(windows)] use winres::WindowsResource;

fn main() {
  if cfg!(target_os = "windows") {
      WindowsResource::new()
        .set_icon("icon.ico")
        .compile()
        .expect("Unable to set the icon!");
    }
}