//! Interface gráfica (egui). Toda a entrada é feita por um teclado próprio na tela,
//! porque o teclado virtual do Android não abre de forma confiável em apps nativos.

use chrono::{Local, NaiveDate};
use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Key, Layout, Margin, RichText, Sense, Stroke, TextStyle,
    Vec2,
};

use crate::calc::{self, Variacao};

// ---------------------------------------------------------------------------
// Cores
// ---------------------------------------------------------------------------

const FUNDO: Color32 = Color32::from_rgb(0x12, 0x14, 0x18);
const PAINEL: Color32 = Color32::from_rgb(0x1c, 0x1f, 0x26);
const CAMPO: Color32 = Color32::from_rgb(0x23, 0x27, 0x30);
const CAMPO_ATIVO: Color32 = Color32::from_rgb(0x2b, 0x2f, 0x3a);
const BORDA: Color32 = Color32::from_rgb(0x34, 0x39, 0x45);
const TECLA_NUM: Color32 = Color32::from_rgb(0x2a, 0x2e, 0x37);
const TECLA_OP: Color32 = Color32::from_rgb(0x36, 0x3b, 0x47);
const DESTAQUE: Color32 = Color32::from_rgb(0xff, 0x9f, 0x0a);
const TEXTO: Color32 = Color32::from_rgb(0xf2, 0xf3, 0xf5);
const TEXTO_FRACO: Color32 = Color32::from_rgb(0x9a, 0xa0, 0xab);
const VERDE: Color32 = Color32::from_rgb(0x34, 0xc7, 0x59);
const VERMELHO: Color32 = Color32::from_rgb(0xff, 0x5a, 0x52);

// ---------------------------------------------------------------------------
// Teclado da tela
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Debug)]
enum Acao {
    Ch(char),
    Apagar,
    Limpar,
    Enter,
    Proximo,
    Hoje,
    Sinal,
    Parenteses,
}

#[derive(Clone, Copy, PartialEq)]
enum Estilo {
    Num,
    Op,
    Destaque,
}

struct Tecla {
    rotulo: &'static str,
    acao: Acao,
    span: f32,
    estilo: Estilo,
}

const fn num(r: &'static str, c: char) -> Tecla {
    Tecla { rotulo: r, acao: Acao::Ch(c), span: 1.0, estilo: Estilo::Num }
}
const fn op(r: &'static str, a: Acao) -> Tecla {
    Tecla { rotulo: r, acao: a, span: 1.0, estilo: Estilo::Op }
}
const fn larga(r: &'static str, a: Acao, span: f32, estilo: Estilo) -> Tecla {
    Tecla { rotulo: r, acao: a, span, estilo }
}

const APAGAR: &str = "Apagar";

const TECLADO_PADRAO: [[Tecla; 4]; 5] = [
    [op("C", Acao::Limpar), op("( )", Acao::Parenteses), op("%", Acao::Ch('%')), op("÷", Acao::Ch('÷'))],
    [num("7", '7'), num("8", '8'), num("9", '9'), op("×", Acao::Ch('×'))],
    [num("4", '4'), num("5", '5'), num("6", '6'), op("−", Acao::Ch('-'))],
    [num("1", '1'), num("2", '2'), num("3", '3'), op("+", Acao::Ch('+'))],
    [op(APAGAR, Acao::Apagar), num("0", '0'), num(",", ','), larga("=", Acao::Enter, 1.0, Estilo::Destaque)],
];

/// Desenha o teclado e devolve a tecla tocada (se houver).
fn teclado(ui: &mut egui::Ui, linhas: &[&[Tecla]], altura_tecla: f32) -> Option<Acao> {
    let mut res = None;
    let gap = 8.0;
    ui.spacing_mut().item_spacing = Vec2::splat(gap);
    let w = ui.available_width();
    for linha in linhas {
        let total: f32 = linha.iter().map(|t| t.span).sum();
        let unidade = (w - gap * (linha.len() as f32 - 1.0)) / total;
        ui.horizontal(|ui| {
            for t in linha.iter() {
                let largura = t.span * unidade;
                let (fundo, cor) = match t.estilo {
                    Estilo::Num => (TECLA_NUM, TEXTO),
                    Estilo::Op => (TECLA_OP, DESTAQUE),
                    Estilo::Destaque => (DESTAQUE, Color32::BLACK),
                };
                let tamanho = if t.rotulo.chars().count() > 3 { 20.0 } else { 28.0 };
                let botao = egui::Button::new(RichText::new(t.rotulo).size(tamanho).color(cor))
                    .fill(fundo)
                    .stroke(Stroke::NONE)
                    .corner_radius(CornerRadius::same(14));
                if ui.add_sized([largura, altura_tecla], botao).clicked() {
                    res = Some(t.acao);
                }
            }
        });
    }
    res
}

/// Converte teclas de um teclado físico (PC ou teclado Bluetooth) em ações.
fn teclado_fisico(ctx: &egui::Context) -> Vec<Acao> {
    let mut out = Vec::new();
    ctx.input(|i| {
        for ev in &i.events {
            match ev {
                egui::Event::Text(t) => {
                    for c in t.chars() {
                        match c {
                            '0'..='9' | ',' | '+' | '-' | '%' | '(' | ')' => out.push(Acao::Ch(c)),
                            '.' => out.push(Acao::Ch(',')),
                            '*' | 'x' | 'X' => out.push(Acao::Ch('×')),
                            '/' => out.push(Acao::Ch('÷')),
                            '=' => out.push(Acao::Enter),
                            _ => {}
                        }
                    }
                }
                egui::Event::Key { key, pressed: true, .. } => match key {
                    Key::Backspace => out.push(Acao::Apagar),
                    Key::Delete => out.push(Acao::Limpar),
                    Key::Enter => out.push(Acao::Enter),
                    Key::Tab => out.push(Acao::Proximo),
                    _ => {}
                },
                _ => {}
            }
        }
    });
    out
}

// ---------------------------------------------------------------------------
// Componentes visuais
// ---------------------------------------------------------------------------

/// Campo de entrada "tocável". Devolve true se foi tocado.
fn campo(ui: &mut egui::Ui, rotulo: &str, texto: &str, placeholder: &str, ativo: bool) -> bool {
    ui.label(RichText::new(rotulo).color(TEXTO_FRACO).size(17.0));
    let w = ui.available_width();
    let (fundo, borda) = if ativo {
        (CAMPO_ATIVO, Stroke::new(2.0, DESTAQUE))
    } else {
        (CAMPO, Stroke::new(1.0, BORDA))
    };
    let conteudo = if texto.is_empty() {
        RichText::new(placeholder).color(TEXTO_FRACO)
    } else {
        RichText::new(texto).color(TEXTO)
    };
    let botao = egui::Button::new(conteudo.size(26.0))
        .fill(fundo)
        .stroke(borda)
        .corner_radius(CornerRadius::same(12));
    ui.add_sized([w, 58.0], botao).clicked()
}

fn cartao<R>(ui: &mut egui::Ui, conteudo: impl FnOnce(&mut egui::Ui) -> R) -> R {
    egui::Frame::new()
        .fill(PAINEL)
        .corner_radius(CornerRadius::same(16))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            conteudo(ui)
        })
        .inner
}

/// Botão de duas opções (ex.: Simples / Ponderada).
fn alternador(ui: &mut egui::Ui, opcoes: [&str; 2], selecionado: &mut usize) {
    let w = (ui.available_width() - 8.0) / 2.0;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        for (i, nome) in opcoes.iter().enumerate() {
            let ativo = *selecionado == i;
            let b = egui::Button::new(RichText::new(*nome).size(19.0).color(if ativo { Color32::BLACK } else { TEXTO }))
                .fill(if ativo { DESTAQUE } else { CAMPO })
                .stroke(Stroke::NONE)
                .corner_radius(CornerRadius::same(10));
            if ui.add_sized([w, 44.0], b).clicked() {
                *selecionado = i;
            }
        }
    });
}

fn botao_menu(ui: &mut egui::Ui, icone: &str, titulo: &str, descricao: &str) -> bool {
    let w = ui.available_width();
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 92.0), Sense::click());
    let fundo = if resp.is_pointer_button_down_on() {
        CAMPO_ATIVO
    } else {
        PAINEL
    };
    let p = ui.painter();
    p.rect_filled(rect, CornerRadius::same(18), fundo);
    let circulo = egui::pos2(rect.left() + 46.0, rect.center().y);
    p.circle_filled(circulo, 28.0, TECLA_OP);
    p.text(circulo, Align2::CENTER_CENTER, icone, FontId::proportional(26.0), DESTAQUE);
    let x = rect.left() + 92.0;
    p.text(
        egui::pos2(x, rect.center().y - 13.0),
        Align2::LEFT_CENTER,
        titulo,
        FontId::proportional(23.0),
        TEXTO,
    );
    p.text(
        egui::pos2(x, rect.center().y + 16.0),
        Align2::LEFT_CENTER,
        descricao,
        FontId::proportional(16.0),
        TEXTO_FRACO,
    );
    p.text(
        egui::pos2(rect.right() - 20.0, rect.center().y),
        Align2::RIGHT_CENTER,
        "›",
        FontId::proportional(32.0),
        TEXTO_FRACO,
    );
    resp.clicked()
}

// ---------------------------------------------------------------------------
// Edição de textos numéricos
// ---------------------------------------------------------------------------

fn editar_numero(s: &mut String, a: Acao, permite_negativo: bool) {
    match a {
        Acao::Ch(c) if c.is_ascii_digit() => {
            if s.chars().filter(|c| c.is_ascii_digit()).count() >= 15 {
                return;
            }
            if s == "0" {
                s.clear();
            } else if s == "-0" {
                s.pop();
            }
            s.push(c);
        }
        Acao::Ch(',') => {
            if !s.contains(',') {
                if s.is_empty() || s == "-" {
                    s.push('0');
                }
                s.push(',');
            }
        }
        Acao::Ch('-') | Acao::Sinal if permite_negativo => {
            if s.starts_with('-') {
                s.remove(0);
            } else {
                s.insert(0, '-');
            }
        }
        Acao::Apagar => {
            s.pop();
        }
        Acao::Limpar => s.clear(),
        _ => {}
    }
}

/// Exibe o número digitado com separador de milhar: "1234567,5" -> "1.234.567,5".
fn exibir_numero(s: &str) -> String {
    let (sinal, resto) = match s.strip_prefix('-') {
        Some(r) => ("-", r),
        None => ("", s),
    };
    let (int, frac) = match resto.split_once(',') {
        Some((i, f)) => (i, Some(f)),
        None => (resto, None),
    };
    let digitos: Vec<char> = int.chars().filter(|c| *c != '.').collect();
    let mut g = String::from(sinal);
    for (i, c) in digitos.iter().enumerate() {
        if i > 0 && (digitos.len() - i) % 3 == 0 {
            g.push('.');
        }
        g.push(*c);
    }
    if let Some(f) = frac {
        g.push(',');
        g.push_str(f);
    }
    g
}

/// Agrupa os milhares de todos os números de uma expressão, para exibição.
fn exibir_expressao(expr: &str) -> String {
    let mut out = String::new();
    let mut numero = String::new();
    for c in expr.chars() {
        if c.is_ascii_digit() || c == ',' {
            numero.push(c);
        } else {
            if !numero.is_empty() {
                out.push_str(&exibir_numero(&numero));
                numero.clear();
            }
            match c {
                '+' => out.push_str(" + "),
                '-' => out.push_str(" − "),
                '×' => out.push_str(" × "),
                '÷' => out.push_str(" ÷ "),
                _ => out.push(c),
            }
        }
    }
    out.push_str(&exibir_numero(&numero));
    out.replace("(  − ", "(−").trim_start().to_string()
}

fn plural(n: i64, singular: &str, plural: &str) -> String {
    format!("{} {}", calc::format_num(n as f64, 0), if n.abs() == 1 { singular } else { plural })
}

fn juntar(partes: &[String]) -> String {
    match partes.len() {
        0 => String::new(),
        1 => partes[0].clone(),
        n => format!("{} e {}", partes[..n - 1].join(", "), partes[n - 1]),
    }
}

fn hoje() -> NaiveDate {
    Local::now().date_naive()
}

// ---------------------------------------------------------------------------
// Aplicativo
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum Tela {
    Menu,
    Media,
    Datas,
    Variacao,
    Padrao,
}

pub struct App {
    tela: Tela,

    // Média
    media_tipo: usize, // 0 = simples, 1 = ponderada
    media_valor: String,
    media_peso: String,
    media_campo: usize,
    media_itens: Vec<(f64, f64)>,

    // Dias entre datas
    data_ini: String,
    data_fim: String,
    data_campo: usize,
    incluir_ultimo: bool,

    // Desconto / aumento
    var_orig: String,
    var_novo: String,
    var_campo: usize,

    // Calculadora padrão
    expr: String,
    anterior: String,
    calculado: bool,
    erro: Option<&'static str>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configurar_estilo(&cc.egui_ctx);
        Self {
            tela: Tela::Menu,
            media_tipo: 0,
            media_valor: String::new(),
            media_peso: String::new(),
            media_campo: 0,
            media_itens: Vec::new(),
            data_ini: String::new(),
            data_fim: calc::digitos_da_data(hoje()),
            data_campo: 0,
            incluir_ultimo: false,
            var_orig: String::new(),
            var_novo: String::new(),
            var_campo: 0,
            expr: String::new(),
            anterior: String::new(),
            calculado: false,
            erro: None,
        }
    }

    /// Abre direto numa tela com dados de exemplo (usado só para testes/capturas no PC).
    pub fn demo(mut self, tela: &str) -> Self {
        let teclar = |app: &mut Self, s: &str, f: fn(&mut Self, Acao)| {
            for c in s.chars() {
                f(app, if c == '=' { Acao::Enter } else if c == '>' { Acao::Proximo } else { Acao::Ch(c) });
            }
        };
        match tela {
            "media" => {
                self.tela = Tela::Media;
                teclar(&mut self, "7,5=8=9,25=", Self::acao_media);
            }
            "ponderada" => {
                self.tela = Tela::Media;
                self.media_tipo = 1;
                teclar(&mut self, "8>2=6>1=", Self::acao_media);
                teclar(&mut self, "10", Self::acao_media);
            }
            "datas" => {
                self.tela = Tela::Datas;
                teclar(&mut self, "01012026", Self::acao_datas);
            }
            "variacao" => {
                self.tela = Tela::Variacao;
                teclar(&mut self, "1250>999,9", |a, x| {
                    if x == Acao::Proximo {
                        a.var_campo = 1;
                    } else {
                        let alvo = if a.var_campo == 0 { &mut a.var_orig } else { &mut a.var_novo };
                        editar_numero(alvo, x, true);
                    }
                });
            }
            "padrao" => {
                self.tela = Tela::Padrao;
                teclar(&mut self, "1500+10%=×(2+3", Self::acao_padrao);
            }
            _ => {}
        }
        self
    }

    fn cabecalho(&mut self, ui: &mut egui::Ui, titulo: &str) {
        egui::Panel::top("cabecalho")
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::new().fill(FUNDO).inner_margin(Margin { left: 12, right: 12, top: 14, bottom: 8 }))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let voltar = egui::Button::new(RichText::new("‹ Menu").size(20.0).color(DESTAQUE))
                        .fill(PAINEL)
                        .stroke(Stroke::NONE)
                        .corner_radius(CornerRadius::same(10));
                    if ui.add_sized([96.0, 42.0], voltar).clicked() {
                        self.tela = Tela::Menu;
                    }
                    ui.add_space(8.0);
                    ui.label(RichText::new(titulo).size(24.0).strong().color(TEXTO));
                });
            });
    }

    /// Painel inferior com o teclado. Devolve a tecla tocada.
    fn painel_teclado(ui: &mut egui::Ui, linhas: &[&[Tecla]]) -> Option<Acao> {
        let altura_tela = ui.ctx().content_rect().height();
        let altura_tecla = ((altura_tela * 0.40) / linhas.len() as f32 - 8.0).clamp(46.0, 76.0);
        egui::Panel::bottom("teclado")
            .resizable(false)
            .show_separator_line(false)
            .frame(egui::Frame::new().fill(FUNDO).inner_margin(Margin { left: 12, right: 12, top: 8, bottom: 16 }))
            .show(ui, |ui| teclado(ui, linhas, altura_tecla))
            .inner
    }

    fn area_central(ui: &mut egui::Ui, conteudo: impl FnOnce(&mut egui::Ui)) {
        egui::CentralPanel::no_frame()
            .frame(egui::Frame::new().fill(FUNDO).inner_margin(Margin::symmetric(12, 4)))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
                    conteudo(ui);
                    ui.add_space(8.0);
                });
            });
    }

    // ------------------------------------------------------------------ Menu

    fn tela_menu(&mut self, ui: &mut egui::Ui) {
        egui::CentralPanel::no_frame()
            .frame(egui::Frame::new().fill(FUNDO).inner_margin(Margin::same(16)))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
                    ui.add_space(24.0);
                    ui.label(RichText::new("Calculadora").size(36.0).strong().color(TEXTO));
                    ui.label(RichText::new("Escolha uma opção").size(18.0).color(TEXTO_FRACO));
                    ui.add_space(20.0);
                    ui.spacing_mut().item_spacing.y = 12.0;
                    if botao_menu(ui, "∑", "Média", "Simples ou ponderada") {
                        self.tela = Tela::Media;
                    }
                    if botao_menu(ui, "31", "Dias entre datas", "Por padrão, até hoje") {
                        self.tela = Tela::Datas;
                    }
                    if botao_menu(ui, "%", "Desconto ou aumento", "Variação em % entre dois valores") {
                        self.tela = Tela::Variacao;
                    }
                    if botao_menu(ui, "÷", "Calculadora padrão", "+  −  ×  ÷  %") {
                        self.tela = Tela::Padrao;
                    }
                });
            });
    }

    // ----------------------------------------------------------------- Média

    fn tela_media(&mut self, ui: &mut egui::Ui, fisicas: &[Acao]) {
        self.cabecalho(ui, "Média");
        let ponderada = self.media_tipo == 1;
        let teclas: [&[Tecla]; 4] = [
            &[num("7", '7'), num("8", '8'), num("9", '9'), op(APAGAR, Acao::Apagar)],
            &[num("4", '4'), num("5", '5'), num("6", '6'), op("C", Acao::Limpar)],
            &[num("1", '1'), num("2", '2'), num("3", '3'), op("±", Acao::Sinal)],
            &[num(",", ','), num("0", '0'), larga("Adicionar", Acao::Enter, 2.0, Estilo::Destaque)],
        ];
        let toque = Self::painel_teclado(ui, &teclas);
        for a in toque.into_iter().chain(fisicas.iter().copied()) {
            self.acao_media(a);
        }

        let mut tipo = self.media_tipo;
        Self::area_central(ui, |ui| {
            alternador(ui, ["Simples", "Ponderada"], &mut tipo);
            ui.add_space(4.0);
            if ponderada {
                let w = (ui.available_width() - 8.0) / 2.0;
                ui.horizontal(|ui| {
                    ui.allocate_ui(Vec2::new(w, 90.0), |ui| {
                        ui.vertical(|ui| {
                            if campo(ui, "Valor", &exibir_numero(&self.media_valor), "0", self.media_campo == 0) {
                                self.media_campo = 0;
                            }
                        });
                    });
                    ui.allocate_ui(Vec2::new(w, 90.0), |ui| {
                        ui.vertical(|ui| {
                            if campo(ui, "Peso", &exibir_numero(&self.media_peso), "1", self.media_campo == 1) {
                                self.media_campo = 1;
                            }
                        });
                    });
                });
            } else if campo(ui, "Valor", &exibir_numero(&self.media_valor), "Digite um valor", true) {
                self.media_campo = 0;
            }

            if !self.media_itens.is_empty() {
                ui.add_space(4.0);
                let resultado = if ponderada {
                    calc::media_ponderada(&self.media_itens)
                } else {
                    let v: Vec<f64> = self.media_itens.iter().map(|(v, _)| *v).collect();
                    calc::media_simples(&v).ok_or("Adicione pelo menos um valor")
                };
                cartao(ui, |ui| {
                    ui.label(RichText::new("Média").color(TEXTO_FRACO).size(17.0));
                    match resultado {
                        Ok(m) => {
                            ui.label(RichText::new(calc::format_num(m, 6)).size(40.0).strong().color(DESTAQUE));
                        }
                        Err(e) => {
                            ui.label(RichText::new(e).size(20.0).color(VERMELHO));
                        }
                    }
                    let soma: f64 = self.media_itens.iter().map(|(v, _)| v).sum();
                    let mut info = format!(
                        "{} · soma {}",
                        plural(self.media_itens.len() as i64, "valor", "valores"),
                        calc::format_num(soma, 6)
                    );
                    if ponderada {
                        let pesos: f64 = self.media_itens.iter().map(|(_, p)| p).sum();
                        info.push_str(&format!(" · pesos {}", calc::format_num(pesos, 6)));
                    }
                    ui.label(RichText::new(info).size(16.0).color(TEXTO_FRACO));
                });

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Valores adicionados").color(TEXTO_FRACO).size(17.0));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(egui::Button::new(RichText::new("Limpar tudo").color(VERMELHO).size(16.0)).frame(false))
                            .clicked()
                        {
                            self.media_itens.clear();
                        }
                    });
                });
                let mut remover = None;
                for (i, (v, p)) in self.media_itens.iter().enumerate() {
                    egui::Frame::new()
                        .fill(PAINEL)
                        .corner_radius(CornerRadius::same(10))
                        .inner_margin(Margin::symmetric(14, 6))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                let mut t = format!("{}.  {}", i + 1, calc::format_num(*v, 6));
                                if ponderada {
                                    t.push_str(&format!("   (peso {})", calc::format_num(*p, 6)));
                                }
                                ui.label(RichText::new(t).size(20.0).color(TEXTO));
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    let b = egui::Button::new(RichText::new("×").size(26.0).color(TEXTO_FRACO))
                                        .frame(false)
                                        .min_size(Vec2::new(40.0, 36.0));
                                    if ui.add(b).clicked() {
                                        remover = Some(i);
                                    }
                                });
                            });
                        });
                }
                if let Some(i) = remover {
                    self.media_itens.remove(i);
                }
            } else {
                ui.add_space(8.0);
                ui.label(
                    RichText::new(if ponderada {
                        "Digite o valor e o peso e toque em Adicionar."
                    } else {
                        "Digite cada valor e toque em Adicionar."
                    })
                    .size(17.0)
                    .color(TEXTO_FRACO),
                );
            }
        });
        if tipo != self.media_tipo {
            self.media_tipo = tipo;
            self.media_campo = 0;
        }
    }

    fn acao_media(&mut self, a: Acao) {
        let ponderada = self.media_tipo == 1;
        match a {
            Acao::Enter => {
                let Some(v) = calc::parse_num(&self.media_valor) else {
                    self.media_campo = 0;
                    return;
                };
                let p = if ponderada {
                    match calc::parse_num(&self.media_peso) {
                        Some(p) => p,
                        None if self.media_campo == 0 && self.media_peso.is_empty() => {
                            self.media_campo = 1;
                            return;
                        }
                        None => 1.0,
                    }
                } else {
                    1.0
                };
                self.media_itens.push((v, p));
                self.media_valor.clear();
                self.media_peso.clear();
                self.media_campo = 0;
            }
            Acao::Proximo => self.media_campo = if ponderada { 1 - self.media_campo } else { 0 },
            _ => {
                let alvo = if ponderada && self.media_campo == 1 {
                    &mut self.media_peso
                } else {
                    &mut self.media_valor
                };
                editar_numero(alvo, a, true);
            }
        }
    }

    // ----------------------------------------------------------------- Datas

    fn tela_datas(&mut self, ui: &mut egui::Ui, fisicas: &[Acao]) {
        self.cabecalho(ui, "Dias entre datas");
        let teclas: [&[Tecla]; 4] = [
            &[num("7", '7'), num("8", '8'), num("9", '9'), op(APAGAR, Acao::Apagar)],
            &[num("4", '4'), num("5", '5'), num("6", '6'), op("C", Acao::Limpar)],
            &[num("1", '1'), num("2", '2'), num("3", '3'), op("Hoje", Acao::Hoje)],
            &[num("0", '0'), larga("Próxima data", Acao::Proximo, 3.0, Estilo::Destaque)],
        ];
        let toque = Self::painel_teclado(ui, &teclas);
        for a in toque.into_iter().chain(fisicas.iter().copied()) {
            self.acao_datas(a);
        }

        let hoje = hoje();
        Self::area_central(ui, |ui| {
            let ini = calc::parse_data(&self.data_ini);
            let fim = calc::parse_data(&self.data_fim);

            for (i, rotulo) in ["Data inicial", "Data final"].iter().enumerate() {
                let (digitos, data) = if i == 0 { (&self.data_ini, ini) } else { (&self.data_fim, fim) };
                if campo(ui, rotulo, &calc::mascara_data(digitos), "", self.data_campo == i) {
                    self.data_campo = i;
                }
                let dica = match data {
                    Some(d) if d == hoje => format!("Hoje, {}", calc::dia_da_semana(d)),
                    Some(d) => {
                        let mut s = calc::dia_da_semana(d).to_string();
                        if let Some(c) = s.get_mut(0..1) {
                            c.make_ascii_uppercase();
                        }
                        s
                    }
                    None if digitos.len() == 8 => "Data inválida".into(),
                    None => "Digite dia, mês e ano (dd/mm/aaaa)".into(),
                };
                let cor = if data.is_none() && digitos.len() == 8 { VERMELHO } else { TEXTO_FRACO };
                ui.label(RichText::new(dica).size(15.0).color(cor));
                ui.add_space(2.0);
            }

            ui.checkbox(
                &mut self.incluir_ultimo,
                RichText::new("Incluir o último dia na contagem").size(18.0).color(TEXTO),
            );
            ui.add_space(4.0);

            if let (Some(a), Some(b)) = (ini, fim) {
                let r = calc::diferenca_datas(a, b, self.incluir_ultimo);
                cartao(ui, |ui| {
                    ui.label(RichText::new("Diferença").color(TEXTO_FRACO).size(17.0));
                    ui.label(RichText::new(plural(r.dias.abs(), "dia", "dias")).size(40.0).strong().color(DESTAQUE));
                    let mut partes = Vec::new();
                    if r.anos > 0 {
                        partes.push(plural(r.anos as i64, "ano", "anos"));
                    }
                    if r.meses > 0 {
                        partes.push(plural(r.meses as i64, "mês", "meses"));
                    }
                    if r.dias_resto > 0 || partes.is_empty() {
                        partes.push(plural(r.dias_resto as i64, "dia", "dias"));
                    }
                    ui.label(RichText::new(juntar(&partes)).size(19.0).color(TEXTO));
                    if r.semanas > 0 {
                        let mut s = plural(r.semanas, "semana", "semanas");
                        if r.dias_semana_resto > 0 {
                            s = format!("{s} e {}", plural(r.dias_semana_resto, "dia", "dias"));
                        }
                        ui.label(RichText::new(s).size(17.0).color(TEXTO_FRACO));
                    }
                    if r.dias < 0 {
                        ui.label(RichText::new("A data final é anterior à inicial.").size(15.0).color(VERMELHO));
                    }
                });
            }
        });
    }

    fn acao_datas(&mut self, a: Acao) {
        let alvo = if self.data_campo == 0 { &mut self.data_ini } else { &mut self.data_fim };
        match a {
            Acao::Ch(c) if c.is_ascii_digit() => {
                if alvo.len() < 8 {
                    alvo.push(c);
                }
                // Ao completar a data inicial, pula para a final (se ainda não estiver preenchida).
                if self.data_campo == 0 && self.data_ini.len() == 8 && calc::parse_data(&self.data_fim).is_none() {
                    self.data_campo = 1;
                }
            }
            Acao::Apagar => {
                alvo.pop();
            }
            Acao::Limpar => alvo.clear(),
            Acao::Hoje => *alvo = calc::digitos_da_data(hoje()),
            Acao::Proximo | Acao::Enter => self.data_campo = 1 - self.data_campo,
            _ => {}
        }
    }

    // ------------------------------------------------------------- Variação

    fn tela_variacao(&mut self, ui: &mut egui::Ui, fisicas: &[Acao]) {
        self.cabecalho(ui, "Desconto ou aumento");
        let teclas: [&[Tecla]; 4] = [
            &[num("7", '7'), num("8", '8'), num("9", '9'), op(APAGAR, Acao::Apagar)],
            &[num("4", '4'), num("5", '5'), num("6", '6'), op("C", Acao::Limpar)],
            &[num("1", '1'), num("2", '2'), num("3", '3'), op("±", Acao::Sinal)],
            &[num(",", ','), num("0", '0'), larga("Próximo", Acao::Proximo, 2.0, Estilo::Destaque)],
        ];
        let toque = Self::painel_teclado(ui, &teclas);
        for a in toque.into_iter().chain(fisicas.iter().copied()) {
            match a {
                Acao::Proximo | Acao::Enter => self.var_campo = 1 - self.var_campo,
                _ => {
                    let alvo = if self.var_campo == 0 { &mut self.var_orig } else { &mut self.var_novo };
                    editar_numero(alvo, a, true);
                }
            }
        }

        Self::area_central(ui, |ui| {
            if campo(ui, "Valor original", &exibir_numero(&self.var_orig), "0", self.var_campo == 0) {
                self.var_campo = 0;
            }
            ui.add_space(2.0);
            if campo(ui, "Novo valor", &exibir_numero(&self.var_novo), "0", self.var_campo == 1) {
                self.var_campo = 1;
            }
            ui.add_space(6.0);

            let (Some(o), Some(n)) = (calc::parse_num(&self.var_orig), calc::parse_num(&self.var_novo)) else {
                ui.label(
                    RichText::new("Informe o valor original e o novo valor.")
                        .size(17.0)
                        .color(TEXTO_FRACO),
                );
                return;
            };
            cartao(ui, |ui| match calc::variacao(o, n) {
                Err(e) => {
                    ui.label(RichText::new(e).size(20.0).color(VERMELHO));
                }
                Ok(v) => {
                    let (titulo, pct, dif, cor, sinal) = match v {
                        Variacao::Desconto { pct, diferenca } => ("Desconto", pct, diferenca, VERDE, "−"),
                        Variacao::Aumento { pct, diferenca } => ("Aumento", pct, diferenca, VERMELHO, "+"),
                        Variacao::Igual => ("Sem alteração", 0.0, 0.0, TEXTO, ""),
                    };
                    ui.label(RichText::new(titulo).color(TEXTO_FRACO).size(17.0));
                    ui.label(
                        RichText::new(format!("{sinal}{}%", calc::format_num(pct, 2)))
                            .size(44.0)
                            .strong()
                            .color(cor),
                    );
                    ui.label(
                        RichText::new(format!("Diferença: {sinal}{}", calc::format_num(dif, 2)))
                            .size(19.0)
                            .color(TEXTO),
                    );
                    ui.label(
                        RichText::new(format!(
                            "O novo valor equivale a {}% do original",
                            calc::format_num(n / o * 100.0, 2)
                        ))
                        .size(16.0)
                        .color(TEXTO_FRACO),
                    );
                }
            });
        });
    }

    // -------------------------------------------------------------- Padrão

    fn tela_padrao(&mut self, ui: &mut egui::Ui, fisicas: &[Acao]) {
        self.cabecalho(ui, "Calculadora padrão");
        let linhas: Vec<&[Tecla]> = TECLADO_PADRAO.iter().map(|l| l.as_slice()).collect();
        let toque = Self::painel_teclado(ui, &linhas);
        for a in toque.into_iter().chain(fisicas.iter().copied()) {
            self.acao_padrao(a);
        }

        egui::CentralPanel::no_frame()
            .frame(egui::Frame::new().fill(FUNDO).inner_margin(Margin::symmetric(16, 8)))
            .show(ui, |ui| {
                ui.with_layout(Layout::bottom_up(Align::Max), |ui| {
                    let preview = if self.calculado || !self.expr.chars().skip(1).any(|c| "+-×÷%".contains(c)) {
                        None
                    } else {
                        calc::avaliar(&self.expr).ok()
                    };
                    if let Some(e) = self.erro {
                        ui.label(RichText::new(e).size(20.0).color(VERMELHO));
                    } else if let Some(v) = preview {
                        ui.label(RichText::new(format!("= {}", calc::format_num(v, 10))).size(26.0).color(TEXTO_FRACO));
                    } else {
                        ui.label(RichText::new(" ").size(26.0));
                    }
                    let texto = if self.expr.is_empty() { "0".to_string() } else { exibir_expressao(&self.expr) };
                    let tamanho = match texto.chars().count() {
                        0..=10 => 52.0,
                        11..=16 => 40.0,
                        17..=24 => 30.0,
                        _ => 24.0,
                    };
                    ui.add(egui::Label::new(RichText::new(texto).size(tamanho).color(TEXTO)).wrap());
                    if !self.anterior.is_empty() {
                        ui.label(RichText::new(&self.anterior).size(20.0).color(TEXTO_FRACO));
                    }
                });
            });
    }

    fn acao_padrao(&mut self, a: Acao) {
        self.erro = None;
        let ultimo = self.expr.chars().last();
        let eh_op = |c: char| "+-×÷".contains(c);
        match a {
            Acao::Limpar => {
                self.expr.clear();
                self.anterior.clear();
                self.calculado = false;
            }
            Acao::Apagar => {
                if self.calculado {
                    self.expr.clear();
                    self.calculado = false;
                } else {
                    self.expr.pop();
                }
            }
            Acao::Enter => {
                if self.expr.is_empty() {
                    return;
                }
                match calc::avaliar(&self.expr) {
                    Ok(v) => {
                        self.anterior = format!("{} =", exibir_expressao(&self.expr));
                        let r = calc::format_num(v, 10);
                        // Resultados em notação científica não podem ser reeditados.
                        self.expr = if r.contains('e') { String::new() } else { r.replace('.', "") };
                        if r.contains('e') {
                            self.anterior = format!("{} {}", self.anterior, r);
                        }
                        self.calculado = true;
                    }
                    Err(e) => self.erro = Some(e.mensagem()),
                }
            }
            Acao::Ch(c) if c.is_ascii_digit() => {
                if self.calculado {
                    self.expr.clear();
                    self.calculado = false;
                }
                if matches!(ultimo, Some(')') | Some('%')) {
                    self.expr.push('×');
                }
                // Evita zeros à esquerda: "05" -> "5"
                let segmento: String = self.expr.chars().rev().take_while(|c| c.is_ascii_digit() || *c == ',').collect();
                if segmento == "0" {
                    self.expr.pop();
                }
                if self.expr.len() < 80 {
                    self.expr.push(c);
                }
            }
            Acao::Ch(',') => {
                if self.calculado {
                    self.calculado = false;
                }
                let segmento: String = self.expr.chars().rev().take_while(|c| c.is_ascii_digit() || *c == ',').collect();
                if segmento.contains(',') {
                    return;
                }
                if matches!(ultimo, Some(')') | Some('%')) {
                    self.expr.push('×');
                }
                if segmento.is_empty() {
                    self.expr.push('0');
                }
                self.expr.push(',');
            }
            Acao::Ch(c) if eh_op(c) => {
                self.calculado = false;
                match ultimo {
                    None => {
                        if c == '-' {
                            self.expr.push('-');
                        }
                    }
                    Some('(') => {
                        if c == '-' {
                            self.expr.push('-');
                        }
                    }
                    Some(u) if eh_op(u) => {
                        // Permite "2×-3"; nos demais casos troca o operador.
                        if c == '-' && (u == '×' || u == '÷') {
                            self.expr.push('-');
                        } else {
                            self.expr.pop();
                            if let Some(u2) = self.expr.chars().last() {
                                if eh_op(u2) {
                                    self.expr.pop();
                                }
                            }
                            if !self.expr.is_empty() && !self.expr.ends_with('(') {
                                self.expr.push(c);
                            } else if c == '-' {
                                self.expr.push('-');
                            }
                        }
                    }
                    Some(',') => {
                        self.expr.pop();
                        self.expr.push(c);
                    }
                    Some(_) => self.expr.push(c),
                }
            }
            Acao::Ch('%') => {
                self.calculado = false;
                if matches!(ultimo, Some(u) if u.is_ascii_digit() || u == ')') {
                    self.expr.push('%');
                }
            }
            Acao::Parenteses | Acao::Ch('(') | Acao::Ch(')') => {
                if self.calculado && a != Acao::Ch(')') {
                    self.calculado = false;
                }
                let abertos = self.expr.matches('(').count() as i32 - self.expr.matches(')').count() as i32;
                let depois_de_valor = matches!(ultimo, Some(u) if u.is_ascii_digit() || u == ')' || u == '%');
                let fechar = match a {
                    Acao::Ch('(') => false,
                    Acao::Ch(')') => true,
                    _ => abertos > 0 && depois_de_valor,
                };
                if fechar {
                    if abertos > 0 && depois_de_valor {
                        self.expr.push(')');
                    }
                } else {
                    if self.calculado {
                        self.calculado = false;
                    }
                    if depois_de_valor {
                        self.expr.push('×');
                    } else if ultimo == Some(',') {
                        self.expr.pop();
                        self.expr.push('×');
                    }
                    self.expr.push('(');
                }
            }
            _ => {}
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Botão "voltar" do Android (chega como BrowserBack) e Esc no PC.
        let voltar = ui.ctx().input(|i| i.key_pressed(Key::BrowserBack) || i.key_pressed(Key::Escape));
        if voltar {
            if self.tela == Tela::Menu {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            } else {
                self.tela = Tela::Menu;
            }
        }

        let fisicas = if self.tela == Tela::Menu { Vec::new() } else { teclado_fisico(ui.ctx()) };
        match self.tela {
            Tela::Menu => self.tela_menu(ui),
            Tela::Media => self.tela_media(ui, &fisicas),
            Tela::Datas => self.tela_datas(ui, &fisicas),
            Tela::Variacao => self.tela_variacao(ui, &fisicas),
            Tela::Padrao => self.tela_padrao(ui, &fisicas),
        }
    }
}

fn configurar_estilo(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = FUNDO;
    visuals.window_fill = FUNDO;
    visuals.extreme_bg_color = CAMPO;
    visuals.override_text_color = Some(TEXTO);
    visuals.selection.bg_fill = DESTAQUE;
    visuals.selection.stroke = Stroke::new(1.0, Color32::BLACK);
    visuals.widgets.inactive.weak_bg_fill = CAMPO;
    visuals.widgets.hovered.weak_bg_fill = CAMPO_ATIVO;
    visuals.widgets.active.weak_bg_fill = BORDA;
    ctx.set_visuals_of(egui::Theme::Dark, visuals.clone());
    ctx.set_visuals_of(egui::Theme::Light, visuals);
    ctx.set_theme(egui::Theme::Dark);
    ctx.all_styles_mut(|s| {
        s.text_styles.insert(TextStyle::Body, FontId::proportional(20.0));
        s.text_styles.insert(TextStyle::Button, FontId::proportional(20.0));
        s.text_styles.insert(TextStyle::Heading, FontId::proportional(28.0));
        s.text_styles.insert(TextStyle::Small, FontId::proportional(15.0));
        s.spacing.interact_size = Vec2::new(40.0, 40.0);
        s.spacing.icon_width = 26.0;
        s.spacing.icon_width_inner = 14.0;
        s.interaction.selectable_labels = false;
    });
}
