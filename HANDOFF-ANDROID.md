# Handoff — App Android NexTags AI (Tauri)

Contexto pra sessão nova (recomendado rodar com Opus, tarefa de config nativa Android tende a
precisar de mais raciocínio pra troubleshooting de toolchain).

## Objetivo

Empacotar o mesmo projeto Tauri (que já roda como app desktop Windows) também como APK Android,
carregando `https://app.nextagsai.com.br/` numa WebView nativa. Sem abas — janela única, é
proposital (feedback do usuário: "não tem abas né, mas acho que justamente é essa a ideia").

## Estado atual (o que já está pronto)

**Projeto**: `Z:\Gustavo\nextags-desktop-app` (scaffold Tauri 2, criado via `npm create tauri-app`)

**Desktop Windows já funcional e entregue**:
- `src-tauri/src/lib.rs` — janela única carregando `APP_URL` direto (sem frontend local), UA de
  Chrome desktop pra evitar bloqueio OAuth do Google, menu de contexto customizado (botão direito
  → "Recarregar página" / "Limpar cache e recarregar", via `initialization_script` JS +
  comando Rust `clear_cache` que chama `window.clear_all_browsing_data()`).
- `src-tauri/tauri.conf.json` — `productName: "NexTags AI"`, `identifier: br.com.nextagsai.desktop`,
  bundle targets `["nsis"]`, ícone da marca já aplicado (ver abaixo).
- Ícone: `app-icon-source.png` (1024x1024, hexágono azul da marca, fundo transparente, extraído e
  limpo de `nextags.jpg`) → já rodei `npx tauri icon app-icon-source.png`, gerou todos os tamanhos
  em `src-tauri/icons/` **incluindo os mipmaps Android** (`android/mipmap-*`) e ícones iOS — não
  precisa regenerar, só reaproveitar.
- Build release + instalador NSIS testados e aprovados pelo usuário (login Google/Facebook
  funcionando dentro da própria janela, cache/tradutor do Chrome não incomodam mais).
- Entregáveis ficam em `dist-cliente/` (portable .exe + instalador .exe) — script de cópia simples,
  repetir se gerar nova release.

## Toolchain Android já instalado nesta máquina

- **JDK 17**: `C:\Program Files\Microsoft\jdk-17.0.20.8-hotspot` (via winget
  `Microsoft.OpenJDK.17`)
- **Android SDK**: `C:\Android\sdk` (cmdline-tools em `C:\Android\sdk\cmdline-tools\latest\bin`)
  - `platform-tools`, `platforms;android-34`, `build-tools;34.0.0` instalados
  - Licenças já aceitas
- **NDK**: `C:\Android\sdk\ndk\27.0.12077973`

**Variáveis de ambiente que a sessão precisa exportar** (não estão persistidas no perfil do
usuário ainda — só usadas ad-hoc nas sessões anteriores):

```bash
export JAVA_HOME="/c/Program Files/Microsoft/jdk-17.0.20.8-hotspot"
export ANDROID_HOME=/c/Android/sdk
export NDK_HOME=/c/Android/sdk/ndk/27.0.12077973
```

Se for rodar via PowerShell (recomendado pros comandos do `tauri android`, que tendem a invocar
`.bat`/gradle nativos do Windows melhor por lá do que via Git Bash):

```powershell
$env:JAVA_HOME = "C:\Program Files\Microsoft\jdk-17.0.20.8-hotspot"
$env:ANDROID_HOME = "C:\Android\sdk"
$env:NDK_HOME = "C:\Android\sdk\ndk\27.0.12077973"
$env:Path = "$env:JAVA_HOME\bin;$env:ANDROID_HOME\platform-tools;$env:Path"
```

**Vale considerar persistir essas variáveis via `setx`** no início da sessão nova, pra não ter que
reexportar toda hora.

## ATUALIZAÇÃO (ago/2026) — o build Android já roda no CI

O passo 2 abaixo (`tauri android init`) **não precisa mais ser feito à mão numa máquina Windows**.
O `.github/workflows/build.yml` tem um job `android` que faz tudo em runner Linux da GitHub:
instala JDK 17, detecta o NDK da imagem, adiciona os targets Rust, roda `android init` (necessário a
cada run, porque `src-tauri/gen/` é gitignored), compila e anexa o APK ao release rascunho.

**O APK sai assinado em DEBUG, de propósito.** Sem keystore, `tauri android build` de release
produz `app-universal-release-unsigned.apk` — e o Android **não instala APK sem assinatura**. O de
debug é assinado com a chave de debug do Android SDK e instala normalmente (basta permitir "fontes
desconhecidas"). Serve para testar; não serve para produção.

### Para virar APK de produção, faltam três coisas

1. **Keystore da NexTags.** Perguntar se já existe uma; se não, criar com `keytool` e guardar como
   secret do repo (base64). Importa que seja **a mesma para sempre**: o Android só aceita atualizar
   um app instalado se a nova versão tiver a mesma assinatura. Trocar de chave obriga o usuário a
   desinstalar e perder os dados locais.
2. **Ajuste de UI mobile** — o passo 3 abaixo, que era o motivo original do pedido. Continua
   pendente e não dá para fazer sem device/emulador.
3. **OAuth Google/Facebook em WebView** — o passo 7 abaixo. Continua sendo o maior risco do projeto
   e continua não testado.

Os itens 2 e 3 exigem rodar o app num aparelho de verdade. O APK de debug do release rascunho existe
justamente para isso.

## Próximos passos (ordem sugerida)

1. **Exportar env vars acima** na sessão nova (testar `java -version`, `echo $ANDROID_HOME` antes
   de seguir).
2. Rodar `npm run tauri android init` dentro de `Z:\Gustavo\nextags-desktop-app` — isso gera a
   pasta `src-tauri/gen/android/` com projeto Gradle completo.
3. **Ajustar UI mobile** — esse é o motivo original do pedido ("UIX zuada"). Testar
   `npm run tauri android dev` (emulador ou device físico via USB debugging) e observar o que
   quebra no layout do site em viewport mobile:
   - Se for CSS responsivo do próprio site (`app.nextagsai.com.br` não adaptado pra telas
     pequenas) — não dá pra corrigir pelo wrapper, precisa mexer no CSS do site em si (fora de
     escopo deste projeto Tauri, seria outro time).
   - Se for comportamento do WebView (zoom automático, teclado cobrindo campo de input, gestos de
     navegação por swipe conflitando) — dá pra corrigir via `initialization_script` (mesmo padrão
     já usado no desktop pro menu de contexto) injetando CSS/JS: `viewport` meta fixo, prevenção
     de double-tap-zoom, ajuste de `resize` ao abrir teclado, etc. **Recomendo simular o site no
     Chrome DevTools em viewport mobile primeiro** (ferramenta `chrome-devtools-mcp` disponível)
     antes de decidir o que precisa de fix — usuário pediu explicitamente pra descobrir isso
     simulando, não assumir.
4. **Menu de contexto / cache no mobile**: Android não tem "botão direito" — provavelmente vale
   um long-press no título ou um botão flutuante (FAB) discreto com as mesmas duas ações
   (recarregar / limpar cache). Perguntar ao usuário preferência antes de implementar (long-press
   vs. botão visível vs. pull-to-refresh nativo pra reload).
5. Ícone Android já está gerado (mipmaps), mas confirmar que `tauri android init` não sobrescreve
   com o ícone placeholder — se sobrescrever, rodar `npx tauri icon app-icon-source.png` de novo
   depois do init.
   - **Atenção**: `tauri icon` só redimensiona `app-icon-source.png` 1:1 pro canvas de cada
     densidade do `ic_launcher_foreground.png` — não deixa a margem ("safe zone") que o Android
     precisa pra mascarar o ícone em círculo/squircle/rounded-square sem cortar o hexágono. Era
     esse o motivo do ícone "vazar o azul" (a borda azul ficava perto demais da borda do canvas e
     lançadores mais agressivos no corte — Samsung/MIUI/etc. — cortavam ela de forma desigual).
     Já corrigido nos `src-tauri/icons/android/mipmap-*/ic_launcher_foreground.png` deste repo
     (logo redimensionado pra ~60% do canvas, dentro do limite de 66dp/108dp recomendado pelo
     Google). **Se rodar `npx tauri icon` de novo, rode `python3 scripts/fix_android_adaptive_icon.py`
     em seguida** pra reaplicar a margem antes de gerar/buildar o projeto Android.
6. Build APK/AAB: `npm run tauri android build` (gera `.apk`/`.aab` em
   `src-tauri/gen/android/app/build/outputs/`). Pra instalar direto num Android de teste sem Play
   Store, `.apk` assinado em debug já serve; produção (Play Store) precisa de AAB assinado com
   keystore de release — ver seção "Publicação na Play Store" abaixo.
7. **OAuth Google/Facebook no Android**: diferente do desktop, WebView Android **é** frequentemente
   bloqueada pelo Google ("disallowed_useragent") pra login OAuth — política do Google desde 2016
   contra embedded webviews em apps nativos. Se acontecer, a solução padrão é usar
   `Custom Tabs`/browser do sistema pra etapa de login (Tauri não tem isso pronto — precisaria de
   plugin ou WebView intent customizado). Testar primeiro antes de assumir que vai falhar; se
   falhar, esse é o ponto de maior risco/retrabalho do projeto todo.

## Publicação na Play Store

Nada disso pode ser feito a partir deste repo/sessão — depende de conta e credenciais do usuário
e do toolchain Android (item "Toolchain Android" acima). É trabalho manual do lado do usuário,
mas documentado aqui pra a sessão que for fazer o build/acompanhar poder orientar passo a passo.

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

## Coisas que NÃO fazer

- Não trocar a arte/logo do ícone — `app-icon-source.png` já está pronto e aprovado (é a marca
  NexTags, hexágono azul, sem texto — usuário pediu explicitamente sem texto). A correção de
  padding do ícone Android (item 5 acima) só ajusta a margem de segurança do adaptive icon, não
  muda o desenho.
- Não mudar a decisão de "sem abas" — é intencional.
- Não presumir qual é o problema de UI mobile sem simular primeiro — usuário quer que seja
  descoberto na prática, não adivinhado.
- Não commitar keystore, senhas ou service account JSON da Play Console neste repo — usar
  variáveis de ambiente/secrets do CI ou um cofre de senhas.

## Referência rápida de arquivos

- `src-tauri/src/lib.rs` — lógica da janela, menu de contexto, comando `clear_cache`
- `src-tauri/tauri.conf.json` — config do bundle, ícones, identifier
- `app-icon-source.png` — fonte do ícone (raiz do projeto)
- `scripts/fix_android_adaptive_icon.py` — reaplica a margem de segurança do adaptive icon Android
  (rodar depois de qualquer `npx tauri icon` novo)
- `dist-cliente/` — builds prontos pra entrega desktop (não mexer, é output)
