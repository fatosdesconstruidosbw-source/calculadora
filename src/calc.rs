//! Lógica de todos os cálculos, sem nenhuma dependência de interface.
//! Assim ela pode ser testada no PC com `cargo test`.

use chrono::{Datelike, Months, NaiveDate};

// ---------------------------------------------------------------------------
// Números no formato brasileiro
// ---------------------------------------------------------------------------

/// Converte texto digitado ("1.234,56", "12,5", "-3") em número.
/// O ponto é tratado como separador de milhar e a vírgula como decimal.
pub fn parse_num(s: &str) -> Option<f64> {
    let t: String = s.trim().chars().filter(|c| *c != '.').collect();
    if t.is_empty() || t == "-" || t == "," || t == "-," {
        return None;
    }
    t.replace(',', ".").parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Formata um número no padrão brasileiro: 1.234.567,89
/// `max_dec` é o número máximo de casas decimais (zeros à direita são removidos).
pub fn format_num(v: f64, max_dec: usize) -> String {
    if !v.is_finite() {
        return "Erro".into();
    }
    let a = v.abs();
    if a != 0.0 && (a >= 1e15 || a < 1e-9) {
        let s = format!("{:.8e}", v);
        let (m, e) = s.split_once('e').unwrap_or((&s, "0"));
        let m = m.trim_end_matches('0').trim_end_matches('.');
        return format!("{}e{}", m.replace('.', ","), e);
    }
    let s = format!("{:.*}", max_dec, a);
    let (int, frac) = match s.split_once('.') {
        Some((i, f)) => (i.to_string(), f.trim_end_matches('0').to_string()),
        None => (s.clone(), String::new()),
    };
    let digits: Vec<char> = int.chars().collect();
    let mut grouped = String::new();
    for (i, c) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            grouped.push('.');
        }
        grouped.push(*c);
    }
    let is_zero = grouped == "0" && frac.is_empty();
    let sign = if v < 0.0 && !is_zero { "-" } else { "" };
    if frac.is_empty() {
        format!("{sign}{grouped}")
    } else {
        format!("{sign}{grouped},{frac}")
    }
}

// ---------------------------------------------------------------------------
// Média
// ---------------------------------------------------------------------------

/// Média aritmética simples.
pub fn media_simples(valores: &[f64]) -> Option<f64> {
    if valores.is_empty() {
        None
    } else {
        Some(valores.iter().sum::<f64>() / valores.len() as f64)
    }
}

/// Média ponderada: soma(valor × peso) / soma(pesos).
pub fn media_ponderada(itens: &[(f64, f64)]) -> Result<f64, &'static str> {
    if itens.is_empty() {
        return Err("Adicione pelo menos um valor");
    }
    let soma_pesos: f64 = itens.iter().map(|(_, p)| p).sum();
    if soma_pesos == 0.0 {
        return Err("A soma dos pesos não pode ser zero");
    }
    Ok(itens.iter().map(|(v, p)| v * p).sum::<f64>() / soma_pesos)
}

// ---------------------------------------------------------------------------
// Datas
// ---------------------------------------------------------------------------

/// Mostra os dígitos digitados (até 8) como "dd/mm/aaaa", com "_" no que falta.
pub fn mascara_data(digitos: &str) -> String {
    let mut out = String::new();
    let mut it = digitos.chars();
    for (i, _) in "ddmmaaaa".chars().enumerate() {
        if i == 2 || i == 4 {
            out.push('/');
        }
        out.push(it.next().unwrap_or('_'));
    }
    out
}

/// Converte 8 dígitos "DDMMAAAA" em data. Retorna None se incompleta ou inválida.
pub fn parse_data(digitos: &str) -> Option<NaiveDate> {
    if digitos.len() != 8 || !digitos.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let d: u32 = digitos[0..2].parse().ok()?;
    let m: u32 = digitos[2..4].parse().ok()?;
    let a: i32 = digitos[4..8].parse().ok()?;
    NaiveDate::from_ymd_opt(a, m, d)
}

/// Data no formato "DDMMAAAA" (o mesmo usado nos campos).
pub fn digitos_da_data(d: NaiveDate) -> String {
    format!("{:02}{:02}{:04}", d.day(), d.month(), d.year())
}

pub fn dia_da_semana(d: NaiveDate) -> &'static str {
    use chrono::Weekday::*;
    match d.weekday() {
        Mon => "segunda-feira",
        Tue => "terça-feira",
        Wed => "quarta-feira",
        Thu => "quinta-feira",
        Fri => "sexta-feira",
        Sat => "sábado",
        Sun => "domingo",
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiferencaDatas {
    /// Dias de `inicio` até `fim` (negativo se `fim` for anterior).
    pub dias: i64,
    pub anos: u32,
    pub meses: u32,
    pub dias_resto: u32,
    pub semanas: i64,
    pub dias_semana_resto: i64,
}

/// Diferença entre duas datas. Se `incluir_ultimo_dia`, soma 1 dia à contagem
/// (útil para contar "do dia X ao dia Y, inclusive").
pub fn diferenca_datas(inicio: NaiveDate, fim: NaiveDate, incluir_ultimo_dia: bool) -> DiferencaDatas {
    let mut dias = (fim - inicio).num_days();
    let (a, b) = if inicio <= fim { (inicio, fim) } else { (fim, inicio) };

    // Maior número de meses inteiros que cabe entre a e b.
    let mut meses_total = (b.year() - a.year()) * 12 + b.month() as i32 - a.month() as i32;
    while meses_total > 0
        && a
            .checked_add_months(Months::new(meses_total as u32))
            .is_none_or(|x| x > b)
    {
        meses_total -= 1;
    }
    let meses_total = meses_total.max(0) as u32;
    let base = a.checked_add_months(Months::new(meses_total)).unwrap_or(a);
    let mut dias_resto = (b - base).num_days() as u32;

    if incluir_ultimo_dia {
        dias += if dias < 0 { -1 } else { 1 };
        dias_resto += 1;
    }

    let abs = dias.abs();
    DiferencaDatas {
        dias,
        anos: meses_total / 12,
        meses: meses_total % 12,
        dias_resto,
        semanas: abs / 7,
        dias_semana_resto: abs % 7,
    }
}

// ---------------------------------------------------------------------------
// Desconto / aumento
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Variacao {
    Aumento { pct: f64, diferenca: f64 },
    Desconto { pct: f64, diferenca: f64 },
    Igual,
}

/// Compara o valor original com o novo e diz se houve aumento ou desconto, em %.
pub fn variacao(original: f64, novo: f64) -> Result<Variacao, &'static str> {
    if original == 0.0 {
        return Err("O valor original não pode ser zero");
    }
    let diferenca = novo - original;
    let pct = diferenca / original.abs() * 100.0;
    Ok(if diferenca > 0.0 {
        Variacao::Aumento { pct, diferenca }
    } else if diferenca < 0.0 {
        Variacao::Desconto { pct: -pct, diferenca: -diferenca }
    } else {
        Variacao::Igual
    })
}

// ---------------------------------------------------------------------------
// Calculadora padrão: avaliador de expressões
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum ErroCalc {
    Sintaxe,
    DivisaoPorZero,
}

impl ErroCalc {
    pub fn mensagem(&self) -> &'static str {
        match self {
            ErroCalc::Sintaxe => "Expressão incompleta",
            ErroCalc::DivisaoPorZero => "Não é possível dividir por zero",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    Num(f64),
    Mais,
    Menos,
    Vezes,
    Div,
    Pct,
    Abre,
    Fecha,
}

fn tokenizar(s: &str) -> Result<Vec<Tok>, ErroCalc> {
    let chars: Vec<char> = s.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            ' ' => {}
            '0'..='9' | ',' => {
                let mut num = String::new();
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == ',' || chars[i] == '.') {
                    match chars[i] {
                        '.' => {} // separador de milhar
                        ',' => num.push('.'),
                        d => num.push(d),
                    }
                    i += 1;
                }
                let v: f64 = num.parse().map_err(|_| ErroCalc::Sintaxe)?;
                out.push(Tok::Num(v));
                continue;
            }
            '+' => out.push(Tok::Mais),
            '-' | '−' => out.push(Tok::Menos),
            '×' | '*' | 'x' => out.push(Tok::Vezes),
            '÷' | '/' => out.push(Tok::Div),
            '%' => out.push(Tok::Pct),
            '(' => out.push(Tok::Abre),
            ')' => out.push(Tok::Fecha),
            _ => return Err(ErroCalc::Sintaxe),
        }
        i += 1;
    }
    Ok(out)
}

struct Parser {
    toks: Vec<Tok>,
    pos: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
    }

    fn next(&mut self) -> Option<Tok> {
        let t = self.toks.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    // expr := termo (('+'|'-') termo)*
    fn expr(&mut self) -> Result<f64, ErroCalc> {
        let (mut v, _) = self.termo()?;
        while let Some(op) = self.peek().cloned() {
            if op != Tok::Mais && op != Tok::Menos {
                break;
            }
            self.next();
            let (mut r, so_pct) = self.termo()?;
            // "200 + 10%" = 220 (porcentagem do valor à esquerda), como nas calculadoras de celular.
            if so_pct {
                r *= v;
            }
            v = if op == Tok::Mais { v + r } else { v - r };
        }
        Ok(v)
    }

    // termo := unario (('×'|'÷') unario)*
    // Retorna também se o termo é apenas "N%" (para a regra da porcentagem acima).
    fn termo(&mut self) -> Result<(f64, bool), ErroCalc> {
        let (mut v, mut so_pct) = self.unario()?;
        while let Some(op) = self.peek().cloned() {
            if op != Tok::Vezes && op != Tok::Div {
                break;
            }
            self.next();
            let (r, _) = self.unario()?;
            so_pct = false;
            if op == Tok::Vezes {
                v *= r;
            } else {
                if r == 0.0 {
                    return Err(ErroCalc::DivisaoPorZero);
                }
                v /= r;
            }
        }
        Ok((v, so_pct))
    }

    // unario := '-' unario | '+' unario | posfixo
    fn unario(&mut self) -> Result<(f64, bool), ErroCalc> {
        match self.peek() {
            Some(Tok::Menos) => {
                self.next();
                let (v, p) = self.unario()?;
                Ok((-v, p))
            }
            Some(Tok::Mais) => {
                self.next();
                self.unario()
            }
            _ => self.posfixo(),
        }
    }

    // posfixo := primario '%'*
    fn posfixo(&mut self) -> Result<(f64, bool), ErroCalc> {
        let mut v = self.primario()?;
        let mut pct = false;
        while self.peek() == Some(&Tok::Pct) {
            self.next();
            v /= 100.0;
            pct = true;
        }
        Ok((v, pct))
    }

    // primario := número | '(' expr ')'
    fn primario(&mut self) -> Result<f64, ErroCalc> {
        match self.next() {
            Some(Tok::Num(v)) => Ok(v),
            Some(Tok::Abre) => {
                let v = self.expr()?;
                // Parênteses não fechados no fim são aceitos: "2×(3+4" = 14
                match self.peek() {
                    Some(Tok::Fecha) => {
                        self.next();
                    }
                    None => {}
                    _ => return Err(ErroCalc::Sintaxe),
                }
                Ok(v)
            }
            _ => Err(ErroCalc::Sintaxe),
        }
    }
}

/// Avalia uma expressão como "12,5 + 3 × (2 − 1)" ou "200 + 10%".
pub fn avaliar(expressao: &str) -> Result<f64, ErroCalc> {
    let toks = tokenizar(expressao)?;
    if toks.is_empty() {
        return Err(ErroCalc::Sintaxe);
    }
    let mut p = Parser { toks, pos: 0 };
    let v = p.expr()?;
    if p.pos != p.toks.len() {
        return Err(ErroCalc::Sintaxe);
    }
    if !v.is_finite() {
        return Err(ErroCalc::Sintaxe);
    }
    Ok(v)
}

// ---------------------------------------------------------------------------
// Testes
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn d(dia: u32, mes: u32, ano: i32) -> NaiveDate {
        NaiveDate::from_ymd_opt(ano, mes, dia).unwrap()
    }

    #[test]
    fn numeros_br() {
        assert_eq!(parse_num("1.234,56"), Some(1234.56));
        assert_eq!(parse_num("-3"), Some(-3.0));
        assert_eq!(parse_num(",5"), Some(0.5));
        assert_eq!(parse_num(""), None);
        assert_eq!(parse_num("-"), None);
        assert_eq!(format_num(1234567.891, 2), "1.234.567,89");
        assert_eq!(format_num(0.1 + 0.2, 10), "0,3");
        assert_eq!(format_num(-5.0, 2), "-5");
        assert_eq!(format_num(-0.0001, 2), "0");
        assert_eq!(format_num(100.0, 2), "100");
        assert_eq!(format_num(2e20, 2), "2e20");
    }

    #[test]
    fn medias() {
        assert_eq!(media_simples(&[7.0, 8.0, 9.0]), Some(8.0));
        assert_eq!(media_simples(&[]), None);
        assert_eq!(media_ponderada(&[(10.0, 2.0), (4.0, 1.0)]), Ok(8.0));
        assert!(media_ponderada(&[(10.0, 0.0)]).is_err());
    }

    #[test]
    fn datas() {
        assert_eq!(mascara_data("0112"), "01/12/____");
        assert_eq!(parse_data("29022024"), Some(d(29, 2, 2024)));
        assert_eq!(parse_data("29022023"), None);
        assert_eq!(parse_data("3112"), None);
        assert_eq!(digitos_da_data(d(5, 3, 2026)), "05032026");

        let r = diferenca_datas(d(1, 1, 2026), d(26, 11, 2026), false);
        assert_eq!(r.dias, 329);
        assert_eq!((r.anos, r.meses, r.dias_resto), (0, 10, 25));
        assert_eq!((r.semanas, r.dias_semana_resto), (47, 0));

        let r = diferenca_datas(d(31, 1, 2025), d(1, 3, 2025), false);
        assert_eq!(r.dias, 29);
        assert_eq!((r.anos, r.meses, r.dias_resto), (0, 1, 1));

        let r = diferenca_datas(d(26, 11, 2026), d(26, 11, 2016), false);
        assert_eq!(r.dias, -3652);
        assert_eq!((r.anos, r.meses, r.dias_resto), (10, 0, 0));

        let r = diferenca_datas(d(1, 1, 2026), d(1, 1, 2026), true);
        assert_eq!(r.dias, 1);
    }

    #[test]
    fn variacoes() {
        assert_eq!(variacao(100.0, 80.0), Ok(Variacao::Desconto { pct: 20.0, diferenca: 20.0 }));
        assert_eq!(variacao(50.0, 75.0), Ok(Variacao::Aumento { pct: 50.0, diferenca: 25.0 }));
        assert_eq!(variacao(10.0, 10.0), Ok(Variacao::Igual));
        assert!(variacao(0.0, 10.0).is_err());
    }

    #[test]
    fn expressoes() {
        assert_eq!(avaliar("2+3×4"), Ok(14.0));
        assert_eq!(avaliar("(2+3)×4"), Ok(20.0));
        assert_eq!(avaliar("10÷4"), Ok(2.5));
        assert_eq!(avaliar("1,5+1,5"), Ok(3.0));
        assert_eq!(avaliar("-3+5"), Ok(2.0));
        assert_eq!(avaliar("2×-3"), Ok(-6.0));
        assert_eq!(avaliar("200+10%"), Ok(220.0));
        assert_eq!(avaliar("200-10%"), Ok(180.0));
        assert_eq!(avaliar("200×10%"), Ok(20.0));
        assert_eq!(avaliar("50%"), Ok(0.5));
        assert_eq!(avaliar("2×(3+4"), Ok(14.0));
        assert_eq!(avaliar("1.234,5+0,5"), Ok(1235.0));
        assert_eq!(avaliar("5÷0"), Err(ErroCalc::DivisaoPorZero));
        assert_eq!(avaliar("5+"), Err(ErroCalc::Sintaxe));
        assert_eq!(avaliar(""), Err(ErroCalc::Sintaxe));
        assert_eq!(avaliar("2)"), Err(ErroCalc::Sintaxe));
    }
}
