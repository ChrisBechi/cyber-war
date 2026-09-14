# Spec 09 — Windows Packaging

Checkpoint 0.4.2: artefatos de distribuição anteriores não validam o código atual. Novos instaladores e teste em Windows limpo continuam pendentes durante a consolidação. Não distribuir esta árvore como versão consolidada sem fechar os gates e QA listados no [relatório](../RELATORIO-CORRECOES-0.4.2.md).

## Experiência final

```text
GameHacker-Setup.exe
→ instalar
→ atalho
→ Game Hacker
→ intro/menu
```

Jogador não instala Node, Rust, API nem servidor.

## Bundles

- NSIS;
- MSI.

NSIS é o principal alvo inicial.

## Build

```bash
pnpm install
pnpm check
pnpm tauri build
```

## Pré-requisitos de desenvolvimento no Windows

- Microsoft C++ Build Tools;
- WebView2;
- Rust;
- Node LTS;
- pnpm.

## Code signing

Antes de lançamento amplo, adquirir certificado, guardar em secret store/CI e assinar executável e instalador.

Nunca versionar certificado.

## Saves

Atualização e uninstall não devem apagar saves silenciosamente.
