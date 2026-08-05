fn main() {
    // Registra `clear_cache` no ACL, gerando a permissao `allow-clear-cache`.
    // Sem isso o comando nao existe na ACL resolvida e o invoke vindo da pagina
    // remota e recusado em tempo de execucao.
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(&["clear_cache"])),
    )
    .expect("failed to run tauri-build");
}
