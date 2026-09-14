# Spec 01 — Architecture

## Atualização implementada — 0.4.2

`GameService` mantém a fronteira transacional do mundo virtual. `MissionRuntime` contém recursos/efeitos por tentativa, sem baseline integral do mundo. O motor admite uma única missão ativa. `terminal_sessions` mantém contextos runtime por janela e adapta o contexto selecionado aos comandos existentes sob mutex; VFS e rede permanecem compartilhados. As configurações gerais ainda usam o mapa anterior e o VFS ainda é textual. Veja [o checkpoint e as pendências](../RELATORIO-CORRECOES-0.4.2.md).

```text
┌────────────────────────────────────────────┐
│                LIFEOS UI                   │
│ React / TypeScript / xterm / future Monaco │
└────────────────────┬───────────────────────┘
                     │ Tauri IPC
┌────────────────────▼───────────────────────┐
│            APPLICATION SERVICES            │
│ Commands / Save / Mission / Event APIs     │
└────────────────────┬───────────────────────┘
                     │
┌────────────────────▼───────────────────────┐
│             SIMULATION CORE                │
│ Rust                                       │
│ VirtualFileSystem                          │
│ CommandEngine                              │
│ VirtualNetwork                             │
│ VirtualProcessManager                      │
│ VirtualUsers                               │
│ MissionEngine                              │
│ EventEngine                                │
└────────────────────┬───────────────────────┘
                     │
┌────────────────────▼───────────────────────┐
│                 SQLITE                     │
└────────────────────────────────────────────┘
```

## Frontend

Responsável por apresentação, janelas, input, animações, terminal visual e menus.

Não é autoridade para dinheiro, missão concluída, permissões, hosts, credenciais ou resultado de ataques.

## Rust

Fonte de verdade para filesystem, comandos, rede, processos, usuários/privilégios, missões, flags, recompensas e saves.

## Comunicação

Usar comandos Tauri tipados. Não criar servidor HTTP localhost para o core.
