# VERBALE — Brendan Gregg, sedia 9, Concilio WP-91

VERDETTO: **CONCORDO CON EMENDAMENTI**

## Q1 — storia g1→g2→g3 nel ledger

Ledgerata per generazione: righe 61-63 di `m89.campaign.ledger` portano
`judge_sha` + `generation` + `esito` + `fails=` + `verdict_file`. Verificato a
macchina: g1 `ae4f528f93095979` = blob committato **1021162**; g3
`294898431ef72d16` = blob **778d96f** = file attuale. **BUCO 1 (capitale di
catena): g2 `24cd290ae0a9fc2b` NON risolve a NESSUN blob committato** — stato
working-tree transiente (tutte e tre le generazioni "judged at HEAD=418def8",
HEAD mai mosso). Il pin ==1 refutato di g2 è ricostruibile solo dall'OUTPUT
(g2.out r.15-54), non dal codice. **BUCO 2**: nessun campo `reason=` /
`supersede_of=` — il PERCHÉ del supersede (g1: regime clamped nuovo da
DICHIARARE; g2: pin di forma-canale sbagliato) vive nei verdict file e nel
doc, non nel ledger. **BUCO 3**: la qualifica "FAIL del GIUDICE, non VOID
della campagna" sta SOLO in MEASURE89_RESULTS.md — dal solo ledger un FAIL di
giudice e un FAIL di dati sono indistinguibili.

## Q2 — pin presence-guard ==2

Verificato su **TUTTI i 57 raw m89** (conteggio `tag=mi_proc win=0 `): 2
esatto, zero devianti. `memcensus.rs` `dump_exit()` (r.196-218): dump #1 =
checkpoint `exit_mi` PRE-collect; poi `mi_collect(true)` sotto
`PHPR_MI_COLLECT_EXIT=1` (armato da run_arm) e dump #2 = `exit_collect_mi`
POST. **NON è doppia emissione: sono due checkpoint DISTINTI by design — il
delta pre/post È la misura di retention (commento r.202-206). Una emissione
per win distruggerebbe quell'informazione. La riqualificazione REGGE.**
Estrattore: `verdict89.sh:301` (awk, ultima occorrenza vince in END) e :303
(slack resettato a ogni mi_proc win=0) ⇒ consuma la POST ✓. Fragilità vera:
la riga mi_proc NON porta il nome del checkpoint — selezione POSIZIONALE su
canale append non-atomico (nota A-DL40); e nei campioni pre==post su
`commit=` SEMPRE ⇒ il selettore non è mai stato esercitato da dati che
discriminano. → A-BG54.

## Q3 — A-BG51 consumato

boot_epoch: giudicato su ogni raw (`verdict89.sh:138-148`: presenza +
boot≤epoch, else raw VOID); verificato su raw veri (identity count==1,
`srv_boot_epoch=` presente, coerente). **MA** `BOOT_EPOCH=$(date +%s)` del
HARNESS post-assert (campaign ~r.246), non tempo di boot del PROCESSO:
giudizio quasi-vacuo (morde solo su campo assente/orologio). Pid-echo:
`assert_http_pid` invocato in TUTTI e tre i runner (r.247/284/316) ⇒ **ogni
fase**, prima di ogni request misurata. Però NON è letteralmente "la prima
request": `wait_up` spara fino a 100 probe `/__reqns` NON asseriti prima —
la dicitura del doc va precisata in "prima request GIUDICATA". Lane
preflight: dichiarata (attempt=0 `phase=preflight`, r.84-88) ma ledgerata
SOLO su VOID: ledger pulito non distingue "preflight ok" da "mai girato".

## Q4 — doc contro g3

Ogni cifra verificata nel raw/verdict committato: b/se/2σ/a, 4 modi
dominanti, marginali BASE 3/3 IN, conversioni MiB rifatte ad aritmetica, cal
7.801.102×2, floor_inc, P-DT20 4/4, set clamped {dt1×2, dt2×2, dt5.afirst},
UNSTABLE 2/3 + `regime=OVERLAP-only (A-BG52)` ✓. Regime clamped nel doc con
pari forza del verdict ✓ (DICHIARATO + escluso dal tally + KB-88-1).
**REFUTAZIONE: "un marginale fuori fascia" (doc r.50-52) è FALSO — g3 marca
DUE marginali RET0 [OUT]: 24.395.776 (W4→8, r.112) E 16.400.384 (W12→16,
r.114)**; il riassunto del giudice stesso (r.115) sotto-riporta al singolare.
Inoltre WP_SESSION_89.md (r.44-48) omette il grade ADVISORY di b_ret0 nel
bullet VATTR; il verdict r.117 è ambiguo ("verdict-grade… grade inherits the
slope grades above"). Grade e conclusioni invariati — ma il gate è per NOME,
mai per conteggio.

## Emendamenti

- **A-BG53 (judge tether)**: il giudice di OGNI generazione va committato
  PRIMA del run (come il binario col matrix tether A-SK59); riga
  `phase=verdict` con `reason=` sintetico e `supersede_of=`.
- **A-BG54 (checkpoint nominato)**: `tag=mi_proc win=0` porti
  `ckpt=exit_mi|exit_collect_mi`; estrattori selezionano per NOME; pin ⇒
  "==1 per ckpt" (==2 totale ne discende).
- **A-BG55 (boot reale + preflight-ok)**: srv_boot_epoch dal processo
  (`ps -o lstart=` o header in-band), non wall-clock harness; riga
  `preflight=ok` anche sul cammino pulito.
- **A-BG56 (sanatoria)**: MEASURE89_RESULTS.md corregga "un marginale" in
  DUE per NOME; WP_SESSION_89 citi il grade ADVISORY di b_ret0; il giudice
  stampi conteggio+nomi dei marginali OUT.

## Kill-switch

- **KG-91-1**: judge_sha in ledger senza blob committato risolvibile ⇒
  generazione VOID-of-record: non supersede, non citabile dai doc.
- **KG-91-2**: (post A-BG54) estrattore win=0 posizionale anziché per ckpt
  nominato ⇒ cifra non verdict-grade.
- **KG-91-3**: doc che riporta il CONTEGGIO delle deviazioni difforme dal
  SET per NOME nel verdict ⇒ rigetto al gate cifre.

— Brendan Gregg, sedia 9
