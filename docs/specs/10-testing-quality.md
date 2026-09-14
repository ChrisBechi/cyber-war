# Spec 10 — Testing & Quality

## Gate

`pnpm check`

## Frontend

ESLint, Prettier, Stylelint, TypeScript strict, Vitest e Testing Library.

## Rust

rustfmt, Clippy com warnings como erro e cargo test.

## Prioridades de testes

1. Sandbox.
2. Save integrity.
3. Mission state.
4. VFS.
5. Terminal.
6. Virtual Network.
7. UI.

## Casos obrigatórios

- comando desconhecido nunca executa host;
- VFS nunca escapa;
- rede virtual não abre socket real;
- slots aceitam 1..5 e rejeitam demais;
- save/load round-trip;
- checkpoint rollback;
- corrupção;
- requirements/branching de missão.
