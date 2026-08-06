# Concilio WP-104 — INDICE (su S-102 e programma S-103)

**Formato indice** (decisione utente 2026-08-06): questo file INDICIZZA,
non copia — i testi integrali vivono SOLO in `verbali/`. Verbali
individuali = fonte VINCOLANTE; note di team = coordinamento; sintesi =
`verbali/SYNTHESIS.md`.

**Esito in sessione**: convocato alla chiusura S-102 (2026-08-06,
protocollo due fasi). 9/9 sedie; nessun MI OPPONGO; **8 refutazioni
capitali** (Hoare, Matsakis, Klabnik [SANATA in sessione], Hejlsberg,
Bak ×3, Pedersen, Leijen, Gregg; Stogov: nessuna capitale ma riserva
forte). L'ordine DEFINITIVO S-103 è nel §S-103 di
NEXT_SESSION_WORDPRESS.md, derivato da `verbali/SYNTHESIS.md`.

## Fase 1 — verbali individuali (vincolanti)

| sedia | file | ricevuta |
|---|---|---|
| 1 Hoare | [verbali/verbale-1-hoare.md](verbali/verbale-1-hoare.md) | CON EMENDAMENTI — A-HO-104-1 Generator: fixture di morso o birth-track, terza deroga vietata; -2 riga base=1 (temp receiver) nella tavola INV-RECV-1; -3 marcatori stabili ai 12 osservatori; -4 invariante diags mai troncata + assert anti-sovrapposizione marche; -5 H-C2 via predicato unico — KS-HO-104-1/2/3 — **capitale: l'audit prova «invariante» solo sotto base=2; il caso base=1 tocca due osservatori `==2` non esaminati** |
| 2 Matsakis | [verbali/verbale-2-matsakis.md](verbali/verbale-2-matsakis.md) | CON EMENDAMENTI — A-MA-104-1 fixture 19 ricevitore-ultima-ref mid-arm (17-18 girano intra-arm ma con slot vivo: slack ≥2, il −1 non è arbitrato); -2 audit get_mut/try_unwrap/make_mut; -3 KS-MA-103-x in NEXT_SESSION; -4 fast-out H-C2 = is_gc_container — KS-MA-104-1/2 — **capitale: fixture a distanza ≥2 dalla soglia non arbitra il −1** |
| 3 Klabnik | [verbali/verbale-3-klabnik.md](verbali/verbale-3-klabnik.md) | APPROVATA CON RISERVA — A-KL-104-1 regola scritta «set che SCENDE» (4 passi + ri-verdetto retroattivo); -2 hash phpr fail-closed nei gate fixture; -3 mode-probe nei fixture-gate; -4 celle cb worker-diverso + post-errore; -5 hc2-criterio.out nominato ORA + controllo positivo H-D — KS-KL-104-1/2/3 — **capitale: gate corpus citato «verde» con rc=2 archiviato, ri-giudizio non registrato → SANATA in sessione (`wp102-harness/corpus-gate/riverdetto-ref1417.txt`, verde ×2 modi)** |
| 4 Hejlsberg | [verbali/verbale-4-hejlsberg.md](verbali/verbale-4-hejlsberg.md) | APPROVATO CON RISERVE — A-HE-104-1 braccio `=0` discriminante nel dente sottoprocesso; -2 controllo positivo che il dump copra prop_init; -3 body-zoo `== 1` esatto + funnel da all_funcs; -4 A-HE-103-7 PRIMA del punto 3 S-103, A-HE-103-2 al punto 5 timeboxed — KS-HE-104-1 pin `size_of::<Zval>()` prima di H-C2 — **capitale: il dente dichiara «modulo intero» senza provare che il dump stampi i corpi fuori-funnel** |
| 5 Bak | [verbali/verbale-5-bak.md](verbali/verbale-5-bak.md) | census pila VALIDO su prop, programma S-103 emendato — A-BA-104-1 denominatore per sito×prim mai «÷23»; -2 leva-nulla PREFISSO di H-C2; -3 drop-census prima della banda; -4 H-D istogramma size-class + tag TL RAII (indiziato: ret_cell Rc); -5 Other+grow — KS-BA-104-1/2/3 — **capitali (3): denominatore aggregato inesistente; ~11 drop mai contati; ABAB cieco al code-layout** |
| 6 Pedersen | [verbali/verbale-6-pedersen.md](verbali/verbale-6-pedersen.md) | S-102 AMMESSA (debito saldato, lettera intera, KS rispettati); S-103 punto 1 CON EMENDAMENTI — A-PE-104-1 braccio warning-line cb2 (§3.13 al confine); -2 interleaving cross-fixture; -3 riga 49a91e4d NON-pin nel registro; -4 grado MINIMO basta per S-103 — KS-PE-104-1/2/3 — **capitale: il launcher S-102 invariato è CIECO al §3.13 che motiva il pin nuovo** |
| 7 Leijen | [verbali/verbale-7-leijen.md](verbali/verbale-7-leijen.md) | REGGE CON RISERVE — A-LE-104-1 realloc disaggregato in CountingMi; -2 due-punti calls_small; -3 istogramma size-class + tag per-sito con residuo≡0; -4 dump atexit senza note zval; -5 zona marginale A/B (Δ∈(banda,2×banda] ⇒ R≥7) — KS-LE-104-1/2/3 — **capitale: «2 alloc/chiamata, 35 B» è ESISTENZA, non cifra (realloc conta doppio; netto 32,0 B; linearità di calls mai misurata)** |
| 8 Stogov | [verbali/verbale-8-stogov.md](verbali/verbale-8-stogov.md) | APPROVATO CON RISERVA FORTE — A-ST-104-1 fixture handler-timing (la marca cura la riga, non il tempo); -2 censimento famiglia fetch non-timbrata §3.11/§3.12; -3 fixture generator-in-cycle (Generator=false infedele a Zend); -4 sanare contraddizione Ref in is_gc_container; -5 fixture 15-bis float→int + typed-REF — KS: H-C1c resta GATED; niente leve gc_note senza -4 — nessuna capitale, ma «FEDELE» è sovradimensionato (5 siti su ~435) |
| 9 Gregg (mandato inverso) | [verbali/verbale-9-gregg.md](verbali/verbale-9-gregg.md) | AMMESSA (oggetto avanzato in conoscenza; prezzo ordinato, non deriva — limite nominato) — A-GR-104-1 banda tra-sere del giudice come NUMERO (≥3 sere); -2 audit finestra A/B peak + tetto spread 1,5× fase 1; -3 contatore sessioni-senza-Δ-oggetto (=1); -4 riconciliare contabilità calls prima di H-D — KS-GR-104-1/2 — **capitale: i «trasversali» S-101 sono TRA-SERE (calls 7,3→7,7 li rimangia), declassati a indizio** |

## Fase 2 — note di team

| team | file | ricevuta |
|---|---|---|
| ricevitore (Hoare·Matsakis·Stogov) | [verbali/team-ricevitore.md](verbali/team-ricevitore.md) | convergenze: fixture 19 a due corni (A-MA-104-1∘A-HO-104-2); esito audit ristretto a slot-held; gate MOVE/H-C1c (KS-MA-104-2∘KS-ST-104-1); H-C2 solo via predicato unico (KS-MA-104-1); Generator senza terza deroga; §3.13 claim ridimensionato+assert — conflitti: Ref (composto: assert obbligatorio), soglia Generator (KS-HO-104-3 prevale), onere base=1 (chiude fixture 19b) — promozioni H-C1a/b INTATTE |
| misura (Bak·Leijen·Gregg) | [verbali/team-misura.md](verbali/team-misura.md) | convergenze: canale-mai-contato fuori dai criteri (KS-BA-104-3≡RC-LE-104-1); attribuzione H-D istogramma+tag residuo≡0; denominatori per specie mai ÷23; tetto spread 1,5×; bande=numeri locali — conflitti: leva-nulla layout (Bak) vs calibro profilo (Gregg) → UNA build due letture; «2 alloc/35B» → adottata Leijen (esistenza, non cifra) — priorità: H-C2 leva-nulla→drop-census→criterio→A/B; H-D istogramma+tag→cifra netta→leva |
| catena (Klabnik·Hejlsberg·Pedersen) | [verbali/team-catena.md](verbali/team-catena.md) | convergenze: le 3 capitali = UNA classe (verde senza artifact che possa mostrare rosso); braccio discriminante ovunque; fail-closed hash+registro; attese esatte — conflitti: celle workers=2 vs grado MINIMO (composto: celle nel minimo, pieno resta option+restapi); dump-prima-del-claim vs timebox (claim declassato subito, dump al p.5) — p.1 S-103: launcher + cb2 warning-line + cross-fixture + errore-poi-successo + riga NON-pin; RC-KL-104-1 SANATA (riverdetto-ref1417 rc=0) |

## Sintesi

[verbali/SYNTHESIS.md](verbali/SYNTHESIS.md) — apre con §FONDAMENTALI
(regola 2026-08-03); ordine DEFINITIVO S-103 recepito in
NEXT_SESSION_WORDPRESS.md §S-103.
