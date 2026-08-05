# Handoff — App Android NexTags AI (Tauri)

**Atualizado em 05/08/2026.** Substitui a versão anterior deste arquivo.

## Objetivo

Empacotar o mesmo projeto Tauri (já entregue como app desktop Windows) como APK Android,
carregando `https://app.nextagsai.com.br/` numa WebView nativa. Janela única, sem abas — proposital.

`main` continua com o build desktop/macOS intocado. Todas as mudanças de comportamento Android são
guardadas por `#[cfg(target_os = "android")]` / `#[cfg(desktop)]`, então não há risco para os builds
Windows/macOS/Linux do CI.

## NÃO trabalhe a partir de `Z:\Gustavo\nextags-desktop-app` diretamente

`Z:` é drive de rede mapeado (`\\nextagsdados\nextags`, DriveType 4). O `cargo-mobile2`
canonicaliza o path para UNC (`\\nextagsdados\...`) e a validação `starts_with(root_dir)` falha:

```
AssetDirOutsideOfAppRoot { asset_dir: "assets", root_dir: "...\src-tauri" }
```

Mesmo bug do issue tauri#8715. **`tauri android init` e `tauri android build` não rodam de share
SMB.** Copie o projeto pra um caminho local (ex.: `C:\build\nextags-desktop-app`, via
`robocopy /E /XD node_modules target .git`) e trabalhe de lá. Confirmado empiricamente: em `C:` o
init/build passa. Depois de gerar/atualizar `src-tauri/gen/android`, `.cargo/config.toml`, etc.
localmente, sincronize as mudanças relevantes de volta para o repo em `Z:` antes de commitar.

## Setup no PC local

### 1. Modo de Desenvolvedor do Windows — OBRIGATÓRIO

O `cargo-mobile2` cria um **symlink** do `.so` para `jniLibs`. Sem privilégio o build morre:

```
Failed to create a symbolic link ... libtauri_app_lib.so
Creation symbolic link is not allowed for this system.
```

Uma conta não-elevada e sem `SeCreateSymbolicLinkPrivilege` não consegue. Em PowerShell **como
administrador**:

```powershell
reg add "HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\AppModelUnlock" /t REG_DWORD /f /v "AllowDevelopmentWithoutDevLicense" /d "1"
```

Ou pela GUI: Configurações → Privacidade e segurança → Para desenvolvedores → Modo de
desenvolvedor. Efeito imediato, sem reboot.

Contorno sem admin/dev mode (só se travar de vez): compile o `.so`, copie manualmente para
`jniLibs/<abi>/` e rode o Gradle direto, pulando a task Rust:

```bash
cp src-tauri/target/aarch64-linux-android/debug/libtauri_app_lib.so \
   src-tauri/gen/android/app/src/main/jniLibs/arm64-v8a/
cd src-tauri/gen/android && ./gradlew assembleArm64Debug -x rustBuildArm64Debug
```

Isso **não** contorna `tauri android dev` (o Gradle chama `tauri android android-studio-script`,
que refaz o mesmo symlink e ainda espera um addr-file de dev server). Só serve pra `assembleDebug`
mesmo assim tende a falhar se a task `rustBuildArm64Debug` for reexecutada — o caminho confiável é
sempre ativar o Developer Mode e rodar `npm run tauri android build` normal.

### 2. Toolchain

Versões exatas usadas (replicar):

| Componente | Versão / caminho |
|---|---|
| JDK | Microsoft OpenJDK 17.0.20 (`winget install Microsoft.OpenJDK.17`) |
| Android SDK | cmdline-tools + `platform-tools` |
| SDK platforms | **`android-36`** (o Gradle gerado usa `compileSdk = 36` / `targetSdk = 36`) |
| build-tools | `36.0.0` |
| NDK | `27.0.12077973` |
| Rust targets | `aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android` |
| tauri-cli | 2.11.4+ (via `@tauri-apps/cli ^2`) |

```powershell
[Environment]::SetEnvironmentVariable('JAVA_HOME','<caminho do jdk 17>','User')
[Environment]::SetEnvironmentVariable('ANDROID_HOME','<caminho do sdk>','User')
[Environment]::SetEnvironmentVariable('NDK_HOME','<sdk>\ndk\27.0.12077973','User')
```

Use `[Environment]::SetEnvironmentVariable`, **não `setx`** — `setx` trunca em 1024 caracteres e
pode corromper o `Path`.

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
```

`platforms;android-34` **não basta**: o `build.gradle.kts` gerado pede `compileSdk = 36`.

### 3. Device

USB debugging ligado, `adb devices` deve listar o aparelho. Para apenas instalar um APK pronto,
só `platform-tools` é necessário; para compilar, o toolchain completo.

### Ordem que importa: `tauri icon` DEPOIS de `android init`

`tauri android init` sobrescreve os mipmaps com o placeholder do template. Rodar
`npx tauri icon app-icon-source.png` **depois** do init corrige (verificado por hash). Se
reinicializar o projeto Android, repita — e rode
`python3 scripts/fix_android_adaptive_icon.py` em seguida pra reaplicar a margem de segurança do
adaptive icon (ver "Correção do ícone" abaixo).

## Correção do ícone adaptativo Android

`tauri icon` só redimensiona `app-icon-source.png` 1:1 pro canvas de cada densidade do
`ic_launcher_foreground.png` — não deixa a margem ("safe zone") que o Android precisa pra mascarar
o ícone em círculo/squircle/rounded-square sem cortar o hexágono (~79% do canvas, acima do limite
de 66dp/108dp ~61% recomendado pelo Google). Isso fazia a borda azul "vazar"/cortar de forma
desigual em launchers mais agressivos (Samsung/MIUI/etc.).

Corrigido: logo redimensionado pra ~60% do canvas em `src-tauri/icons/android/mipmap-*/`.
`scripts/fix_android_adaptive_icon.py` reaplica essa margem sempre que os ícones forem
regenerados via `tauri icon` — rodar de novo depois de qualquer `npx tauri icon`.

## Medições sobre a UI mobile — não repita este trabalho

Chrome DevTools em 390×844 e `curl` com três UAs:

| Teste | Resultado |
|---|---|
| HTML servido: UA desktop vs mobile vs WebView | **md5 idêntico** — o servidor não faz UA sniffing |
| Inbox a 390px, UA mobile | 50 conversas, overflow horizontal **0** |
| Inbox a 390px, UA desktop do `lib.rs` | **idêntico** — mesmas 50 linhas, mesma altura 4972px |
| Viewport meta do site | `width=device-width, initial-scale=1.0, shrink-to-fit=no` (sem `viewport-fit=cover`) |

**Conclusões:**

1. A UA desktop hardcoded **não** degrada o layout mobile deste site. Mantida (serve ao OAuth do
   Google no Windows). O research automatizado afirmou o contrário em tese; a medição refuta no
   caso concreto.
2. O site **tem** layout responsivo funcional em largura de celular. Não há overflow.
3. Sem `viewport-fit=cover`, `env(safe-area-inset-*)` é 0 → fix de safe-area só por CSS injetado
   **não funciona**. Tem de ser nativo (ver `MainActivity.kt` abaixo).
4. "Nenhuma conversa aparece" era a pasta **Arquivadas**, genuinamente vazia. Não era bug.
5. `smallTargets: 3` (alvos de toque pequenos) — não é problema.
6. **`tinyFont: 146`** — 146 nós de texto com `font-size < 12px`. Principal queixa real de
   legibilidade, e é CSS do site. Ver "Aberto".
7. `button.ktt10-btn` (widget de suporte do site) é `position:fixed` 50×50 no canto inferior
   direito. Por isso **não usar FAB** — decisão já tomada.

### Bug do site, fora do escopo do wrapper

`vue-easy-lightbox.umd.min.js` carrega **antes** de `vue.min.js` e morre com
`Uncaught TypeError: Cannot read properties of undefined (reading 'extend')`. O lightbox de imagem
não registra. É ordem de `<script>` no site — reportar ao time do site.

## Fatos verificados no source dos crates (tauri 2.11.5 / wry 0.55.1 / tao 0.35.3)

Levantados por research com verificação adversarial, lendo os crates vendorizados em disco.

1. `WebviewWindowBuilder` dentro de `setup()` **funciona** no Android; `app.windows` deve
   continuar `[]`. Não mover a webview para a config.
2. No Android, `inner_size` / `min_inner_size` / `title` / `resizable` / `decorations` são
   **silenciosamente ignorados** (tao: `// FIXME this ignores requested window attributes`).
3. `.user_agent()` **aplica** no Android (vai para `WebSettings.setUserAgentString`).
4. **Init scripts rodam 2× em URL remota** no Android (`addDocumentStartJavaScript` **e** de novo
   em `RustWebViewClient.onPageStarted`, porque https não é interceptada por custom protocol).
   Scripts injetados **precisam ser idempotentes**. Já tratado no código.
5. `clear_all_browsing_data()` existe no Android mas limpa **só** cache/histórico/form data —
   **não** cookies, localStorage, IndexedDB nem service workers. No Windows a WebView2 limpa tudo.
   "Limpar cache" significa menos no Android; documentar para o usuário.
6. **`invoke('clear_cache')` estava quebrado — inclusive no build Windows já entregue.** Desde o
   tauri 2.11.1, IPC de origem não-local é rejeitado sem uma capability com `remote.urls`.
   `app.nextagsai.com.br` é não-local e nenhuma capability declarava isso. Passava despercebido
   porque o JS chamava `.finally(() => location.reload())` — a página recarregava e parecia
   funcionar, mas o cache nunca era limpo. **Corrigido** (`build.rs` + `capabilities/remote.json`).
   Confirmar no console/logcat quando testar.
7. `app.security.dangerousRemoteDomainIpcAccess` **não existe** no Tauri 2 (só no módulo de
   migração do v1). Não tente usar — `deny_unknown_fields` rejeita a config.
8. O padrão `"https://app.nextagsai.com.br"` (sem path) casa com qualquer path/query/hash. Mas
   `https://*.nextagsai.com.br` **não** casa com o domínio apex.
9. `android:usesCleartextTraffic` não precisa mudar (site é https). Se houver sub-recurso em http
   puro, será bloqueado no release — auditar mixed content no site, não habilitar cleartext.
10. wry **não configura nenhuma setting de viewport/zoom**. Herda defaults do Chromium, onde
    `builtInZoomControls = false` → **pinch-zoom desativado**. Corrigido via
    `src-tauri/.cargo/config.toml`.
11. Manifest gerado não tinha `windowSoftInputMode`; com `targetSdk 36` o manifest sozinho **não
    resolve** o teclado, e `window.visualViewport` não atualiza quando o IME abre (tauri#10631),
    logo o site não consegue se defender. Precisa de inset listener nativo. Corrigido.
12. Rotação **não** recria a Activity (`configChanges` inclui `orientation|screenSize`), então os
    init scripts não re-rodam ao girar.

## Mudanças de comportamento Android já feitas

| Arquivo | O quê |
|---|---|
| `src-tauri/src/lib.rs` | Menu compartilhado com guarda de idempotência. Android: **long-press** (550ms) abre menu com "Recarregar" + "Limpar cache e recarregar"; **pull-to-refresh** (90px) recarrega. Listeners todos `passive`. `inner_size`/`min_inner_size` agora sob `#[cfg(desktop)]`. Erro de `build()` logado em vez de panic opaco. |
| `src-tauri/build.rs` | `app_manifest` declarando o comando `clear_cache` (gera a permissão `allow-clear-cache`). |
| `src-tauri/capabilities/remote.json` | Capability com `remote.urls` liberando `clear_cache` para a origem remota. Corrige o item 6 acima **no desktop também**. |
| `src-tauri/.cargo/config.toml` | `WRY_RUSTWEBVIEW_CLASS_INIT` habilitando `builtInZoomControls` (pinch-zoom), `displayZoomControls=false`, `useWideViewPort`, `loadWithOverviewMode`. Só afeta `target_os = "android"`. |
| `gen/android/.../AndroidManifest.xml` | `android:windowSoftInputMode="adjustResize"`. |
| `gen/android/.../MainActivity.kt` | `OnApplyWindowInsetsListener` aplicando `systemBars` + `ime` como padding. Resolve edge-to-edge (barra fixa de 48px do site atrás da status bar) e teclado. |
| `.gitignore` | `src-tauri/gen` → `src-tauri/gen/schemas` + `src-tauri/gen/apple`. `gen/android` passa a ser versionado. |

O nome da env var do item do pinch-zoom foi confirmado no source: o `build.rs` do wry monta
`WRY_{STEM_EM_MAIÚSCULAS}_CLASS_INIT` por arquivo Kotlin, e o arquivo é `RustWebView.kt`, daí
`WRY_RUSTWEBVIEW_CLASS_INIT`. Ele também emite `cargo:rerun-if-env-changed`.

### Por que `gen/android` é versionado

Tauri **já gera** `.gitignore` dentro de `gen/android` e `gen/android/app` excluindo `build/`,
`jniLibs/**/*.so`, `tauri.properties`, `tauri.build.gradle.kts` e o Kotlin gerado — ou seja, o
diretório foi feito para ser commitado. As edições em `MainActivity.kt` e `AndroidManifest.xml`
seriam perdidas por um `tauri android init` e não sobrevivem se o diretório for ignorado.

**Se rodar `tauri android init` de novo, confira o diff desses dois arquivos antes de commitar.**

## Copiar e colar / seleção de texto

Não há tratamento nativo específico de copy/paste no wrapper — a WebView Android padrão já
oferece seleção de texto e copiar/colar nativamente (menu contextual do sistema ao segurar o
dedo sobre texto). O long-press custom (550ms) do menu de recarregar/limpar cache é registrado de
forma passiva e não deveria capturar o long-press sobre texto selecionável, mas **isso precisa ser
validado num device físico** — é o item 3 de "Validar no device" abaixo. Se o long-press do menu
"roubar" o long-press de seleção de texto, a correção é aumentar o delay ou checar
`window.getSelection().toString()` antes de abrir o menu customizado.

## Estado do build

- Toolchain Android validada de ponta a ponta.
- `cargo check` desktop passa limpo com as mudanças novas.
- Target `aarch64-linux-android` compila com as mudanças novas.
- APK debug (arm64-v8a) gerado com sucesso após ativar Developer Mode.

## Próximos passos

1. `npm run tauri android dev` com o celular plugado (ou instalar o `.apk` gerado via `adb install -r`).
2. **Validar no device**, em ordem de risco:
   - **OAuth Google/Facebook.** Maior risco do projeto. WebView Android é frequentemente
     bloqueada pelo Google (`disallowed_useragent`). A UA desktop spoofada pode ou não passar. Se
     bloquear, a saída padrão é Custom Tabs para a etapa de login, e aí vem o problema de o cookie
     ficar no cookie jar do Chrome e não na WebView. É o maior retrabalho potencial do projeto.
   - Init script chegou? No logcat, procurar erro de `clear_cache`; no console da WebView, checar
     `typeof window.__TAURI__` e `window.__nextagsMenuInstalled`. Existe issue aberta
     (tauri#7863) de init script não rodar em domínio remoto no Android — verificar, não presumir.
   - Long-press abre o menu sem atropelar seleção de texto nem long-press do próprio site (ver
     "Copiar e colar" acima).
   - Pull-to-refresh não briga com a lista de conversas do inbox (ela tem `overflowY` próprio e
     ~4972px de altura). O código só arma quando o container sob o dedo **e** a página estão no topo.
   - Teclado: abrir uma conversa e focar o campo de mensagem. O campo deve subir, não ficar coberto.
   - Status bar / nav bar não cobrindo a barra fixa de 48px do site.
   - Pinch-zoom funcionando (ajuda com os 146 nós de fonte < 12px).
   - "Limpar cache": confirmar que não erra mais no ACL. Lembrar que no Android não derruba a
     sessão (não limpa cookies).
3. Se o pinch-zoom não resolver a legibilidade, decidir sobre os 146 nós com fonte < 12px. É CSS do
   site. Opções: pedir ajuste ao time do site (correto), ou injetar CSS de mínimo de fonte pelo
   wrapper (risco de quebrar layout — medir antes).
4. Build release: precisa de keystore (ver "Publicação na Play Store" abaixo). Debug-signed serve
   para teste interno, não para distribuição.

## Aberto / não resolvido

- **Research de OAuth em WebView Android não foi feito.** Em aberto: o Google ainda bloqueia
  embedded WebView em 2026? A UA desktop spoofada passa? `tauri-plugin-opener` abre Custom Tabs no
  Android? O cookie sobrevive ao ida-e-volta Custom Tab → WebView (provavelmente **não**, cookie
  jars separados)?
- **Research de `identifier` por plataforma não foi feito.** Pergunta:
  `src-tauri/tauri.android.conf.json` permite sobrescrever `identifier` só no Android? Hoje o
  applicationId Android é `br.com.nextagsai.desktop` — funciona, mas "desktop" no ID de um app
  Android é estranho para a Play Store. Mudar o `identifier` global quebraria o caminho de upgrade
  do NSIS já entregue (instalaria lado a lado em vez de atualizar). **Decidir antes de publicar**;
  mudar depois é troca de identidade do app.
- **Emulador não instalou** numa tentativa anterior. O `sdkmanager` baixa, descompacta até 100%,
  registra em `.knownPackages` e **não grava no disco** — `emulator.exe` e `system.img` não existem
  em lugar nenhum, e reinstalar sai com exit 0 sem baixar nada. Contorno: apagar
  `<sdk>\.knownPackages` e reinstalar, ou usar o SDK Manager do Android Studio. Com device físico,
  não é bloqueador.
- APK debug tem ~230 MB (`.so` não-stripado, 2 ABIs) ou ~123 MB (só arm64). Release com
  `--target aarch64` e symbols stripados fica menor. Não otimizado ainda.

## Publicação na Play Store

Depende de conta e credenciais do usuário — trabalho manual do lado do usuário, documentado aqui
pra a sessão que for acompanhar poder orientar passo a passo.

1. **Conta Google Play Console**: criar em https://play.google.com/console (taxa única de
   registro, ~US$ 25). Verificação de identidade da conta de desenvolvedor pode levar de algumas
   horas a alguns dias — vale começar esse processo o quanto antes, em paralelo com o resto.
2. **Keystore de release**: gerar um keystore próprio pra assinar builds de produção (diferente
   do debug). Perguntar ao usuário se já existe uma keystore da NexTags antes de criar uma nova —
   **perder essa keystore depois de publicar significa não poder mais atualizar o app na mesma
   ficha da Play Store**, então precisa ser guardada com backup (ex.: 1Password/cofre da empresa),
   nunca só na máquina local.
   ```bash
   keytool -genkeypair -v -keystore nextags-release.keystore -alias nextags \
     -keyalg RSA -keysize 2048 -validity 10000
   ```
   Configurar o Gradle do projeto gerado (`src-tauri/gen/android/app/build.gradle.kts`, seção
   `signingConfigs`) pra usar esse keystore no build `release` — ver
   https://v2.tauri.app/distribute/google-play/ pro passo a passo específico do Tauri (inclui como
   referenciar keystore/senhas via variáveis de ambiente, sem commitar segredo no repo).
3. **Build de produção**: `npm run tauri android build` com o signing configurado gera o `.aab`
   assinado em `src-tauri/gen/android/app/build/outputs/bundle/release/`. É esse `.aab` que sobe
   na Play Console (não o `.apk`).
4. **Ficha da loja (Play Console → "Configurar app")**: preencher nome, descrição curta/completa,
   categoria, e-mail de contato, **política de privacidade (URL obrigatória)** — como o app faz
   login Google/Facebook e carrega `app.nextagsai.com.br`, essa política precisa existir e cobrir
   isso. Também precisa: ícone 512x512 (já temos, `src-tauri/icons/icon.png`), gráfico de
   destaque 1024x500, e pelo menos 2 screenshots de celular.
5. **Formulários obrigatórios**: classificação de conteúdo (questionário dentro do Console),
   "Segurança dos dados" (declarar que tipos de dado o app coleta/compartilha — relevante pelo
   OAuth Google/Facebook e pelo carregamento do site), público-alvo, e anúncios (declarar que não
   tem, se for o caso).
6. **Teste antes de produção**: contas novas de desenvolvedor no Google Play são obrigadas a rodar
   uma trilha de teste (fechado, com pelo menos ~12-20 testadores) por 14 dias corridos antes de
   poder publicar em produção — checar o requisito atual na própria Console ao criar o app, essa
   regra do Google muda de vez em quando. Vale já separar uma lista de e-mails de testadores
   (equipe interna serve) pra não travar nesse passo.
7. **Enviar pra revisão**: depois da trilha de teste, promover o release pra produção. Primeira
   revisão do Google costuma levar de algumas horas a poucos dias.

## Decisões já tomadas — não relitigar

- **Sem abas**, janela única. Proposital.
- **Sem FAB.** O canto inferior direito já é do widget de suporte do site. Recarregar fica em
  pull-to-refresh **e** no long-press; limpar cache só no long-press.
- **Não regerar o ícone do zero.** `app-icon-source.png` está aprovado (hexágono azul da marca, sem
  texto). Só rodar `tauri icon` quando o init sobrescrever, seguido de
  `fix_android_adaptive_icon.py`.
- **Manter a UA desktop.** Medido: não afeta o layout mobile, e serve ao OAuth.
- **Não presumir qual é o problema de UI mobile sem simular primeiro.**
- **Não commitar keystore, senhas ou service account JSON da Play Console neste repo** — usar
  variáveis de ambiente/secrets do CI ou um cofre de senhas.

## Armadilha de shell que gerou diagnóstico errado

`comando | tail -20; echo "EXIT=$?"` reporta o exit code do **`tail`**, não do comando. Isso fez
builds falhados parecerem sucesso. Use `${PIPESTATUS[0]}`.

## Referência rápida de arquivos

- `src-tauri/src/lib.rs` — janela, menu de contexto/long-press, pull-to-refresh, comando `clear_cache`
- `src-tauri/build.rs` — manifest de comandos/permissões
- `src-tauri/capabilities/remote.json` — capability de IPC remoto (`clear_cache`)
- `src-tauri/.cargo/config.toml` — flags de WebView Android (pinch-zoom)
- `src-tauri/tauri.conf.json` — config do bundle, ícones, identifier
- `src-tauri/gen/android/.../MainActivity.kt` — insets nativos (status bar/teclado)
- `src-tauri/gen/android/.../AndroidManifest.xml` — `windowSoftInputMode`
- `app-icon-source.png` — fonte do ícone (raiz do projeto)
- `scripts/fix_android_adaptive_icon.py` — reaplica a margem de segurança do adaptive icon Android
  (rodar depois de qualquer `npx tauri icon` novo)
- `dist-cliente/` — builds prontos pra entrega desktop (não mexer, é output)
