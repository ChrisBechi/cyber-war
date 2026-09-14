# Spec 12 — Internal API Contracts

"API" significa principalmente **Tauri IPC interno**, não servidor web.

## Categorias

### System

`system_health`, futuro `get_build_info`.

### Terminal

`terminal_open({ sessionId, cwd?, asRoot? })`, `terminal_close({ sessionId })`, `execute_terminal({ sessionId, command })`, `terminal_complete({ sessionId, line, cursor })`, `nano_write`, `nano_read`, `nano_close`.

Resultados de comandos mantêm stdout, stderr e exitCode independentes. Não rejeitar um resultado válido só porque seu exitCode é não zero. Leitores podem produzir saída parcial e diagnóstico juntos; o contrato detalhado está em [comandos auditados](../CONTRATOS-COMANDOS-0.4.2.md).

Nano recebe o mesmo `sessionId` da janela. O identificador omitido mantém o contexto legado dos instrumentos internos; janelas de terminal sempre enviam ID. Abertura/fechamento de contexto são runtime, sem avaliação de missão. `end_session` retorna null após normalizar e salvar; `session_start` retorna o mundo após reconstruir o runtime. As transições da GUI aguardam também a atualização de `world_get`/`mission_get_state` antes de navegar.

`terminal_complete({ line, cursor })` retorna `{ line, cursor, candidates }`. O cursor é um índice de caracteres Unicode (não bytes nem unidades UTF-16). A consulta usa o VFS e a sessão atual, sem executar comandos nem alterar o mundo. O frontend descarta respostas quando a linha foi editada enquanto a consulta estava em andamento. Interrupção de comandos assíncronos permanece futura.

### Saves

- `list_save_slots`
- `save_slot`
- `load_slot`
- `create_mission_checkpoint`
- `list_checkpoints`
- `restore_checkpoint`

### VFS futuro

`vfs_list`, `vfs_read`, `vfs_write`, `vfs_create_directory`, `vfs_move`, `vfs_remove`, `vfs_stat`.

Binários implementados: `vfs_import_bytes({ path, base64, mime, expectedModified?, asRoot? })` retorna null após commit; expectedModified ausente/null significa criar exclusivamente, e substituição exige versão coincidente. `vfs_read_bytes({ path, asRoot? })` retorna `{ base64, mime, size }` após validar leitura/lixeira. Paths são do VFS local; asRoot é somente ator virtual. Limite de bytes: 32 MiB/arquivo. Base64 é transporte IPC, nunca conteúdo de snapshot. `vfs_stat`/list podem incluir `blob: { hash, size, mime }`; texto antigo permanece compatível. Veja a spec 04 e os testes em binary_tests.rs.

### Mission futuro

`mission_list_available`, `mission_start`, `mission_abort_attempt`, `mission_get_state`, `mission_choose`.

## Regra

Não criar endpoint HTTP local só para conectar React e Rust.
