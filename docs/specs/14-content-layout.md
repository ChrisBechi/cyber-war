# Spec 14 — Content Layout

Conteúdo narrativo deve ficar separado de código quando possível.

```text
content/
├── missions/
│   ├── session_1/
│   ├── session_2/
│   └── session_3/
├── dialogues/
├── forums/
├── messages/
├── networks/
├── hosts/
├── devices/
└── apps/
```

Usar JSON/TOML/YAML apenas com schema validado. Recomendação inicial: JSON + Zod no tooling e Serde no core.

Nunca inserir uma missão inteira dentro de componente React.
