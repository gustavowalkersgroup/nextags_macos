# Handoff — Build macOS (e Linux) via GitHub Actions

Contexto: build nativo macOS não pode ser gerado a partir de Windows (falta toolchain Apple/Xcode,
que só roda em Mac). Solução: repositório GitHub com workflow de CI que compila em runner macOS
hospedado pela própria GitHub — sem precisar de Mac físico.

## Repositório

`https://github.com/gustavowalkersgroup/nextags_macos` (público). Projeto já enviado (branch
`main`), incluindo `.github/workflows/build.yml`.

## Como disparar o build

Duas formas (o workflow está configurado para as duas):

1. **Manual**: aba "Actions" do repo → workflow "Build NexTags AI Desktop" → "Run workflow".
2. **Por tag**: `git tag v0.1.0 && git push origin v0.1.0` — qualquer tag `v*` dispara o build
   automaticamente.

## O que o workflow gera

Matriz de 4 builds em paralelo:
- macOS Apple Silicon (`aarch64-apple-darwin`)
- macOS Intel (`x86_64-apple-darwin`)
- Linux (`ubuntu-latest`, gera `.deb`/`.AppImage`)
- Windows (`windows-latest`, gera `.exe`/instalador NSIS — já temos isso localmente, mas fica
  redundante/consistente ali também)

Os binários viram um **GitHub Release em modo rascunho (draft)** — não fica público
automaticamente. Precisa ir em "Releases" no repo e clicar em "Publish release" pra liberar (ou
baixar direto os arquivos anexados ao rascunho, sem publicar, se for só pra teste interno).

## Ponto de atenção — assinatura de código macOS

Build sai **sem assinatura Apple** (não configuramos certificado de desenvolvedor). Isso significa
que ao abrir o `.app`/`.dmg` num Mac, o Gatekeeper vai bloquear com "não é possível verificar o
desenvolvedor" ou "app está danificado". Contorno pro usuário final: botão direito no app →
"Abrir" (em vez de duplo-clique) → confirma na caixa de diálogo. Funciona, mas é fricção extra.

Pra eliminar esse aviso de vez, precisa de:
- Conta Apple Developer (US$ 99/ano)
- Certificado de assinatura + notarização (processo automatizável no mesmo workflow via
  `tauri-apps/tauri-action`, que já suporta isso via secrets `APPLE_CERTIFICATE`,
  `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`,
  `APPLE_TEAM_ID` — não configurado ainda, perguntar ao usuário se a NexTags já tem essa conta
  antes de implementar)

Sem isso, ainda é 100% usável — só exige aquele clique extra de "Abrir mesmo assim" na primeira
execução.

## Arquivos relevantes já no repo

- `.github/workflows/build.yml` — workflow de CI
- `src-tauri/src/lib.rs`, `tauri.conf.json` — mesma base do build Windows (janela única, menu de
  contexto com recarregar/limpar cache, ícone da marca)
- `app-icon-source.png` — fonte do ícone (não precisa regenerar)
- `HANDOFF-ANDROID.md` — handoff separado pra app Android (outra sessão, mesma base de projeto)

## Não fazer

- Não publicar o Release automaticamente sem o usuário confirmar (por isso `releaseDraft: true`
  no workflow) — quem decide tornar público é ele, manualmente, na interface do GitHub.
- Não commitar `dist-cliente/`, `src-tauri/target/`, `src-tauri/gen/` — já estão no `.gitignore`,
  são output/regenerável.
