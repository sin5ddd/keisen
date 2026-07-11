fn main() {
    // Windows の exe ファイルアイコン（エクスプローラー表示用）
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon.ico");
        if let Err(e) = res.compile() {
            // クロスコンパイル等で失敗してもビルド自体は続行
            println!("cargo:warning=winresource icon embed failed: {e}");
        }
    }
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=assets/icon.png");
}
