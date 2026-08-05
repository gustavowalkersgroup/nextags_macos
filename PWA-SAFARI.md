# PWA — instalar o NexTags AI sem app

Rota alternativa ao binário: o usuário instala o `app.nextagsai.com.br` como aplicativo a partir
do próprio navegador. **Sem Gatekeeper, sem review da App Store, sem certificado, sem custo.**

Vale para macOS, iPhone, iPad, Windows e Android — o mesmo caminho resolve todas as plataformas de
uma vez, que é justamente o que o wrapper nativo não faz.

## Antes de tudo: um teste que decide se isso é viável

O wrapper Tauri forja user-agent de Chrome no Windows (`src-tauri/src/lib.rs`). Isso levanta a
suspeita de que o `app.nextagsai.com.br` talvez não funcione bem no Safari — se o site depender de
API só do Chromium, ou bloquear por user-agent, a rota do PWA morre no Safari.

**Abra `https://app.nextagsai.com.br/` no Safari de um Mac ou iPhone e faça um login completo.**
Leva 30 segundos e decide o resto.

Esse teste **não foi feito** por aqui: o egress desta sessão bloqueia o domínio com 403 de política
de rede, não por problema do site. É a única peça que falta para essa rota estar confirmada.

## Parte 1 — o que o usuário final faz

Nada para instalar, nada para baixar.

### iPhone / iPad

1. Abrir `https://app.nextagsai.com.br/` **no Safari** (precisa ser o Safari).
2. Botão de **Compartilhar** (o quadrado com a seta para cima, na barra de baixo).
3. Rolar e escolher **"Adicionar à Tela de Início"**.
4. Confirmar o nome e tocar em **Adicionar**.

O ícone aparece na tela de início como qualquer app. Abre em janela própria, sem a barra do
navegador.

### macOS

Requer macOS 14 Sonoma ou mais novo.

1. Abrir `https://app.nextagsai.com.br/` **no Safari**.
2. Menu **Arquivo → Adicionar ao Dock**.
3. Confirmar o nome e clicar em **Adicionar**.

Vira um app em `~/Applications`, com ícone próprio no Dock, janela sem barra de navegação e Spotlight
encontrando por nome.

No Chrome ou Edge o equivalente é o ícone de instalar na barra de endereço, ou
**⋮ → Transmitir, salvar e compartilhar → Instalar página como aplicativo**.

### Windows e Android

Chrome ou Edge, mesmo caminho: ícone de instalar na barra de endereço. No Android, Chrome →
**⋮ → Adicionar à tela inicial**.

## Parte 2 — o que o site precisa servir

O passo a passo acima já funciona sem nenhuma mudança no site, mas o resultado fica feio: nome
truncado, ícone genérico (um print da página), e no iOS o ícone sai composto sobre preto.

Para ficar apresentável, o time que mantém o `app.nextagsai.com.br` precisa de duas coisas.

### 1. Publicar os arquivos deste diretório

O diretório `pwa/` deste repositório já traz tudo pronto, gerado a partir do
`app-icon-source.png` (a mesma fonte dos ícones do app nativo):

| Arquivo | Para quê |
|---|---|
| `manifest.json` | Nome, cores e ícones do app instalado |
| `icon-192.png` | Ícone padrão |
| `icon-512.png` | Ícone grande e splash screen |
| `icon-512-maskable.png` | Android recorta o ícone em formas variadas; este tem o logo na zona segura e fundo opaco |
| `apple-touch-icon.png` | Ícone do iOS |

Servir em `https://app.nextagsai.com.br/pwa/`. Se for outro caminho, ajustar os `src` dentro do
`manifest.json`.

Dois detalhes que não são óbvios e estão já resolvidos nos arquivos:

- **O iOS ignora transparência** no `apple-touch-icon` e compõe o que sobra sobre preto. Por isso
  esse arquivo específico foi achatado em fundo branco, enquanto os outros mantêm o alpha.
- **O ícone maskable** tem fundo opaco e o logo em 80% do quadro. Sem isso, o recorte do Android
  corta o logo ou mostra transparência como preto.

O `theme_color` (`#4050F0`) foi amostrado do próprio ícone da marca — é o azul do hexágono. Se o
time de marca tiver o valor oficial, sobrescrever.

### 2. Adicionar ao `<head>` de todas as páginas

```html
<link rel="manifest" href="/pwa/manifest.json" />
<link rel="apple-touch-icon" href="/pwa/apple-touch-icon.png" />
<meta name="theme-color" content="#4050F0" />

<!-- Legado do iOS: o Safari só passou a ler o manifest no iOS 16.4.
     Manter garante janela sem barra do navegador em iPhone antigo. -->
<meta name="apple-mobile-web-app-capable" content="yes" />
<meta name="apple-mobile-web-app-status-bar-style" content="default" />
<meta name="apple-mobile-web-app-title" content="NexTags AI" />
```

HTTPS é obrigatório para PWA — o site já tem.

## O que se perde em relação ao app nativo

Honestamente, para este caso: pouco.

- **O menu de contexto com "Limpar cache e recarregar"** deixa de existir. É a única função que o
  wrapper tem além de abrir a URL.
- **Controle do user-agent.** O wrapper força um UA de Chrome; num PWA o site vê o navegador real.
  Isso pode ser bom ou ruim — ver o teste no topo deste arquivo.
- **Percepção de produto.** Se o valor de entregar um `.dmg`/`.exe` para o cliente é comercial e não
  técnico, o PWA não substitui.

O que **não** se perde, porque o app nunca usou: system tray, deep links, acesso a arquivos,
atualizador automático, notificações nativas. Confirmável no `src-tauri/tauri.conf.json` —
`windows: []`, nenhum plugin, nenhum updater.

Notificações web funcionam em PWA instalado no iOS 16.4+ e no macOS, mas exigem o usuário aceitar a
permissão, e são implementadas no lado do site.

## Por que isso importa mais no iPhone

No macOS, se a Apple não gostar do app, dá para distribuir por fora. **No iPhone não existe "por
fora"** — ou passa na review da App Store, ou não chega no aparelho. E um wrapper de webview é o caso
clássico de rejeição pela diretriz 4.2 ("your app should include features, content, and UI that
elevate it beyond a repackaged website").

Para iPhone, o PWA não é o plano B. É o plano que funciona.
