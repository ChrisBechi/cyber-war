# GAME HACKER - Pacote de Documentacao Narrativa

**Status:** Pre-producao narrativa / canon em desenvolvimento  
**Versao:** 1.0 - consolidacao de 18/08/2026  
**Formato:** Markdown  

Este pacote reorganiza todo o material atual de **Game Hacker** em uma estrutura de documentacao semelhante a uma *narrative bible / game design documentation repository* usada em producao profissional.

## Fonte de verdade

A ordem de precedencia e:

1. `docs/01_CANON/CANON_RULES.md`
2. `docs/01_CANON/NARRATIVE_BIBLE.md`
3. Documentos especificos de sessao ou rota em `docs/02_CAMPAIGN` e `docs/03_ROUTES`
4. Documentos de design em `docs/04_DESIGN`
5. Material original em `archive/source_material` apenas para historico e rastreabilidade

Quando um arquivo antigo contradizer uma decisao mais recente, prevalece o documento de maior prioridade.

## Estrutura

```text
Game_Hacker_Documentacao_Profissional_v1/
├── README.md
├── CHANGELOG.md
├── MANIFEST.md
├── docs/
│   ├── 01_CANON/
│   │   ├── CANON_RULES.md
│   │   ├── NARRATIVE_BIBLE.md
│   │   ├── CHARACTER_BIBLE.md
│   │   └── CONTINUITY_AND_CAUSALITY.md
│   ├── 02_CAMPAIGN/
│   │   ├── CAMPAIGN_STRUCTURE.md
│   │   ├── SESSION_00_TUTORIAL.md
│   │   ├── SESSION_01_SCRIPT_KIDDIE.md
│   │   ├── SESSIONS_02_04_DEVELOPMENT_BRIEF.md
│   │   └── SESSION_05_NULL_WAR_BRIEF.md
│   ├── 03_ROUTES/
│   │   ├── ROUTE_POLICE.md
│   │   └── ROUTE_MAFIA.md
│   ├── 04_DESIGN/
│   │   ├── NARRATIVE_SYSTEMS.md
│   │   ├── PRESENTATION_AND_UI.md
│   │   └── SAFETY_AND_SIMULATION.md
│   └── 05_PRODUCTION/
│       ├── DOCUMENT_PRECEDENCE.md
│       ├── OPEN_QUESTIONS.md
│       └── NEXT_STEPS.md
└── archive/source_material/
    └── arquivos originais preservados sem alteracao
```

## Estado atual

O inicio/tutorial e a Sessao 1 estao detalhados. A espinha narrativa, a guerra com NULL e as rotas finais Policia e Mafia possuem estrutura consolidada. O principal bloco ainda a desenvolver em nivel de missao e formado pelas **Sessoes 2, 3 e 4**, seguido pelo refinamento da Sessao 5 e pela integracao final das rotas.

## Regra de producao

Toda nova decisao canonica deve primeiro atualizar `CANON_RULES.md` e, em seguida, o documento especifico afetado. Evite manter a mesma decisao divergente em varios arquivos.
