use tauri::{WebviewUrl, WebviewWindowBuilder};

const APP_URL: &str = "https://app.nextagsai.com.br/";

const CONTEXT_MENU_JS: &str = r#"
(function () {
  function closeMenu() {
    const el = document.getElementById('__nextags_ctx_menu');
    if (el) el.remove();
  }

  window.addEventListener('contextmenu', function (e) {
    e.preventDefault();
    closeMenu();

    const menu = document.createElement('div');
    menu.id = '__nextags_ctx_menu';
    menu.style.cssText = [
      'position:fixed',
      'top:' + e.clientY + 'px',
      'left:' + e.clientX + 'px',
      'background:#1e1e2e',
      'color:#f0f0f0',
      'border:1px solid #3a3a4a',
      'border-radius:8px',
      'box-shadow:0 4px 16px rgba(0,0,0,0.4)',
      'padding:4px',
      'z-index:2147483647',
      'font-family:system-ui,sans-serif',
      'font-size:13px',
      'min-width:200px'
    ].join(';');

    const items = [
      ['Recarregar pagina', function () { window.location.reload(); }],
      ['Limpar cache e recarregar', function () {
        window.__TAURI__.core.invoke('clear_cache').finally(function () {
          window.location.reload();
        });
      }]
    ];

    items.forEach(function (item) {
      const btn = document.createElement('div');
      btn.textContent = item[0];
      btn.style.cssText = 'padding:8px 12px;cursor:pointer;border-radius:5px;white-space:nowrap;';
      btn.addEventListener('mouseenter', function () { btn.style.background = '#3a3a5a'; });
      btn.addEventListener('mouseleave', function () { btn.style.background = 'transparent'; });
      btn.addEventListener('click', function () { closeMenu(); item[1](); });
      menu.appendChild(btn);
    });

    document.body.appendChild(menu);

    const rect = menu.getBoundingClientRect();
    if (rect.right > window.innerWidth) menu.style.left = (window.innerWidth - rect.width - 8) + 'px';
    if (rect.bottom > window.innerHeight) menu.style.top = (window.innerHeight - rect.height - 8) + 'px';

    setTimeout(function () {
      window.addEventListener('click', closeMenu, { once: true });
    }, 0);
  });
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
            WebviewWindowBuilder::new(app, "main", WebviewUrl::External(APP_URL.parse().unwrap()))
                .title("NexTags AI")
                .inner_size(1280.0, 800.0)
                .min_inner_size(900.0, 600.0)
                .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36")
                .initialization_script(CONTEXT_MENU_JS)
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
