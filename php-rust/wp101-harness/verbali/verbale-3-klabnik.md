# Verbale sedia 3 — Klabnik (spec, testabilità, matrici e gate) — Concilio WP-101

## VERDETTO

S-99.0 è una sessione di misure solida e di onestà rara (il pin refutato a
tavolino, il gate che urla su sé stesso). Ma due denti dichiarati NON possono
mordere, un claim di identità è dedotto da un cronometro, e la bozza §S-100
contiene tre ambiguità che renderebbero il flip del default un salto con gate
di carta. **Refutazioni capitali: sì (R1–R3).**

## Refutazioni capitali

**R1 — `reg_lower_antiputenv.rs`: la selezione del chunk può giudicare l'unità
sbagliata.** `included_chunk` prende il PRIMO chunk di `split("== unit ")` che
`contains(inc_name)`; ma il sorgente del main contiene il path dell'included
come letterale (`include '…/antiputenv-inc-….php'`). Se il dump del `{main}`
stampa quella costante, il primo match è il chunk del MAIN: nel braccio
set→on è compilato flag-off (nessuna forma ⇒ negativa passa vacuamente), nel
braccio unset→off è compilato flag-on (forme presenti ⇒ positiva passa
vacuamente). Entrambi i bracci possono passare senza aver mai guardato
l'unità inclusa. Sommato al compile-order accident già dichiarato, il dente
CLI è doppiamente senza denti.

**R2 — Il sigillo non ha UN test che possa fallire, e il server non è mai
stato eseguito flag-on.** CLI: ammesso nel test stesso. Server: «chiuso per
costruzione» = zero collaudo dinamico; `s99-parity-server.sh` costruisce
l'ambiente con `PHPR_REG_LOWER` ASSENTE, quindi sentinella+option+restapi
hanno collaudato il pin server SOLO flag-off. La promozione a default
accenderebbe di colpo un modo in cui il server non ha mai servito una
richiesta.

**R3 — «Emissione davvero bit-identica» dedotta da 5,43→5,44.** Un cronometro
non prova bit-identità; la prova è il diff dei dump (A-HE-100-4). Il claim va
declassato a «compatibile con»: com'è scritto, dichiara ciò che non prova.

## Refutazioni minori

**R4** — Il gate corpus è un bit/fail per NOME: 1418 nomi uguali non dicono
COME falliscono, né confrontano off↔on tra loro (solo contro la lista wp82).
A-KL-100-2 è senza definizione operativa: quale diff, normalizzato come,
criterio di PASS scritto dove?

**R5** — `pair99.sh names()`: `sed 's/^[0-9]*) …/'` cattura qualunque riga
«N) » anche nei corpi dei messaggi; e «media: 0 nomi IDENTICI» è
indistinguibile da estrattore morto (stessa classe del morso (c)). Nemmeno il
corpus gate ASSERISCE che `wc -l` dei nomi quadri con «failures: N» del
runner: lo stampa e basta.

**R6** — Matrice REG_FORMS asimmetrica: `BinarySS` è negato in antiputenv ma
nessun controllo positivo prova che il corpo lo emetterebbe (negativa vacua);
il braccio unset si accontenta di `any()` mentre il funnel esige tutte e tre
le forme: una regressione parziale del pass passa.

**R7 — Ambiguità §S-100**: (i) post-flip, «flag-OFF» non ha semantica: oggi
il flag è presenza/assenza — serve un disable esplicito (`PHPR_REG_LOWER=0`)
pinnato nel funnel PRIMA del flip, o A-PE-100-4 è ineseguibile; (ii) la
«parità server (script 1a)» con lista chiusa non può esprimere i DUE modi
senza modifica del launcher; (iii) N_OPS≤255 (186/256, A-LE-100-3) assente
dai gate del flip mentre la coda H-B2 aggiunge opcodi.

## Emendamenti

- **A-KL-101-1**: fix selezione chunk (match sull'header esatto dell'unità,
  non substring) + assert che il chunk scelto NON sia il `{main}`.
- **A-KL-101-2**: braccio unset esige TUTTE le forme del funnel; ogni forma
  negata ha il suo controllo positivo o esce dalla matrice.
- **A-KL-101-3**: assert conteggi↔nomi in pair e corpus gate (estratti ==
  dichiarati dallo strumento).
- **A-KL-101-4**: definizione operativa di A-KL-100-2: diff normalizzato
  per-test off↔on, criterio pre-registrato = zero differenze.
- **A-KL-101-5**: semantica del disable post-flip definita e pinnata nel
  funnel PRIMA del flip.
- **A-KL-101-6**: launcher parità bimodale (parametro di modo nella lista
  chiusa).

## Kill-switch

- **KS-KL-101-1**: VIETATO il flip del default finché il pin server non è
  collaudato flag-on (sentinella + option + restapi); violazione ⇒ flip VOID.
- **KS-KL-101-2**: ogni claim «bit-identico» senza diff di dump/output ⇒
  VOID; il cronometro non giudica l'identità.
- **KS-KL-101-3**: un gate i cui nomi estratti non quadrano col conteggio
  dichiarato dallo strumento è VOID: urla su sé stesso prima che sul motore.
