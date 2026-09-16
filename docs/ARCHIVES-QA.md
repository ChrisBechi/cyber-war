# Archive e compressão — validação

Data: 15 de setembro de 2026. Windows x64, Rust 1.98.1 GNU, LLVM-MinGW,
Node 24.19.0, Tauri 2 e WebView2. Decisões, bibliotecas e limites:
[ADR 0002](adr/0002-vfs-archives.md).

## Verificações executadas

| Verificação                                        | Resultado                                                    |
| -------------------------------------------------- | ------------------------------------------------------------ |
| Rust, testes unitários e documentação              | 112 testes passaram; zero falhas                             |
| Clippy, todos os targets e features, `-D warnings` | Passou                                                       |
| Rustfmt                                            | Passou                                                       |
| Frontend, suíte completa                           | 170 testes passaram, 30 arquivos                             |
| Frontend, após ajustes finais de navegação         | 34 testes passaram: Browser, ArchiveViewer e Desktop.context |
| TypeScript, ESLint e Stylelint                     | Passaram                                                     |
| Catálogo e conteúdo                                | 12 missões, 11 tópicos, 8 hosts e 303 entradas Kali válidos  |
| Vite, produção                                     | Passou                                                       |
| Windows, executável de desenvolvimento             | Compilado e executado                                        |
| E2E no Tauri                                       | Oito fluxos passaram com IPC real                            |

O teste nativo reproduz criação, leitura e extração no VFS compartilhado entre
terminal, gerenciador de arquivos e anexos. Confere senha incorreta/correta,
limpeza do campo de senha, corrupção, progresso e cancelamento sem saída parcial.
Também compara os bytes extraídos pela interface e pelo terminal, executa
redirecionamento binário, `zcat | grep` e testa a tela de endereço inexistente.

Os testes Rust cobrem os oito formatos, permissões, arquivos ocultos, links virtuais,
archives aninhados, disco cheio, tamanhos esparsos, persistência SQLite, paths
maliciosos, entradas duplicadas, contagens ZIP excessivas, truncamento e limites de
expansão. ZIP protegido pode pedir senha antes de iniciar um job grande; cancelar
esse job preserva o VFS. O snapshot não contém a senha.

## Reproduzir o teste nativo

No ambiente com Node 22+, pnpm, Rust e compilador Windows configurados:

1. Execute `pnpm dev:web --host 127.0.0.1`.
2. Compile com `cargo build --manifest-path src-tauri/Cargo.toml`.
3. Inicie o executável de desenvolvimento com as variáveis abaixo.
4. Execute `node scripts/qa-archives.mjs` em outro terminal.

```powershell
$env:CYBER_WAR_ARCHIVE_QA = '1'
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'
Start-Process -FilePath .\src-tauri\target\debug\game-hacker.exe -WindowStyle Hidden
```

A fixture existe somente no build debug, usa SQLite em memória e verifica o nome
`archive-qa` antes do teste. Reinicie o aplicativo para repetir toda a sequência.
Os arquivos `artifacts/archive-qa/results.json` e as capturas PNG são gerados pelo
script e ignorados pelo controle de versão. O teste usa CDP no próprio WebView2;
não substitui o transporte Tauri por mocks.

## Interface e navegador

A página de servidor não encontrado usa a ilustração original
`no-connection.svg` do Firefox, distribuída localmente com MPL-2.0. Exibe o hostname
da aba, repete o endereço preservando seu histórico e oferece ajuda dentro da
página. O erro de navegação não gera um segundo aviso global no desktop.

Capturas do navegador cobrem 1920×1080, 1024×640 e uma janela de 560 pixels. O layout
passa de duas colunas para uma coluna com rolagem. O script também captura File
Manager, Archive Viewer, propriedades, compactação, extração, senha, progresso e
erros em 1024×640, 1280×800, 1440×900 e 1920×1080.

O navegador também recebeu ferramentas de inspeção. Capacidades, atalhos e
verificações estão descritos em [BROWSER-QA.md](BROWSER-QA.md).

Foi corrigida uma seleção obsoleta ao trocar rapidamente de pasta: a lista e a
seleção anteriores são limpas enquanto a nova listagem é carregada.

## Limites da verificação

- O check global de Prettier encontra 23 arquivos anteriores de Vigília fora do
  padrão. Os arquivos alterados nesta entrega são formatados separadamente.
- Vite ainda relata o tamanho do chunk principal acima de 500 kB e comentários
  de pureza em Zod removidos pelo Rollup. O build termina normalmente.
- A captura do plugin Windows falhou com `SetIsBorderRequired`/`0x80004002`;
  a inspeção visual foi feita com capturas CDP do WebView2 nativo.
- A compilação MSVC, instaladores MSI/NSIS e assinatura de distribuição não foram
  validados. O ambiente usado é Windows GNU com LLVM-MinGW.
- Tempos de codec e picos de memória não foram medidos por um benchmark dedicado.
  As cotas são verificadas pelos testes; a duração exibida pelos jobs é virtual.
- ZIP64, volumes divididos, hardlinks e nós especiais são rejeitados. Pipes de
  archives suportam saída textual; streams comprimidos usam redirecionamento.
- Arquivos esparsos preservam seus tamanhos virtuais em metadados; ferramentas
  externas recebem o conteúdo materializado e o envelope, não gigabytes fictícios.
