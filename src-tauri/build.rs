fn main() {
    // Declarar o comando no app manifest e o que gera a permissao `allow-clear-cache`, usada pela
    // capability `remote-app`. Sem isso, `invoke('clear_cache')` a partir da origem remota e
    // rejeitado pelo ACL desde o tauri 2.11.1.
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["clear_cache"])),
    )
    .expect("failed to run tauri-build");
}
