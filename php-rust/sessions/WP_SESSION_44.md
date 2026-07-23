# WP_SESSION_44 — archivio storico della sessione WP-44

> Convenzione: un file per sessione; il handoff tiene solo l'ultima sintesi.
> Dettagli gemelli in memoria: topic php-rust-wordpress-track.

> 🚫 **WP-44 (2026-07-23, commit `35ff89f`+`1e365db` REVERTATI da
> `b1ea256`+`ebc0eb6` — tree finale BYTE-IDENTICO a `e3c8e0b`/WP-43)** —
> **STADIO 2 Leva B (Binary/CmpJmp a operandi diretti) ESEGUITO, PROVATO E
> BOCCIATO su A/B in DUE forme: +1,17% e +1,28% consistenti (12/12 round
> new>old, segno mai invertito). Revert secco da protocollo. ARCO REGISTRI
> CHIUSO** (le fusioni WP-32/33/34 restano; infrastruttura stadio 1 a delta
> zero resta, pass di nuovo vuoto).

## Cosa è stato costruito (e revertato — il codice vive in `35ff89f`/`1e365db`)

- **`Op::BinaryReg { op, l, r, dst }`** e **`Op::CmpJmpReg { op, l, r,
  addr, when }`** con `Operand{Stack|Slot|Const|Temp}`; Op resta 48B
  (size-test pinnato). Handler con fast path `binary_fast` a borrow (zero
  cloni operandi sul hit) e funnel generico `binary_value_ab`; store dst
  replica StoreSlot integrale (guardia typed_refs + write-through +
  gc_note). v2: `RegSrc` borrow-or-owned = risoluzione operandi SINGOLA,
  il miss riusa la risoluzione (into_owned = Rc bump).
- **`reg_lower::lower_func`** (stadio 2): finestre contigue
  [LoadVar/PushConst]{0,2} → Binary|CmpJmp|CmpJmpConst, dst-fold su
  `StoreSlot` e `Dup,StoreSlot,Pop`; compattazione con remap TOTALE
  (14 varianti Addr via `visit_addrs` unico + exc_table + lines parallelo;
  sentinelle >len intatte). Guardie di parità: riga sorgente uniforme
  nella finestra (diagnostica), nessun jump-target/exc-boundary a metà
  finestra (a inizio finestra ok), fold LoadVar solo se
  `consts[name]==slot_names[slot]` (il warning risintetizzato è
  byte-identico), indici ≤ u16::MAX, LoadSlot (silente) MAI foldato.
- **Test**: shape (fold slot+slot→slot, cmp slot/const), battery
  comportamentale 11 snippet (loop/try-finally/foreach val+ref/static/
  default/??/?:/undef-warn/ref/self-assign) old==lowered su `rendered`
  (diag inclusi), remap-validità addr, stack-lhs resta CmpJmpConst,
  size Op. cargo 1640→1641 col pass, 1637 dopo il revert.

## Le prove (tutte PASSATE prima del verdetto perf)

1. **Flag OFF byte-id a WP-43**: dump 3 probe (162.615 righe) BYTE-ID
   all'archivio WP-43 in entrambe le forme; out on==off==oracle.
2. **Catalogo diff di stadio pulito** (PHPR_DUMP_OPS, dump-probe):
   v1: Binary −297→BinaryReg +297 · CmpJmp −21 + CmpJmpConst −1038 →
   CmpJmpReg +1059 · LoadVar −665 · PushConst −163 · Dup/StoreSlot/Pop
   −122 appaiati; NESSUN'altra specie. v2: identico ma CmpJmpConst −526
   (solo i foldati; 512 stack-lhs restano monomorfi).
3. **Gate22 COMPLETO TUTTO VERDE due volte su `35ff89f`**: flag OFF e
   flag ON (corpus 1447 · sess 28 · date 351 · refl 290 IDENTICI · ORM
   3E/13F per nome · hk 1665 0E/0F · cargo · gd/mysqli/media BYTE-ID ·
   http DIFF-set 16 identico · option 413 · restapi 3508 per nome COL
   CONTEGGIO). ⭐⭐ Per il gate ON la prova che il flag è vivo è nel log:
   wrapper `gate22-regon.sh` conta le forme registro nel dump (1356) e
   ABORTISCE a 0 — un gate ON verde senza quella prova è indistinguibile
   da un falso-verde a flag morto (`ps eww` su macOS NON mostra l'env
   nemmeno dei processi propri: non è un check).

## A/B go/no-go (gruppo media, user CPU dai .time, stesso giorno)

| round | v1 old | v1 new | v2 old | v2 new |
|---|---|---|---|---|
| 1 | 55,30 | 55,70 | 55,30 | 55,88 |
| 2 | 55,22 | 56,04 | 55,37 | 55,82 |
| 3 | 55,29 | 56,01 | 55,15 | 55,71 |
| 4 | 55,35 | 56,05 | 55,10 | 55,87 |
| 5 | 55,19 | 55,97 | 55,15 | 55,93 |
| 6 | 55,48 | 55,95 | 55,19 | 56,30 |

Medie: v1 55,305/55,953 = **+1,17%** · v2 55,21/55,92 = **+1,28%**.
Oracle 20,75-20,80 in tutte e quattro le serie = giornata pulitissima.
12/12 round new>old ⇒ **regressione consistente, NO-GO, revert secco**.
(old = `phpr-e3c8e0b`, build in `/tmp/phpr-old-44`, binario archiviato in
`phpr-old-target/release/phpr-e3c8e0b`.)

## ⭐⭐ Lezioni (perché l'arco si chiude)

- **La generalizzazione a registri degli op caldi PERDE contro le fusioni
  enumerate**: due corpi handler polimorfi in più nel run_loop costano in
  I-cache/BTB più di quanto l'elisione del data-movement renda (LoadVar
  elisi = clone già economico + dispatch; il grosso del churn Zval sta
  ALTROVE: Ret→DerefTop, call ABI). Fisica coerente con WP-33 ("+2,9% un
  branch mai preso"), WP-38 (i micro mentono), WP-41 (shim bocciato).
- **v1 vs v2 dimostra che NON era un artefatto d'implementazione**: tolta
  la spolimorfizzazione gratuita (CmpJmpConst 1:1) e la doppia
  risoluzione, la regressione resta ≈uguale. È il costo strutturale delle
  varianti aggiuntive + la quota di CmpJmpConst caldi convertiti.
- **⭐⭐ Un gate a flag ambientale VUOLE la prova positiva nel log**
  (conteggio forme nel dump), come i gate DB vogliono il conteggio nomi.
- ⭐ Il pass a finestre con remap totale è CORRETTO e riusabile (gate22
  intero verde due volte sul bytecode riscritto, corpus 1447 compreso):
  se un giorno si riapre, la macchina di riscrittura c'è già in storia
  git (`35ff89f`, `1e365db`).
- ⭐ bash 3.2 macOS: array vuoto + `set -u` = subshell morte silenziose
  (A/B v0 aveva 0 file oracle/old: ramo esplicito, mai `"${A[@]}"` vuoto).
- ⭐ Il monitor sui `.time` segnala l'APERTURA del redirect (inizio run),
  non la fine: l'unico segnale affidabile è il marker `.done`.

## Stato finale

- **Tree = `e3c8e0b` byte-identico** (diff vuoto); cargo **1637/0**;
  dump probe BYTE-ID a WP-43; out == oracle; flag-on (pass vuoto) =
  identità. Full gate22 post-revert NON rilanciato: tree identico allo
  stato gated verde (9cc141b/e3c8e0b) E il gate OFF di oggi girava su
  bytecode byte-identico — doppia copertura.
- Commit di sessione: `35ff89f` (stadio 2 v1) · `1e365db` (v2) ·
  `b1ea256`+`ebc0eb6` (revert) · docs di chiusura.
- Harness: `wp44-harness/` (build-old-44.sh, ab44.sh, gate22-regon.sh,
  dump-*, ab-out-v1/, ab-out/=v2); gate archiviati in
  `wp22-harness/gate-out-wp44-{off,on}-archived/`.
- Disco: 16G liberi a inizio; `php-rust-output/debug/` rigenerato dal
  cargo test debug (3,8G) e ripulito in corsa — usare SEMPRE
  `cargo test --release` in sessione.

## Decisione d'arco e prossimo lavoro

**Arco registri (Leva B) CHIUSO allo stadio 1** (infra dormiente a delta
zero; stadi 3-4 NON si aprono: condividono la stessa premessa fisica
falsificata — più forme nel loop caldo per elidere data-movement). Il
census resta valido come mappa (Ret→DerefTop 40,5M = call ABI), ma ogni
riapertura richiede un cambio di fisica del dispatch, non nuove varianti.
Da rotta ([[php-rust-roadmap-wp-first]]): con l'arco perf chiuso si apre la
**validazione Laravel**; in alternativa il backlog di
[[php-rust-todo-master]] (candidato pronto: bug isset via `__get` annidato,
probe wp42-harness/probe-isset-div.php §3).
