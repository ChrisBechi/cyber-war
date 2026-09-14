# ADR 0001 — Tauri + React + Rust

## Status

Accepted.

## Decisão

Usar Tauri 2 + React/TypeScript + Rust + SQLite.

## Motivos

O jogo é predominantemente uma interface de desktop simulada, com terminal, janelas, arquivos, chats e ferramentas. HTML/CSS/React aceleram a UI; Rust concentra o mundo simulado e a sandbox; Tauri entrega um aplicativo desktop convencional sem backend HTTP separado.
