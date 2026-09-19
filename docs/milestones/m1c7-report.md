# M1C.7 — wc GNU 9.7 certificado

**STOP_REASON: `MILESTONE_COMPLETE`. PUBLICATION_STATUS: `PUBLISHED`.**

| Medida                         | Antes                                                   | Depois                         |
| ------------------------------ | ------------------------------------------------------- | ------------------------------ |
| wc                             | PARTIAL; entrada acumulada                              | VERIFIED; contagem incremental |
| Casos de projeto / GNU         | 4 / 0                                                   | 421 PASS / 421 matches         |
| Contratos / gates obrigatórios | Cobertura incompleta                                    | 15/15 PASS; 26/26 PASS         |
| Known gaps obrigatórios        | Display width, files0-from, palavras com UTF-8 inválido | 0                              |
| Coreutils                      | 9 VERIFIED / 28 PARTIAL                                 | 10 VERIFIED / 27 PARTIAL       |
| GNU global                     | 1220 matches                                            | 1641 matches                   |
| Declarativa global             | 1336 PASS                                               | 1753 PASS; 0 FAIL; 0 SKIPPED   |

Fingerprint: `1d22fa44ac6868c621d1f1e857f61064327c5a81aee37273a7ea87a959600042`.
Os snapshots [before](../coreutils/evidence/m1c7-before.json) e
[final](../coreutils/evidence/m1c7-final-state.json) registram os valores derivados
do tooling, dependências, gates, referências e hashes dos instaladores.

**Contagem e locale.** A certificação usa exclusivamente o locale locked C.
`-l` conta LF; `-w` reconhece SP, HT, LF, VT, FF e CR como separadores;
`-c` e `-m` contam todos os bytes, inclusive NUL, UTF-8 válido, inválido e
truncado. `-L` conta colunas imprimíveis ASCII, avança tabs até múltiplos de 8,
reinicia em LF/CR/FF e ignora a largura dos demais controles e bytes não ASCII.
EOF sem LF não aumenta a contagem de linhas. As colunas saem na ordem l/w/m/c/L.

O perfil anterior C.UTF-8 de caracteres/palavras foi preservado com decoder de
até quatro bytes e testes entre chunks; ele não integra a certificação GNU.
A largura visual comprometida é a do locale C. Não foi criado helper Unicode
compartilhado nem ampliada a promessa para outros locales.

**Streams, arquivos e formato.** O engine usa os mesmos inputs VFS, pipes,
VirtualTty e sinais do scheduler. Consome blocos de até 4096 bytes, com memória
constante para contadores e no máximo um nome parcial mais um bloco para a lista.
Nomes respeitam o limite VFS de 4096 bytes. Arquivos de dados são contados em uma
passagem; stdin repetido permanece em EOF. Totais somam as quatro contagens e
mantêm o máximo de largura, com overflow explícito e sem wrap silencioso.

Para listas regulares pequenas, uma primeira passagem incremental determina
quantidade e metadados dos nomes para o alinhamento; a segunda conta os arquivos.
Listas vindas de pipe são processadas sem antecipação e usam largura numérica 1.
Isso reproduz o GNU sem guardar a lista inteira. NUL delimita nomes; LF faz parte
do nome. Entradas vazias são diagnosticadas com posição; NUL final não acrescenta
entrada. `--files0-from=-` rejeita um nome de dados `-`. Erros preservam as
linhas válidas e os totais observados, incluindo a linha zero de um diretório.
`--total=auto|always|only|never`, abreviações e os limites 9/10, 99/100 e 999/1000
foram capturados.

**Diferenças corrigidas.** A auditoria eliminou a acumulação integral, a ausência
de -L/files0-from, a classificação incorreta de VT, os diagnósticos incompletos
e o alinhamento baseado apenas nos bytes lidos. Probes finais identificaram a
mensagem de loop de symlink na abertura da lista e o uso de aspas C no argumento
excedente com apóstrofo. Ambos usam agora o diagnóstico/quoting GNU e têm regressões
canônicas. Os 421 casos tiveram captura dupla e verify independente;
as 419 observações iniciais permaneceram idênticas. O
[código GNU v9.7](https://github.com/coreutils/coreutils/blob/v9.7/src/wc.c)
orientou os probes; as expectativas vieram somente das execuções do oracle.

**Proveniência e armazenamento DEV.** Os 1220 goldens anteriores
permaneceram byte a byte intactos. A extensão do harness compartilhado foi
revalidada em duas execuções por caso, registrando recibos separados por comando.
Cada recibo vincula o golden, harness, ambiente e binário relevante; mudanças de
bytes ou identidade impedem sua aceitação. Alterar somente o hash de um binário
irmão não invalida esse recibo. Os arquivos locais do contador e dos testes de
wc não são dependências dos handlers de cat/base64/tee.

A matriz global excedeu o limite de string do Node com um JSON de 696 MB. O bridge
passou a gravar um índice pequeno e arquivos por caso com SHA-256. O leitor mantém
no máximo oito resultados no cache; snapshots completos, bytes e comparações
permanecem presentes. Testes verificam roundtrip, merge, leitura do formato antigo
e rejeição de conteúdo/identidade adulterados. Isso pertence somente a DEV/CI.

| Validação final                                                                 | Resultado                                                               |
| ------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| Rust completo, serial                                                           | 282 PASS; 14 testes opcionais ignorados                                 |
| Frontend                                                                        | 243 PASS em 40 arquivos; dois workers                                   |
| Tooling                                                                         | 52 PASS                                                                 |
| Legacy                                                                          | 76 casos exatos PASS                                                    |
| Declarativa global                                                              | 1753 PASS                                                               |
| Normal, regressão e strict dos dez certificados                                 | PASS                                                                    |
| Strict global                                                                   | FAIL esperado: 1510 entradas da dívida conhecida; zero erros adicionais |
| rustfmt, Clippy -D warnings, Prettier, ESLint, Stylelint, TypeScript e conteúdo | PASS                                                                    |
| Web, release Windows, NSIS/MSI, Host Guard                                      | PASS                                                                    |

As propriedades cobrem blocos de 1, 2, 3, 7, 31, 255, 1024 e 4096 bytes e
fragmentação pseudoaleatória, incluindo UTF-8, CRLF, tabs, palavras e nomes NUL.
Há pipelines reais por métrica, produtor de mais de 4 MiB, produtor infinito
controlado, TTY, SIGINT/SIGTERM e verificação de limpeza de handles/processos.
O empacotamento mantém somente o recurso de licenças de arquivos, sem executáveis
externos. Nenhum comando seguinte foi implementado.

- NEXT QUEUE ITEM: `coreutils/sha256sum`.
- CLASSIFICATION: `SMALL_SHARED_EXTENSION`.
- COMPLEXITY: 3/5.
- ACTION: `NEEDS_IMPLEMENTATION_AND_EVIDENCE`.
- KNOWN GAPS: Escaped checksum lists, malformed-line warnings, --strict/--warn/--zero and stdin check semantics incomplete.
- DEPENDENCIES: SHELL.CONTEXT, SHELL.EXPANSION, SHELL.GLOBBING, SHELL.LISTS, SHELL.PARSING, SHELL.PIPELINES, SHELL.REDIRECTION, VFS.DIRECTORIES, VFS.HARDLINKS, VFS.INODES, VFS.METADATA, VFS.PATHS, VFS.PERMISSIONS, VFS.REGULAR_FILES, VFS.SPECIAL_PERMISSIONS, VFS.SYMLINKS, VFS.TIMESTAMPS.

Publicado na main: commit `0ef5bd0` (feat(coreutils): certify wc against GNU 9.7).
