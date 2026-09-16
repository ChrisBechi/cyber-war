# Navegador: console e ferramentas de inspeção

## Recursos e atalhos

| Recurso            | Acesso                    | Comportamento                                                                     |
| ------------------ | ------------------------- | --------------------------------------------------------------------------------- |
| Histórico          | Menu ou Ctrl+H            | Últimas 50 páginas distintas da sessão, com limpeza.                              |
| Voltar / avançar   | Botões ou Alt+← / Alt+→   | Navegação independente por aba.                                                   |
| Recarregar         | Botão, F5 ou Ctrl+R       | Recarrega sem acrescentar uma posição à navegação.                                |
| Favoritos          | Estrela ou Ctrl+D         | Adição, edição e remoção persistidas no save.                                     |
| Modo desenvolvedor | Menu, F12 ou Ctrl+Shift+I | Painel acoplado que permanece aberto durante a navegação.                         |
| Código fonte       | Menu ou Ctrl+U            | HTML da página exibida, atualizado durante a navegação, em campo somente leitura. |

## Painéis

- **Console:** eventos de navegação e comandos de consulta ao HTML da aba atual.
  Exemplos: `document.title`, `location.href`,
  `document.querySelector("h1").textContent`,
  `document.querySelectorAll("button").length` e `console.log("mensagem")`.
  `help` lista os comandos suportados. As setas percorrem o histórico de comandos.
  `console.clear()` limpa a saída sem apagar os registros de Rede. Comandos e
  resultados ficam separados por aba enquanto o painel está aberto.
- **Rede:** registros das navegações e ações realizadas pelo componente Browser,
  com URL, operação, resultado, duração medida e mensagem de falha. Possui filtro,
  detalhes e limpeza da aba atual; guarda até 100 registros por sessão.
- **Storage:** favoritos e preferências reais da campanha. As páginas virtuais
  atuais não implementam cookies, localStorage ou sessionStorage por domínio.
- **Sessão:** abas abertas, histórico recente e posições disponíveis para voltar
  e avançar. Histórico e abas duram até fechar o navegador.
- **Código fonte:** cópia do HTML renderizado da página virtual; não é uma resposta
  HTTP original. Valores editáveis de inputs e textareas são removidos da cópia.

O console interpreta um conjunto limitado de consultas, sem executar JavaScript
arbitrário no WebView do jogo. As consultas acessam apenas os elementos da página
virtual. A aba Rede apresenta resultados reais dessas operações, sem inventar
códigos de status HTTP para o transporte interno do jogo.

## Verificação em 15/09/2026

- 174 testes do frontend passaram, incluindo 12 testes do navegador.
- TypeScript, ESLint dos arquivos alterados e Stylelint do painel passaram.
- Build web de produção concluído. Permanecem os avisos existentes de tamanho
  do chunk principal e comentários de pureza do Zod.
- Verificação no WebView2 nativo com campanha isolada em memória: consulta ao
  título da Wipédia, código fonte, navegação com falha, registro do erro e retorno
  à página anterior.
- Inspeção visual em janelas de aproximadamente 900 e 560 pixels: campo do console
  visível e conteúdo com rolagem, sem transbordamento horizontal do navegador.
- Capturas: `artifacts/browser-console.png`,
  `artifacts/browser-console-narrow.png` e `artifacts/browser-source-narrow.png`.

Esta alteração atualiza o projeto e o build web. O executável de distribuição
precisa de uma nova compilação para incorporar estes recursos.
