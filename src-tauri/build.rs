fn main() {
    // Declara o comando no app manifest, o que gera a permissao `allow-clear-cache`.
    // Sem isso o invoke('clear_cache') vindo da pagina remota e rejeitado pelo ACL
    // desde o tauri 2.11.1 (IPC de origem nao-local exige capability com remote.urls).
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["clear_cache"])),
    )
    .expect("failed to run tauri-build");
}
