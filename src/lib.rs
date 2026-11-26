//! Calculadora para Android escrita em Rust.
//!
//! - `calc`: lógica dos cálculos (testada com `cargo test`)
//! - `ui`:   interface feita com egui/eframe
//!
//! No Android o ponto de entrada é `android_main`. No PC use `cargo run --example desktop`.

pub mod calc;
mod ui;

pub use ui::App;

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));

    let options = eframe::NativeOptions {
        android_app: Some(app),
        ..Default::default()
    };
    if let Err(e) = eframe::run_native("Calculadora", options, Box::new(|cc| Ok(Box::new(App::new(cc))))) {
        log::error!("Erro ao iniciar a calculadora: {e}");
    }
}
