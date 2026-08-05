# NexTags AI — App Desktop

App do NexTags AI em janela própria, sem barra de navegador. Abre
`https://app.nextagsai.com.br/` direto, com menu de contexto próprio (botão direito → recarregar /
limpar cache e recarregar).

Disponível para **Windows**, **macOS** (Apple Silicon e Intel), **Android** e **Linux**.

## Baixar

**→ [Baixar a versão mais recente](https://github.com/gustavowalkersgroup/nextags_macos/releases/latest)**

| Seu aparelho | Arquivo | Observação |
|---|---|---|
| Windows | `..._x64-setup.exe` | Instalador — o caminho normal |
| Windows | `..._windows_x64.exe` | Portable: não instala nada, abre direto. Bom para pendrive |
| Mac com chip Apple (M1/M2/M3/M4) | `..._aarch64.dmg` | Veja o passo extra abaixo |
| Mac com Intel | `..._x64.dmg` | Veja o passo extra abaixo |
| Android | `..._android-debug.apk` | Versão de teste — veja abaixo |
| Linux | `.deb` ou `.AppImage` | `.deb` para Ubuntu/Debian; `.AppImage` roda em qualquer distro |

Na dúvida sobre qual Mac você tem: menu Apple → **Sobre este Mac**. Se aparecer "Chip Apple M…",
é o `aarch64`. Se aparecer "Processador Intel", é o `x64`. Baixar o errado não causa problema —
simplesmente não abre.

### macOS: um passo extra na primeira vez

O app ainda não tem assinatura da Apple, então o macOS bloqueia na primeira abertura. É esperado e
seguro de contornar:

1. Dê duplo clique no app. Vai aparecer "não foi possível verificar o desenvolvedor". Feche.
2. Abra **Ajustes do Sistema → Privacidade e Segurança** e role até o fim.
3. Clique em **"Abrir Mesmo Assim"** ao lado da mensagem sobre o NexTags AI.
4. Confirme com a senha de administrador.

Só na primeira vez. Duas coisas que economizam ligação para o suporte:

- O antigo "botão direito → Abrir" **não funciona mais** — a Apple removeu esse atalho no macOS 15.
- No macOS 26, o botão "Abrir Mesmo Assim" só aparece por cerca de **1 hora** depois da tentativa
  bloqueada. Se demorar, tente abrir o app de novo para ele reaparecer.

### Android: é versão de teste

O APK disponível é assinado em modo debug. **Instala e funciona**, mas é para validação, não para
produção — falta a assinatura de release. Para instalar, o Android vai pedir para permitir
"instalar apps de fontes desconhecidas".

Detalhes e o que falta para virar versão de produção: [HANDOFF-ANDROID.md](HANDOFF-ANDROID.md).

## Não quer instalar nada? Use como PWA

Dá para instalar o NexTags AI direto do navegador, sem baixar arquivo nenhum e sem nenhum aviso de
segurança. Funciona em iPhone, Mac, Windows e Android:

- **iPhone / iPad:** abra o site no Safari → botão Compartilhar → **Adicionar à Tela de Início**
- **Mac:** abra o site no Safari → menu **Arquivo → Adicionar ao Dock**
- **Windows / Android:** Chrome ou Edge → ícone de instalar na barra de endereço

O resultado é praticamente o mesmo: ícone próprio, janela sem barra de navegador. É a única opção
para iPhone, porque a App Store não aceita apps que só abrem um site.

Passo a passo completo e o que o site precisa servir para ficar com ícone e nome certos:
[PWA-SAFARI.md](PWA-SAFARI.md).

---

## Para desenvolvedores

Os binários são gerados pelo GitHub Actions, não localmente — build nativo de macOS exige toolchain
da Apple, que só roda em Mac. O workflow compila as cinco plataformas em runners hospedados.

- [HANDOFF-MAC-BUILD.md](HANDOFF-MAC-BUILD.md) — como disparar o build, onde ficam os artefatos,
  e o estado da assinatura Apple
- [HANDOFF-ANDROID.md](HANDOFF-ANDROID.md) — build Android e o que falta para produção

### Rodar local

Requer [Rust](https://rustup.rs/) e Node 20+, mais as
[dependências de sistema do Tauri](https://v2.tauri.app/start/prerequisites/) da sua plataforma.

```sh
npm ci
npm run tauri dev     # modo dev
npm run tauri build   # bundle da plataforma atual
```

### Config por plataforma

Os alvos de bundle e o nome do executável variam por plataforma:

| Plataforma | Bundle | Executável | Onde |
|---|---|---|---|
| Windows | `nsis` | `NexTags AI` | `tauri.conf.json` |
| macOS | `app`, `dmg` | `NexTags AI` | `tauri.macos.conf.json` |
| Linux | `deb`, `appimage` | `nextags-ai` | `tauri.linux.conf.json` |

O Tauri lê e mescla o arquivo da plataforma atual sobre o base automaticamente (arrays substituem,
não concatenam). O nome hifenizado no Linux não é capricho: o `.desktop` gerado usa `Exec=` sem
aspas, e um espaço no nome quebraria o atalho.

Android sai de um job separado no CI, porque o toolchain (JDK, SDK, NDK) não tem relação com o
build desktop. O `src-tauri/gen/` é gerado, não versionado, então o `android init` roda a cada build.

### Estrutura

- `src-tauri/src/lib.rs` — criação da janela, menu de contexto e comando `clear_cache`
- `src-tauri/capabilities/default.json` — ACL; precisa declarar a origem remota, senão o `invoke`
  vindo da página carregada é recusado
- `src/` — frontend local, não usado em runtime (a janela aponta para a URL remota)
- `pwa/` — `manifest.json` e ícones para servir no site
- `.github/workflows/build.yml` — CI das cinco plataformas
