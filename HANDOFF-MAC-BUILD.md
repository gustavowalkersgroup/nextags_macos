# Handoff — Build macOS (e Linux) via GitHub Actions

Contexto: build nativo macOS não pode ser gerado a partir de Windows (falta toolchain Apple/Xcode,
que só roda em Mac). Solução: repositório GitHub com workflow de CI que compila em runner macOS
hospedado pela própria GitHub — sem precisar de Mac físico.

## Repositório

`https://github.com/gustavowalkersgroup/nextags_macos` (público). Projeto na branch `main`,
incluindo `.github/workflows/build.yml`.

## Como disparar o build

Duas formas (o workflow está configurado para as duas):

1. **Manual**: aba "Actions" do repo → workflow "Build NexTags AI Desktop" → "Run workflow".
2. **Por tag**: `git tag v0.1.0 && git push origin v0.1.0` — qualquer tag `v*` dispara o build.

Atenção ao nome do release: o tag do release é sempre `app-v<versão do tauri.conf.json>`, **não**
o tag que você empurrou. Empurrar `v0.2.0` sem antes subir a versão em `src-tauri/tauri.conf.json`
gera um release chamado `app-v0.1.0`. Para soltar uma versão nova, suba a versão em
`src-tauri/tauri.conf.json` (e, por coerência, em `package.json` e `src-tauri/Cargo.toml`) antes de
criar o tag.

O prefixo `app-` é proposital: sem ele, publicar o release criaria um tag `v*` que dispararia o
workflow de novo, em loop.

## O que o workflow gera

Matriz de 4 builds em paralelo:

| Job | Artefato |
|---|---|
| macOS Apple Silicon (`aarch64-apple-darwin`) | `.dmg` + `.app.tar.gz` |
| macOS Intel (`x86_64-apple-darwin`) | `.dmg` + `.app.tar.gz` |
| Linux (`ubuntu-24.04`) | `.deb` + `.AppImage` |
| Windows (`windows-latest`) | instalador NSIS `.exe` |

Os binários viram um **GitHub Release em modo rascunho (draft)** — não fica público
automaticamente. Precisa ir em "Releases" no repo e clicar em "Publish release" pra liberar (ou
baixar direto os arquivos anexados ao rascunho, sem publicar, se for só pra teste interno).

Os alvos de bundle são definidos por plataforma, não num lugar só:

- `src-tauri/tauri.conf.json` — base, `nsis` (Windows)
- `src-tauri/tauri.macos.conf.json` — `app`, `dmg`
- `src-tauri/tauri.linux.conf.json` — `deb`, `appimage`

O Tauri lê automaticamente o arquivo da plataforma atual e mescla sobre o base (arrays substituem,
não concatenam). Se um dia o macOS parar de gerar `.dmg`, é nesse arquivo que se olha primeiro.

## Ponto de atenção — assinatura de código macOS

Build sai **sem assinatura Apple** (não configuramos certificado de desenvolvedor). Ao abrir o
`.app`/`.dmg` num Mac, o Gatekeeper bloqueia com "não é possível verificar o desenvolvedor".

**O contorno mudou.** A instrução antiga (botão direito → "Abrir") não funciona mais: a Apple
removeu esse atalho no macOS 15 Sequoia, em 2024. No macOS 15 e 26, o caminho é:

1. Tentar abrir o app normalmente (vai ser bloqueado — é esperado).
2. Ajustes do Sistema → Privacidade e Segurança → rolar até a seção Segurança.
3. Clicar em "Abrir Mesmo Assim" ao lado da mensagem sobre o app bloqueado.
4. Confirmar com senha de administrador.

Se aparecer "o app está danificado e não pode ser aberto", é a flag de quarentena do download:
`xattr -dr com.apple.quarantine "/Applications/NexTags AI.app"`.

São 4 passos numa caixa de Ajustes do Sistema, não um clique — vale calibrar a expectativa de
quem for instalar.

### Assinatura ad-hoc (aplicada, e o que ela resolve)

`tauri.macos.conf.json` agora traz `bundle.macOS.signingIdentity: "-"`. É assinatura **ad-hoc**:
grátis, sem conta Apple, feita no próprio runner.

Ela **não** satisfaz o Gatekeeper — o aviso de desenvolvedor não verificado continua. O que ela
resolve é outra coisa, e importa: sem nenhuma assinatura, o macOS 15.1+ tende a mostrar **"o app
está danificado e não pode ser aberto"**, que *não tem botão de contorno na interface*. Com o selo
ad-hoc consistente, o app cai no diálogo normal, que tem saída via Ajustes do Sistema.

Em resumo: ad-hoc não remove o atrito, torna o atrito transponível sem Terminal.

Efeito colateral conhecido: há bug intermitente do Tauri no bundle do DMG com ad-hoc
([tauri#13804](https://github.com/tauri-apps/tauri/issues/13804)). Se o job de macOS falhar no passo
do DMG sem motivo aparente, re-rodar costuma resolver.

### Eliminar o aviso de vez

Só notarização faz isso, e notarização exige certificado Developer ID — que só existe dentro do
Apple Developer Program (US$ 99/ano). Não há atalho. Uma vez com a conta, é automatizável no mesmo
workflow via `tauri-action`, com os secrets `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`,
`APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`.

**Decisão tomada (ago/2026): o público inclui cliente externo.** Isso fecha a questão — para máquina
que a NexTags não gerencia, os US$ 99/ano são a opção mais barata e limpa. As alternativas gratuitas
só alcançam Mac gerenciado.

### Alternativas que foram investigadas e NÃO servem

Registrado pra ninguém gastar tempo de novo:

- **Conta Apple gratuita ("Personal Team")** — assina, o `codesign -v` passa, e o `spctl` continua
  rejeitando. É o erro mais comum do assunto.
- **Certificado autoassinado + CA confiável no Keychain**, inclusive distribuída por perfil MDM —
  não funciona. Gatekeeper exige Developer ID emitido pela Apple, ponto.
- **Isenção da taxa de US$ 99** — existe, mas só para ONG reconhecida, instituição de ensino
  credenciada ou órgão governamental. A NexTags não se qualifica.
- **Homebrew cask com `--no-quarantine`** — removido no brew 5.1; sai de circulação de vez em
  01/09/2026. Vale inclusive para tap próprio/privado.
- **Trocar `.dmg` por `.pkg`** — não melhora e piora a segurança: o Installer roda como root.
- **Microsoft Intune** — exige `.pkg` assinado com Developer ID Installer. Os US$ 99 voltam pela
  porta dos fundos.

### Alternativas que funcionam, mas só para Mac gerenciado

Não resolvem o caso da NexTags (cliente externo), ficam registradas caso o público mude:

- **MDM com agente próprio** (Mosyle Business FREE cobre 30 Macs de graça, permanente). O agente
  instala como root e o arquivo nunca recebe a flag de quarentena, então o Gatekeeper não entra na
  história e o usuário não vê aviso nenhum.
- **Canal de entrega que não seta quarentena** (pendrive, SMB, `curl`). Atenção: essa rota saiu
  **contestada** na verificação — não confie sem testar na prática.

### A alternativa de fundo: não distribuir binário

O app é um wrapper de webview, e o `tauri.conf.json` confirma que ele não usa nada nativo —
`windows: []`, nenhum plugin, nenhum updater, nenhum deep link. Um PWA instalado via Safari
("Arquivo → Adicionar ao Dock") ou Chrome dá janela própria, ícone no Dock e sessão isolada, com
atrito zero, custo zero e sem Gatekeeper nenhum.

O que trava essa rota: o wrapper forja user-agent de Chrome/Windows, o que sugere que
`app.nextagsai.com.br` talvez não funcione bem no Safari. **Esse teste não foi feito** — o egress
desta sessão bloqueia o domínio com 403 de política. Abrir o site no Safari de um Mac resolve a
dúvida em 30 segundos, e o resultado decide se essa alternativa é viável.

## Correções feitas antes do primeiro run (jul/2026)

O workflow tinha sido escrito mas nunca executado, e não teria produzido artefato de macOS nem de
Linux. O que estava quebrado:

- **`bundle.targets` fixo em `["nsis"]`.** O bundler intersecta os alvos configurados com os
  suportados pela plataforma; `nsis` não existe no macOS nem no Linux, a interseção ficava vazia, e
  `tauri build` saía com código 0 **sem gerar nada e sem avisar**. O `tauri-action` então falhava
  com `No artifacts were found.` Resolvido com os arquivos de config por plataforma acima.
- **Workflow sem `permissions: contents: write`.** O `GITHUB_TOKEN` padrão é somente-leitura em
  repos criados depois de fev/2023, e o `tauri-action` cria o release **depois** de compilar — a
  falha 403 apareceria só no fim de 10-25 min de build, nos quatro jobs.
- **`xdg-utils` faltando no runner Linux.** O bundler do AppImage copia o `/usr/bin/xdg-open` pra
  dentro do AppDir, e esse pacote não vem na imagem do `ubuntu-24.04`. O job gerava o `.deb` e
  morria no AppImage.
- **ACL recusava o `invoke` da página remota.** A janela carrega `https://app.nextagsai.com.br/`,
  que o Tauri trata como origem remota; a capability não declarava essa origem nem registrava o
  comando `clear_cache`. O item "Limpar cache e recarregar" do menu de contexto só recarregava, sem
  limpar cache — em todas as plataformas, inclusive no build Windows já entregue. Corrigido em
  `src-tauri/build.rs` (registra o comando, gerando `allow-clear-cache`) e em
  `src-tauri/capabilities/default.json` (declara a URL remota).

Também: runner Linux pinado em `ubuntu-24.04` (o `ubuntu-latest` vai migrar pro 26.04, onde
`libappindicator3-dev` não existe mais), cache de Rust, `npm ci`, e metadados de marca no bundle
(antes o `.deb` e o `Info.plist` saíam como "A Tauri App", autor "you").

## Pendências conhecidas

- **Assinatura Apple** — decisão do usuário, ver acima.
- **User-agent falso de Windows** (`src-tauri/src/lib.rs`). A janela se identifica como
  `Chrome/128 on Windows NT 10.0` em todas as plataformas, inclusive no Mac. Considerei corrigir
  por plataforma e desisti: a string já está desatualizada em todos os alvos, e trocá-la pelo UA
  nativo do WKWebView/WebKitGTK tende a piorar login OAuth do Google, que bloqueia justamente a
  assinatura de webview embarcada. Não há histórico no repo que mostre se o spoof foi acidental ou
  se foi contorno deliberado de alguma checagem do `app.nextagsai.com.br`. Quem conhece o
  comportamento do app web decide.
- **"Limpar cache e recarregar" não foi testado em runtime.** A correção de ACL foi validada em
  tempo de compilação; confirmar clicando no item do menu num build real.
- **Depreciação do Node 20 nos runners.** O run atual passa, mas emite aviso: `actions/checkout@v4`
  e `actions/setup-node@v4` têm runtime Node 20 e estão sendo forçados pra Node 24. Quando a GitHub
  cortar de vez, o workflow quebra. Correção quando for conveniente: subir as duas actions pra `@v5`
  (e, de quebra, o `node-version: 20` pra 22). Não mexi agora pra não arriscar o primeiro build
  verde.

## Não fazer

- Não publicar o Release automaticamente sem o usuário confirmar (por isso `releaseDraft: true`
  no workflow) — quem decide tornar público é ele, manualmente, na interface do GitHub.
- Não commitar `dist-cliente/`, `src-tauri/target/`, `src-tauri/gen/` — já estão no `.gitignore`,
  são output/regenerável.
- Não trocar `tagName` pra `v__VERSION__` sem antes ajustar o gatilho: publicar o release passaria
  a disparar o workflow em loop.

## Arquivos relevantes

- `.github/workflows/build.yml` — workflow de CI
- `src-tauri/src/lib.rs`, `tauri.conf.json` — janela única, menu de contexto, ícone da marca
- `src-tauri/capabilities/default.json` — ACL (precisa listar a origem remota)
- `app-icon-source.png` — fonte do ícone (não precisa regenerar)
- `HANDOFF-ANDROID.md` — handoff separado pra app Android (outra sessão, mesma base de projeto)
