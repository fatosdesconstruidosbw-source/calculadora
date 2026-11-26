# Calculadora (Android, em Rust)

Calculadora para Android escrita 100% em Rust, com interface feita em [egui](https://github.com/emilk/egui).

Ao abrir o app, você escolhe uma das opções:

| Opção | O que faz |
|---|---|
| **Média** | Média simples ou ponderada (valor + peso). Mostra também a soma e a quantidade de valores. |
| **Dias entre datas** | Conta os dias entre duas datas. A data final já vem com o **dia de hoje**, mas pode ser alterada. Mostra também anos/meses/dias e semanas, e permite incluir o último dia na contagem. |
| **Desconto ou aumento** | Você informa o valor original e o novo valor; o app mostra se houve **desconto** ou **aumento**, em %, e a diferença. |
| **Calculadora padrão** | `+  −  ×  ÷  %` e parênteses. `200 + 10%` = 220, `200 × 10%` = 20, `50%` = 0,5. |

Os números usam o padrão brasileiro (vírgula decimal, ponto de milhar). O app tem teclado próprio na tela, e o botão "voltar" do Android retorna ao menu.

## Baixar o APK

A cada envio para a branch `main`, o GitHub Actions compila o app e publica o arquivo em
**Releases → "Calculadora – última versão" → `calculadora.apk`**.

Link direto: `https://github.com/fatosdesconstruidosbw-source/calculadora/releases/download/ultima-versao/calculadora.apk`

No celular, abra o link, baixe e instale (o Android vai pedir para permitir a instalação de "fontes desconhecidas" para o navegador/gerenciador de arquivos).

O APK funciona em celulares com Android 7.0 ou superior (ARM 64 e 32 bits).

### Atualizar sem precisar desinstalar (chave fixa)

Sem configuração, cada build é assinado com uma chave nova, então para instalar uma versão nova é preciso desinstalar a anterior. Para usar sempre a mesma chave:

1. Gere uma chave (uma vez só, no seu PC, com Java instalado):
   ```
   keytool -genkeypair -keystore calculadora.keystore -alias calculadora -keyalg RSA -keysize 2048 -validity 10000
   ```
2. Converta para base64: `base64 -w0 calculadora.keystore` (Linux) ou `certutil -encode calculadora.keystore saida.txt` (Windows, sem as linhas BEGIN/END).
3. No GitHub: **Settings → Secrets and variables → Actions → New repository secret**:
   - `ANDROID_KEYSTORE_BASE64` = o texto base64
   - `ANDROID_KEYSTORE_PASSWORD` = a senha escolhida

Guarde o arquivo `.keystore` e a senha: sem eles não dá para publicar atualizações com a mesma assinatura.

## Estrutura

```
src/calc.rs          lógica de todos os cálculos + testes
src/ui.rs            telas e teclado (egui)
src/lib.rs           ponto de entrada do Android (android_main)
examples/desktop.rs  roda a mesma interface no PC
res/                 ícones do app
.github/workflows/   build automático do APK
```

## Desenvolvimento no PC

```
cargo test                      # testa os cálculos
cargo run --example desktop     # abre a calculadora numa janela
```

## Compilar o APK localmente (opcional)

Requer Android SDK (platform 34, build-tools) e NDK instalados, com `ANDROID_HOME` e `ANDROID_NDK_ROOT` definidos:

```
rustup target add aarch64-linux-android armv7-linux-androideabi
cargo install cargo-apk
cargo apk build --release       # usa CARGO_APK_RELEASE_KEYSTORE e CARGO_APK_RELEASE_KEYSTORE_PASSWORD
```

O APK fica em `target/release/apk/calculadora.apk`. Para testar direto num celular conectado via USB: `cargo apk run`.
