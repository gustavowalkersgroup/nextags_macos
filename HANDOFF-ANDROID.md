# Handoff — App Android NexTags AI (Tauri)

**Atualizado em 31/07/2026 ~20:25.** Substitui a versão anterior deste arquivo.

## Abra a próxima sessão LOCALMENTE, não em nuvem

O celular de teste está no PC local. `adb` só enxerga dispositivo USB da máquina onde roda, e
`tauri android dev` exige o device na **mesma máquina que compila**. Sessão em nuvem ou no PC
remoto da empresa não serve para o passo que falta.

Consequência: o toolchain Android está instalado no **PC remoto da empresa**, não no local. Ver
"Setup no PC local" abaixo.

## Objetivo

Empacotar o mesmo projeto Tauri (já entregue como app desktop Windows) como APK Android,
carregando `https://app.nextagsai.com.br/` numa WebView nativa. Janela única, sem abas — proposital.

## Como pegar o código

O trabalho Android está commitado na branch **`feat/android`** de
`https://github.com/gustavowalkersgroup/nextags_macos`.

```bash
git clone https://github.com/gustavowalkersgroup/nextags_macos.git
cd nextags_macos
git checkout feat/android
npm install
```

`main` continua com o build desktop/macOS intocado. Todas as mudanças de comportamento Android são
guardadas por `#[cfg(target_os = "android")]` / `#[cfg(desktop)]`, então `main` pode fazer merge sem
risco para os builds Windows/macOS/Linux do CI.

## NÃO trabalhe a partir de `Z:\Gustavo\nextags-desktop-app`

`Z:` é drive de rede mapeado (`\\nextagsdados\nextags`, DriveType 4). O `cargo-mobile2`
canonicaliza o path para UNC (`\\nextagsdados\...`) e a validação `starts_with(root_dir)` falha:

```
AssetDirOutsideOfAppRoot { asset_dir: "assets", root_dir: "...\src-tauri" }
```

Mesmo bug do issue tauri#8715. **`tauri android init` não roda de share SMB.** Use um caminho
local (`C:\dev\...`). Confirmado empiricamente: em `C:` o init passa.

Aviso: durante a sessão de 31/07 havia **outra sessão editando `Z:` ao mesmo tempo** (o build
macOS, ver `HANDOFF-MAC-BUILD.md`). `Z:` virou repo git às 19:25. Não presuma que `Z:` está
estático.

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

Ou pela GUI: Configurações → Sistema → Para desenvolvedores → Modo de desenvolvedor.

Contorno sem admin (só se travar): compile o `.so`, copie manualmente para
`jniLibs/<abi>/` e rode o Gradle direto, pulando a task Rust — foi assim que os APKs de 31/07
saíram:

```bash
cp src-tauri/target/aarch64-linux-android/debug/libtauri_app_lib.so \
   src-tauri/gen/android/app/src/main/jniLibs/arm64-v8a/
cd src-tauri/gen/android && ./gradlew assembleArm64Debug -x rustBuildArm64Debug
```

Isso **não** contorna `tauri android dev` (o Gradle chama `tauri android android-studio-script`,
que refaz o mesmo symlink). Para iterar com live reload, o Developer Mode é necessário.

### 2. Toolchain

Versões exatas usadas no PC remoto (replicar):

| Componente | Versão / caminho |
|---|---|
| JDK | Microsoft OpenJDK 17.0.20 (`winget install Microsoft.OpenJDK.17`) |
| Android SDK | cmdline-tools + `platform-tools` |
| SDK platforms | **`android-36`** (o Gradle gerado usa `compileSdk = 36` / `targetSdk = 36`) |
| build-tools | `36.0.0` |
| NDK | `27.0.12077973` |
| Rust targets | `aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android` |
| tauri-cli | 2.11.4 (via `@tauri-apps/cli ^2`) |

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

## Estado atual — pronto e verificado

- **APK debug gerado e válido**: `app-universal-debug.apk`, 230 MB, arm64-v8a + x86_64,
  `BUILD SUCCESSFUL in 5m11s`. Ainda **não instalado em device nenhum**.
- Ícone da marca aplicado no projeto Android (hashes conferem com `src-tauri/icons/android/`).
- Toolchain Android validada de ponta a ponta no PC remoto.
- `cargo check` desktop passa limpo com as mudanças novas.
- Target `aarch64-linux-android` compila com as mudanças novas (o build só para no symlink).

### Ordem que importa: `tauri icon` DEPOIS de `android init`

`tauri android init` sobrescreve os mipmaps com o placeholder do template. Rodar
`npx tauri icon app-icon-source.png` **depois** do init corrige (verificado por hash). Se
reinicializar o projeto Android, repita.

## Medições sobre a UI mobile — não repita este trabalho

Chrome DevTools em 390×844 e `curl` com três UAs, em 31/07:

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
   **não funciona**. Tem de ser nativo.
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
   funcionar, mas o cache nunca era limpo. **Corrigido nesta branch** (`build.rs` +
   `capabilities/remote.json`). Confirmar no console/logcat quando testar.
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

## Mudanças feitas nesta branch

| Arquivo | O quê |
|---|---|
| `src-tauri/src/lib.rs` | Menu compartilhado com guarda de idempotência. Android: **long-press** (550ms) abre menu com "Recarregar" + "Limpar cache e recarregar"; **pull-to-refresh** (90px) recarrega. Listeners todos `passive`. `inner_size`/`min_inner_size` agora sob `#[cfg(desktop)]`. Erro de `build()` logado em vez de panic opaco. |
| `src-tauri/build.rs` | `app_manifest` declarando o comando `clear_cache` (gera a permissão `allow-clear-cache`). |
| `src-tauri/capabilities/remote.json` | Novo. Capability com `remote.urls` liberando `clear_cache` para a origem remota. Corrige o item 6 acima **no desktop também**. |
| `src-tauri/.cargo/config.toml` | Novo. `WRY_RUSTWEBVIEW_CLASS_INIT` habilitando `builtInZoomControls` (pinch-zoom), `displayZoomControls=false`, `useWideViewPort`, `loadWithOverviewMode`. Só afeta `target_os = "android"`. |
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

## Próximos passos

1. Setup no PC local (Developer Mode + toolchain + device).
2. `npm run tauri android dev` com o celular plugado. Se falhar,
   `npx tauri android build --debug --apk --target aarch64` e `adb install -r <apk>`.
3. **Validar no device**, em ordem de risco:
   - **OAuth Google/Facebook.** Maior risco do projeto. WebView Android é frequentemente
     bloqueada pelo Google (`disallowed_useragent`). A UA desktop spoofada pode ou não passar —
     **não foi possível pesquisar isso** (ver "Aberto"). Se bloquear, a saída padrão é Custom Tabs
     para a etapa de login, e aí vem o problema de o cookie ficar no cookie jar do Chrome e não na
     WebView. É o maior retrabalho potencial do projeto.
   - Init script chegou? No logcat, procurar erro de `clear_cache`; no console da WebView, checar
     `typeof window.__TAURI__` e `window.__nextagsMenuInstalled`. Existe issue aberta
     (tauri#7863) de init script não rodar em domínio remoto no Android — verificar, não presumir.
   - Long-press abre o menu sem atropelar seleção de texto nem long-press do próprio site.
   - Pull-to-refresh não briga com a lista de conversas do inbox (ela tem `overflowY` próprio e
     ~4972px de altura). O código só arma quando o container sob o dedo **e** a página estão no topo.
   - Teclado: abrir uma conversa e focar o campo de mensagem. O campo deve subir, não ficar coberto.
   - Status bar / nav bar não cobrindo a barra fixa de 48px do site.
   - Pinch-zoom funcionando (ajuda com os 146 nós de fonte < 12px).
   - "Limpar cache": confirmar que não erra mais no ACL. Lembrar que no Android não derruba a
     sessão (não limpa cookies).
4. Se o pinch-zoom não resolver a legibilidade, decidir sobre os 146 nós com fonte < 12px. É CSS do
   site. Opções: pedir ajuste ao time do site (correto), ou injetar CSS de mínimo de fonte pelo
   wrapper (risco de quebrar layout — medir antes).
5. Build release: precisa de keystore. **Perguntar se a NexTags já tem uma** ou se cria nova.
   Debug-signed serve para teste interno, não para distribuição.

## Aberto / não resolvido

- **Research de OAuth em WebView Android não foi feito.** O agente morreu no limite de sessão.
  Em aberto: o Google ainda bloqueia embedded WebView em 2026? A UA desktop spoofada passa?
  `tauri-plugin-opener` abre Custom Tabs no Android? O cookie sobrevive ao ida-e-volta Custom Tab →
  WebView (provavelmente **não**, cookie jars separados)?
- **Research de `identifier` por plataforma não foi feito** (mesma causa). Pergunta:
  `src-tauri/tauri.android.conf.json` permite sobrescrever `identifier` só no Android? Hoje o
  applicationId Android é `br.com.nextagsai.desktop` — funciona, mas "desktop" no ID de um app
  Android é estranho para a Play Store. Mudar o `identifier` global quebraria o caminho de upgrade
  do NSIS já entregue (instalaria lado a lado em vez de atualizar). **Decidir antes de publicar**;
  mudar depois é troca de identidade do app. Trocar agora custa
  `rm -rf src-tauri/gen/android && tauri android init && tauri icon`.
- **Emulador não instalou.** O `sdkmanager` baixa, descompacta até 100%, registra em
  `.knownPackages` e **não grava no disco** — `emulator.exe` e `system.img` não existem em lugar
  nenhum, e reinstalar sai com exit 0 sem baixar nada (ele acredita que já instalou). Contorno:
  apagar `<sdk>\.knownPackages` e reinstalar, ou usar o SDK Manager do Android Studio. Com device
  físico, não é bloqueador.
- APK debug tem 230 MB (`.so` não-stripado, 2 ABIs). Release com `--target aarch64` e symbols
  stripados fica muito menor. Não otimizado ainda.

## Decisões já tomadas — não relitigar

- **Sem abas**, janela única. Proposital.
- **Sem FAB.** O canto inferior direito já é do widget de suporte do site. Recarregar fica em
  pull-to-refresh **e** no long-press; limpar cache só no long-press.
- **Não regerar o ícone do zero.** `app-icon-source.png` está aprovado (hexágono azul da marca, sem
  texto). Só rodar `tauri icon` quando o init sobrescrever.
- **Manter a UA desktop.** Medido: não afeta o layout mobile, e serve ao OAuth.
- **Entrega por commit no GitHub**, branch separada.

## Armadilha de shell que gerou diagnóstico errado

`comando | tail -20; echo "EXIT=$?"` reporta o exit code do **`tail`**, não do comando. Isso fez
builds falhados parecerem sucesso. Use `${PIPESTATUS[0]}`.
