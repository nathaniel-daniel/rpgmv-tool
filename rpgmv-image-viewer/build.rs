fn main() {
    let icon_path = "assets/icon.ico";

    if cfg!(windows) {
        println!("cargo:rerun-if-changed={icon_path}");

        let mut res = winres::WindowsResource::new();

        res.set_icon(icon_path);
        res.compile().unwrap();
    }
}
