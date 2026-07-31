# NexTags AI — App Desktop

Wrapper desktop (Tauri v2) do NexTags AI. Abre `https://app.nextagsai.com.br/` numa janela
nativa unica, com menu de contexto proprio (recarregar / limpar cache e recarregar).

Plataformas: macOS (Apple Silicon e Intel), Linux e Windows.

## Builds

Os binarios sao gerados pelo GitHub Actions, nao localmente — build nativo de macOS exige
toolchain da Apple. Veja [HANDOFF-MAC-BUILD.md](HANDOFF-MAC-BUILD.md) para como disparar o
build, onde baixar os artefatos e o que ainda esta pendente (assinatura Apple).

Para Android, veja [HANDOFF-ANDROID.md](HANDOFF-ANDROID.md).

## Desenvolvimento local

Requer [Rust](https://rustup.rs/) e Node 20+, mais as
[dependencias de sistema do Tauri](https://v2.tauri.app/start/prerequisites/) da sua plataforma.

```sh
npm ci
npm run tauri dev     # roda em modo dev
npm run tauri build   # gera o bundle da plataforma atual
```

Os alvos de bundle sao definidos por plataforma: `nsis` no Windows (`tauri.conf.json`),
`app`/`dmg` no macOS (`tauri.macos.conf.json`) e `deb`/`appimage` no Linux
(`tauri.linux.conf.json`).

## Estrutura

- `src-tauri/src/lib.rs` — criacao da janela, menu de contexto e comando `clear_cache`
- `src-tauri/tauri.conf.json` — config base (Windows) + branding do bundle
- `src-tauri/capabilities/default.json` — ACL; precisa listar a origem remota para o
  `invoke` funcionar a partir da pagina carregada
- `src/` — frontend local, nao usado em runtime (a janela aponta para a URL remota)
- `.github/workflows/build.yml` — CI multiplataforma
