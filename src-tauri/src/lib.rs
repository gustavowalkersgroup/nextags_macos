use tauri::{WebviewUrl, WebviewWindowBuilder};

const APP_URL: &str = "https://app.nextagsai.com.br/";

// UA de Chrome desktop. Necessaria no Windows para o OAuth do Google nao bloquear a WebView2.
// Medido em 31/07/2026: app.nextagsai.com.br nao faz UA sniffing (o HTML servido e byte-identico
// para UA desktop/mobile/WebView) e o layout responsivo depende so da largura da viewport, entao
// manter esta UA no Android nao degrada o layout mobile.
const USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36";

// Menu compartilhado entre desktop e Android.
//
// A guarda de idempotencia e obrigatoria: no Android, com URL remota, o wry injeta os
// initialization scripts DUAS vezes (via WebViewCompat.addDocumentStartJavaScript e de novo em
// RustWebViewClient.onPageStarted, porque uma URL https nao e interceptada por custom protocol).
// Sem a guarda os listeners seriam registrados em duplicidade.
const MENU_JS: &str = r#"
(function () {
  if (window.__nextagsMenuInstalled) return;
  window.__nextagsMenuInstalled = true;

  function closeMenu() {
    var el = document.getElementById('__nextags_ctx_menu');
    if (el) el.remove();
  }

  function clearCacheAndReload() {
    var done = function () { window.location.reload(); };
    try {
      window.__TAURI__.core.invoke('clear_cache').then(done, function (err) {
        console.error('[nextags] clear_cache falhou:', err);
        done();
      });
    } catch (e) {
      console.error('[nextags] invoke indisponivel:', e);
      done();
    }
  }

  window.__nextagsOpenMenu = function (x, y) {
    closeMenu();

    var menu = document.createElement('div');
    menu.id = '__nextags_ctx_menu';
    menu.style.cssText = [
      'position:fixed',
      'top:' + y + 'px',
      'left:' + x + 'px',
      'background:#1e1e2e',
      'color:#f0f0f0',
      'border:1px solid #3a3a4a',
      'border-radius:8px',
      'box-shadow:0 4px 16px rgba(0,0,0,0.4)',
      'padding:4px',
      'z-index:2147483647',
      'font-family:system-ui,sans-serif',
      'font-size:15px',
      'min-width:220px'
    ].join(';');

    var items = [
      ['Recarregar pagina', function () { window.location.reload(); }],
      ['Limpar cache e recarregar', clearCacheAndReload]
    ];

    items.forEach(function (item) {
      var btn = document.createElement('div');
      btn.textContent = item[0];
      btn.style.cssText = 'padding:12px;cursor:pointer;border-radius:5px;white-space:nowrap;';
      btn.addEventListener('mouseenter', function () { btn.style.background = '#3a3a5a'; });
      btn.addEventListener('mouseleave', function () { btn.style.background = 'transparent'; });
      btn.addEventListener('click', function (ev) {
        ev.stopPropagation();
        closeMenu();
        item[1]();
      });
      menu.appendChild(btn);
    });

    document.body.appendChild(menu);

    var rect = menu.getBoundingClientRect();
    if (rect.right > window.innerWidth) menu.style.left = (window.innerWidth - rect.width - 8) + 'px';
    if (rect.bottom > window.innerHeight) menu.style.top = (window.innerHeight - rect.height - 8) + 'px';

    setTimeout(function () {
      window.addEventListener('click', closeMenu, { once: true });
      window.addEventListener('touchstart', closeMenu, { once: true, passive: true });
    }, 0);
  };
})();
"#;

#[cfg(desktop)]
const GESTURES_JS: &str = r#"
(function () {
  if (window.__nextagsGesturesInstalled) return;
  window.__nextagsGesturesInstalled = true;

  window.addEventListener('contextmenu', function (e) {
    e.preventDefault();
    window.__nextagsOpenMenu(e.clientX, e.clientY);
  });
})();
"#;

// Android nao tem botao direito. Long-press abre o mesmo menu (recarregar + limpar cache) e
// pull-to-refresh recarrega. Sem FAB: o canto inferior direito ja esta ocupado pelo botao do
// widget de suporte do proprio site (button.ktt10-btn, 50x50, position:fixed).
//
// Todos os listeners sao passive, ou seja nao chamam preventDefault, entao o scroll nativo do
// site nunca e bloqueado. O pull-to-refresh so arma quando o container rolavel sob o dedo E a
// pagina estao no topo, para nao brigar com as listas de scroll proprio do site (a lista de
// conversas do inbox tem overflowY proprio e ~4972px de altura).
#[cfg(target_os = "android")]
const GESTURES_JS: &str = r#"
(function () {
  if (window.__nextagsGesturesInstalled) return;
  window.__nextagsGesturesInstalled = true;

  var LONG_PRESS_MS = 550;
  var MOVE_TOLERANCE = 10;
  var PTR_TRIGGER = 90;

  var lpTimer = null;
  var startX = 0, startY = 0;
  var ptrArmed = false, ptrWillRefresh = false, ptrTarget = null;
  var indicator = null;

  function cancelLongPress() {
    if (lpTimer !== null) { clearTimeout(lpTimer); lpTimer = null; }
  }

  function isEditable(el) {
    return !!(el && el.closest && el.closest('input,textarea,select,[contenteditable=""],[contenteditable="true"]'));
  }

  function scrollableAncestor(el) {
    for (var n = el; n && n !== document.body; n = n.parentElement) {
      var oy = getComputedStyle(n).overflowY;
      if ((oy === 'auto' || oy === 'scroll') && n.scrollHeight > n.clientHeight + 1) return n;
    }
    return null;
  }

  function ensureIndicator() {
    if (indicator) return indicator;
    indicator = document.createElement('div');
    indicator.id = '__nextags_ptr';
    indicator.style.cssText = [
      'position:fixed', 'top:0', 'left:0', 'width:100%',
      'padding:10px 0', 'text-align:center',
      'background:#1e1e2e', 'color:#f0f0f0',
      'font-family:system-ui,sans-serif', 'font-size:13px',
      'z-index:2147483646', 'pointer-events:none',
      'transform:translateY(-100%)', 'transition:transform 0.12s linear'
    ].join(';');
    document.body.appendChild(indicator);
    return indicator;
  }

  function showIndicator(progress, ready) {
    var el = ensureIndicator();
    el.textContent = ready ? 'Solte para recarregar' : 'Arraste para recarregar';
    el.style.transform = 'translateY(' + (progress * 100 - 100) + '%)';
  }

  function hideIndicator() {
    if (indicator) indicator.style.transform = 'translateY(-100%)';
  }

  document.addEventListener('touchstart', function (e) {
    if (e.touches.length !== 1) { cancelLongPress(); ptrArmed = false; return; }
    var t = e.touches[0];
    startX = t.clientX; startY = t.clientY;

    cancelLongPress();
    if (!isEditable(e.target)) {
      lpTimer = setTimeout(function () {
        lpTimer = null;
        var sel = window.getSelection();
        if (sel && String(sel).length > 0) return; // nao atropela a selecao de texto do sistema
        ptrArmed = false;
        hideIndicator();
        window.__nextagsOpenMenu(startX, startY);
      }, LONG_PRESS_MS);
    }

    ptrTarget = scrollableAncestor(e.target);
    var innerAtTop = !ptrTarget || ptrTarget.scrollTop <= 0;
    var docAtTop = (window.scrollY || document.documentElement.scrollTop || 0) <= 0;
    ptrArmed = innerAtTop && docAtTop;
    ptrWillRefresh = false;
  }, { passive: true });

  document.addEventListener('touchmove', function (e) {
    if (e.touches.length !== 1) { cancelLongPress(); ptrArmed = false; return; }
    var t = e.touches[0];
    var dx = t.clientX - startX;
    var dy = t.clientY - startY;

    if (Math.abs(dx) > MOVE_TOLERANCE || Math.abs(dy) > MOVE_TOLERANCE) cancelLongPress();

    if (!ptrArmed) return;
    if (dy <= 0 || Math.abs(dx) > Math.abs(dy)) {
      ptrArmed = false;
      ptrWillRefresh = false;
      hideIndicator();
      return;
    }
    var progress = Math.min(dy / PTR_TRIGGER, 1);
    ptrWillRefresh = progress >= 1;
    showIndicator(progress, ptrWillRefresh);
  }, { passive: true });

  function endTouch() {
    cancelLongPress();
    if (ptrArmed && ptrWillRefresh) {
      showIndicator(1, true);
      window.location.reload();
      return;
    }
    ptrArmed = false;
    ptrWillRefresh = false;
    hideIndicator();
  }

  document.addEventListener('touchend', endTouch, { passive: true });
  document.addEventListener('touchcancel', endTouch, { passive: true });
  document.addEventListener('scroll', function () { cancelLongPress(); }, { passive: true, capture: true });
})();
"#;

#[tauri::command]
async fn clear_cache(window: tauri::WebviewWindow) -> Result<(), String> {
    window.clear_all_browsing_data().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![clear_cache])
        .setup(|app| {
            let script = format!("{MENU_JS}\n{GESTURES_JS}");

            #[allow(unused_mut)]
            let mut builder =
                WebviewWindowBuilder::new(app, "main", WebviewUrl::External(APP_URL.parse()?))
                    .title("NexTags AI")
                    .user_agent(USER_AGENT)
                    .initialization_script(script.as_str());

            // No Android tao ignora silenciosamente estes atributos
            // (`// FIXME this ignores requested window attributes`), entao ficam so no desktop.
            #[cfg(desktop)]
            {
                builder = builder
                    .inner_size(1280.0, 800.0)
                    .min_inner_size(900.0, 600.0);
            }

            if let Err(e) = builder.build() {
                // Deixa a falha visivel no logcat em vez de virar um panic opaco.
                eprintln!("[nextags] falha ao criar a webview principal: {e}");
                return Err(e.into());
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
