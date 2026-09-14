# Spec 13 — Project Conventions

## Idioma

Código em inglês. Conteúdo e documentação de produto podem ser em português.

## TypeScript

- strict;
- evitar `any`;
- componentes pequenos;
- gameplay autoritativo no Rust;
- Zod em bordas de conteúdo/data-driven.

## Rust

- erros tipados;
- evitar `unwrap()` em runtime;
- nenhuma execução arbitrária;
- serialização de save considerada em toda feature de gameplay.

## Commits

```text
feat(terminal): add virtual ls command
fix(save): preserve checkpoint order
docs(spec): define virtual network
```

## Definition of Done

Spec atualizada, testes, lint, typecheck, fmt/clippy, sandbox preservada e migration considerada.
