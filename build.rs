fn main() {
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("metadata/play.ico");
        res.compile().unwrap();
    }
}
