//! Standalone translator server: reuses the app's LibreTranslate-compatible
//! HTTP surface so any client that speaks it can point at 127.0.0.1:5050.
//! usage: market_server <catalog-index.json> <pack-dir> [port]
use std::sync::Arc;

use translator::catalog::FsPackInstallChecker;
use translator::font_provider::NoFontProvider;
use translator::http::{start, BindAddress, HttpServerConfig, OcrSettings};
use translator::{BackgroundMode, TranslatorSession};

fn usage() -> ! {
    eprintln!("usage: market_server <catalog-index.json> <pack-dir> [port]");
    std::process::exit(2);
}

fn main() {
    let mut args = std::env::args().skip(1);
    let index_path = args.next().unwrap_or_else(|| usage());
    let base_dir = args.next().unwrap_or_else(|| usage());
    let port: u16 = args.next().and_then(|p| p.parse().ok()).unwrap_or(5050);

    let catalog_json = match std::fs::read_to_string(&index_path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("cannot read {index_path}: {error}");
            std::process::exit(1);
        }
    };
    let checker = FsPackInstallChecker::new(base_dir.clone());
    let session = match TranslatorSession::open(&catalog_json, None, base_dir.clone(), &checker) {
        Ok(session) => Arc::new(session),
        Err(error) => {
            eprintln!("catalog rejected: {error}");
            std::process::exit(1);
        }
    };

    let source = {
        let session = Arc::clone(&session);
        move || Some(Arc::clone(&session))
    };
    let config = HttpServerConfig {
        bind: BindAddress::Localhost,
        port,
        output_dir: base_dir.clone(),
        ocr: OcrSettings {
            max_image_size: 0,
            min_confidence: 0,
            background_mode: BackgroundMode::AutoDetect,
        },
        translate_pdf_images: false,
    };

    match start(config, Arc::new(source), Arc::new(NoFontProvider)) {
        Ok(_server) => {
            println!("market_server listening on 127.0.0.1:{port} packs={base_dir}");
            loop {
                std::thread::sleep(std::time::Duration::from_secs(3600));
            }
        }
        Err(error) => {
            eprintln!("cannot start: {error}");
            std::process::exit(1);
        }
    }
}
