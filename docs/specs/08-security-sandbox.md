# Spec 08 — Security Sandbox

## Objetivo

Garantir que as mecânicas hackers permaneçam dentro do jogo.

## Proibido ao CommandEngine

- processos reais;
- filesystem real genérico;
- sockets reais genéricos;
- shell escape;
- WSL;
- SSH real;
- navegação web irrestrita de ferramentas de missão.

## Allowlist

```text
CommandRegistry
├── ls
├── cat
├── ping_virtual
├── nmap_virtual
└── ...
```

Comando desconhecido retorna `command not found`. Nunca há fallback para o host.

## Host

Tauri só acessa o necessário para app data, saves, settings, screenshots/export explícito e logs.

A importação de arquivos é uma ação explícita do usuário por seletor no frontend. Somente os bytes do arquivo escolhido são enviados ao VFS, com limites e permissões; o IPC não recebe um caminho real para abrir no Windows. Blobs não são executáveis do host, e extensão `.sh` em um blob não o transforma em código. Players criam URLs Blob transitórias, sem fontes externas; HTML/SVG binários não são publicados como documentos ativos. O MIME declarado não certifica o conteúdo: a decodificação continua responsabilidade do WebView2.

## Network

Jogo base offline. Cloud save, update ou telemetry futuros ficam separados da VirtualNetwork e nunca são invocáveis pelo terminal do jogo.
