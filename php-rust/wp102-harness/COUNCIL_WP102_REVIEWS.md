# COUNCIL_WP102_REVIEWS — Concilio WP-102 su S-100 e programma S-101 (2026-08-05/06)

**INDICE, non copia** (decisione utente 2026-08-06: i verbali integrali
vivono SOLO in `verbali/` — duplicarli qui faceva pagare due volte gli
stessi token a ogni lettura futura). Fonte VINCOLANTE = i file linkati.

## Sintesi di convergenza (operativa, VINCOLANTE per S-101)

→ [`verbali/SYNTHESIS.md`](verbali/SYNTHESIS.md) — §FONDAMENTALI in testa,
6 refutazioni capitali per convergenza (1-3 SALDATE in sessione),
riformulazione H-C1 a stadi, ordine DEFINITIVO S-101, backlog per NOME,
conflitti registrati.

## Fase 1 — verbali individuali (9 sedie, indipendenti)

| Sedia | File | Ricevuta (≤80 parole, dal messaggio finale dell'agente) |
|---|---|---|
| 1 Hoare | [`verbali/verbale-1-hoare.md`](verbali/verbale-1-hoare.md) | CONCORDO CON EMENDAMENTI — A-HO-102-1 (emit_binary usa `enabled()` non ctx.reg_lower: braccio OFF ≠ produzione OFF), -2 sigillo ZST, -3 design H-C1 refcount+COW + fixture aliasing, -4 corpo condiviso — KS-HO-102-1/2 — capitale: sì, «il modo è un INPUT del funnel» falsificato da un sito residuo |
| 2 Matsakis | [`verbali/verbale-2-matsakis.md`](verbali/verbale-2-matsakis.md) | S-100 regge; capitale su §S-101: H-C1 «clone del valore» punta al canale sbagliato (valori Long-Copy; il churn è il RICEVITORE Rc+gc_note; prior art ThisPropGet) — A-MA-102-1..4 (census per sito, fixture hazard, sigillo di tipo, perimetro tripwire) — KS-102-1..4 |
| 3 Klabnik | [`verbali/verbale-3-klabnik.md`](verbali/verbale-3-klabnik.md) | capitale: «il flip cambia solo la costante» è falso (fb861e4 ricabla l'entry; diff per-test mai ri-giudicato sul pin) — A-KL-102-1 corpus-diff SUL PIN, -2 carve-out per token, -3 dente absent≡`=1`, -4 matrice H-C1 — KS-102-1..3 (evidenza=albero giudicato; soffitto ~9×; WP pair non derogabile) |
| 4 Hejlsberg | [`verbali/verbale-4-hejlsberg.md`](verbali/verbale-4-hejlsberg.md) | S-100 refutata sul claim di testa (concorde con Hoare su emit_binary); §S-101 ammissibile coi denti — A-HE-102-1..6 (dente BinaryAdd OFF, batteria 2 modi, destructuring CompiledClass, perimetro tripwire, modo nel Module, dump ereditarietà) — KS-HE-102-1/2 |
| 5 Bak | [`verbali/verbale-5-bak.md`](verbali/verbale-5-bak.md) | capitali: H-C1 nominata su carico di soli int dove il clone non alloca — ri-nominare il meccanismo — A-BA-102-1 census alloc/refcount, -2 profilo inline-aware del 50% run_loop, -3 fixture per specie, -4 L=12,9 mai coefficiente — KS-BA-102-1 tetto ~9×, -2 ns/op per specie |
| 6 Pedersen | [`verbali/verbale-6-pedersen.md`](verbali/verbale-6-pedersen.md) | capitale: la parità bimodale server non prova che il braccio off girasse off (fails=0×2 indistinguibile da env mai propagato) — A-PE-102-1 mode-probe, -2 sigillo STUB_ELISION/UNIT_CACHE, -3 endurance N≥100, -4 WP via HTTP, -5 gradazioni col binario esecutore — KS-PE-102-1/2 |
| 7 Leijen | [`verbali/verbale-7-leijen.md`](verbali/verbale-7-leijen.md) | capitali: «cross-albero» non provato contro l'ambiente; peak 1929,0 pubblicato per binario mai misurato — A-LE-102-1 A/B pin stessa-sera prima del bisect, -2 voce rinominata OFF+95/ON+36,5, -3 rumore R≥5 banda unilaterale, -4 peak con hash — KS-LE-102-1..3 |
| 8 Stogov | [`verbali/verbale-8-stogov.md`](verbali/verbale-8-stogov.md) | nessuna capitale — A-ST-102-1 riscrivere §3.12 (meccanismo UNDEF→coercizione verificato nel C), -2 §3.11 famiglia fetch-undef, -3 H-C1 in due stadi (censimento specie + bypass scalari), -4 fixture semantiche, -5 census bi-regime calls — KS-ST-102-1..3; il «prestito» secco è forma sbagliata (Zend fa copy+addref condizionale) |
| 9 Gregg | [`verbali/verbale-9-gregg.md`](verbali/verbale-9-gregg.md) | (mandato inverso) PASS pieno: sessione d'oggetto piena, contatore 0, nessuna capitale — A-GR-102-1 census dinamico valida lo statico, -2 decomporre run_loop, -3 bande asimmetriche sul rumore misurato, -4 ri-baseline prima del criterio — KS-GR-102-1/2 |

## Fase 2 — note di team (relatori, 2-3 sedie ciascuno)

| Team | File | Ricevuta |
|---|---|---|
| flip-residuo (1+4+3) | [`verbali/team-flip-residuo.md`](verbali/team-flip-residuo.md) | capitale condivisa emit_binary (fix applicato, denti verdi); fix-solo-col-dente; evidenza solo dal pin (ri-run programmato); conflitti: forma del fix (Hoare forte vs ctx applicato), sigillo ZST (backlog per regola di ammissione) |
| hc-canale (5+2+8) | [`verbali/team-hc-canale.md`](verbali/team-hc-canale.md) | H-C1 sostituita da forma a stadi H-C1a/b/c; borrow nudo dello slot valore VIETATO all'unanimità; misura prima: ri-baseline + census specie×sito×canale con 3 predizioni; tetto ~9× nell'ordine; conflitto prestito-ricevitore (Matsakis borrow col tipo vs Stogov addref: decidono le fixture) |
| misura-server (7+9+6) | [`verbali/team-misura-server.md`](verbali/team-misura-server.md) | bande peak unilaterali sul rumore misurato per-motore (banda<rumore=VOID); ogni cifra nomina l'hash; voce rinominata crescita d'albero OFF+95/ON+36,5; punto 4 riformulato (A/B pin prima del bisect); verdetti divergenti su S-100 per domini disgiunti |

## Esito in sessione (S-100 chiusura)

Capitali 1 (emit_binary), 2 (evidenza sul pin), 3 (mode-probe) SALDATE a
macchina PRIMA della chiusura: fix b618e3a, batteria 1735/0, corpus-diff
ZERO sul pin, parità server ×2 con modo effettivo provato. Pin finali:
phpr f29883eb432806ce · php-server 62b978c51c62e108 (registro graduato in
`PIN_REGISTRY.md`). Ordine DEFINITIVO S-101 recepito in
`NEXT_SESSION_WORDPRESS.md` §S-101.
