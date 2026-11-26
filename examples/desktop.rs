//! Roda a calculadora no PC, numa janela com tamanho de celular.
//! Uso: cargo run --example desktop
//! Opcional: CALC_DEMO=media|ponderada|datas|variacao|padrao abre direto numa tela com dados de exemplo.

fn main() -> eframe::Result {
    let demo = std::env::var("CALC_DEMO").unwrap_or_default();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([400.0, 820.0])
            .with_title("Calculadora"),
        ..Default::default()
    };
    eframe::run_native(
        "Calculadora",
        options,
        Box::new(move |cc| Ok(Box::new(calculadora::App::new(cc).demo(&demo)))),
    )
}
