# WP_SESSION_102 — S-102: l'ordine WP-103 saldato — il server gradato, il MOVE messo sotto guardia, la pila operandi contata, il warning fedele alla sua riga

**In una frase**: abbiamo verificato a fondo che le ottimizzazioni recenti del
nostro motore PHP non cambino nessun comportamento visibile (comprese le
situazioni più insidiose ai confini tra richieste web e distruttori di
oggetti), abbiamo contato con precisione dove il motore spende i suoi accessi
di memoria interni, e corretto un difetto per cui certi avvisi d'errore
venivano stampati con il numero di riga sbagliato — correzione che fa passare
un test di compatibilità in più.

**Data**: 2026-08-06 (09:0x–13:0x). **Modello verificato all'apertura**:
Fable 5. **Ordine eseguito**: Concilio WP-103 §S-102 punti 1, 2, 3 (fase 1 +
A/B LANCIATA), 4 (census; H-C2/slot-diretti restano gated in S-103), 5, 6
(fix §3.13 COMPLETO + seconda pietra H-D). **Commit**: 9e2910a → 98f20ec →
ee842d0 → b6f8098 (+ rotazione), tutti su main, pushati.

## Ordine eseguito

| # | Esito |
|---|---|
| **1 · Collaudo php-server 2c4242b6** | **GRADATO al minimo A-PE-103-2** (debito NON condizionato saldato): sentinella estesa bimodale (16 interleaved + 4 concorrenti, workers=2) PASS ×2 con mode-probe; **dente CAPTURE-BOUNDARY nuovo** (`wp102-harness/fixtures/cb1.php`): output da `__destruct`/shutdown alla request_end, 3 richieste consecutive su workers=1 (stesso worker per costruzione) byte-id, corpo IDENTICO all'oracle `php -S`, mode-probe dedicato sul dump dell'unità, cross-mode byte-id. fails=0×2 (`collaudo-out-{off,on}/`). Riga 2c4242b6 AGGIUNTA a PIN_REGISTRY (mancava) |
| **2 · Guardie famiglia MOVE** | (2a) **INV-RECV-1 nominata e auditata** (`inv-recv-1-audit.md`): 12 osservatori assoluti di `strong_count` in classe A (mid-arm) + classe B (teardown/census) — verdetto INVARIANTE su tutti (il MOVE toglie il *secondo* handle del braccio, mai l'ultimo); commenti ai due siti nel codice; ADDENDUM RC-MA-103-1 alla motivazione registrata di hc1b-criterio.out. (2b) **fixture 14-18** (destruct-reenter-PropSet, typed-write-coercion, clone, lazy-init-drop-ultima-ref, propset-gc-mid-arm), attese PRIMA, **5/5 byte-id oracle nei 2 modi al primo colpo** (2 correzioni post-oracle alle attese REGISTRATE in testa); gate pinnato ai 5 NOMI. (2c) **`Zval::is_gc_container`** esaustivo senza wildcard (variante nuova ⇒ errore di compilazione) che codifica i due livelli Zend (Str/Resource refcounted MAI collectable; Generator = buco A-HO-103-2 NOMINATO); i 3 siti duplicati convergono; perf-neutro provato al ri-baseline (prop 11,5 invariato) |
| **3 · Peak (punto 4 WP-102 emendato)** | **Fase 1: banda rumore full-peak PHPR misurata per la PRIMA volta** (R=5 identici, pin S-100, modo off, `peak-noise-out/noise-band.out`): mediana **1896,91 MiB**, spread **34,64 MiB** (~1,8% — l'oracle balla ~10%!), min 1877,08 max 1911,72. Spread < 48 ⇒ **bisect ammesso dal lato rumore** (KS-LE-103-3). Criterio+regola di chiusura scritti PRIMA (`s102-peak-criterio.out`). **Fase 2 A/B ABAB pin S-99↔S-100 off/off LANCIATA detached** (R=5/braccio, ~3h): verdetto MECCANICO in `peak-ab-out/ab-verdetto.out` dalla regola pre-registrata — lettura = primo atto S-103 |
| **4 · Census pila operandi** | `vm/stackcensus.rs` per SITO-OPCODE × PRIMITIVA (macro `scn!` nei 13 bracci del giudice); attese statiche PRE-registrate (`hc-stack-attese-s102.out`) **CONFERMATE dal dinamico**: **23 transiti-sorgente/iter esatti** (push=9 pop=9 peek=2 len=1 elem=2), code costanti = warmup IC (2 fallback PropGet + 1 miss PropSet), linearità 300:1 (`hc-stack-census-s102.out`). NIENTE attesi in ns (KS congiunto rispettato): è il DENOMINATORE dei futuri Δ_A/B. **Gamba alloc a mem-census DIRETTO** (A-LE-103-1): GA_*_N nel CountingMi ⇒ su prop **+2 eventi alloc su 29,9M iter = alloc/iter ≡ 0 CONTATO** (declassamento S-101 registrato). H-C2 e slot-diretti: gate saldato, aperture in S-103 (un'altra leva runtime oggi = un altro ciclo corpus+coppia) |
| **5 · Denti e igiene** | **A-HE-103-3**: dente VERO absent≡`=1` in SOTTOPROCESSO (`php-cli/tests/absent_eq_one.rs`: env COSTRUITO, dump-diff BYTE su BODY_ZOO, controllo positivo forme-registro) — VERDE; la metà tautologica `f(x)==f(x)` ELIMINATA (resta la grammatica, A-HE-103-4). **A-HE-103-1 body-zoo**: attesa scritta PRIMA **CONFERMATA** — sotto ON il residuo `Binary(Add)` fuori-funnel (prop_init, RC-2) ESISTE ed è pinnato per NOME nel dente. **A-KL-103-2**: gate fixture pinnato ai 13 NOMI o VOID (e ai 5 nel runner MOVE). Batteria **1739/0** |
| **6 · Fix §3.13 FEDELE** | `diag_line_marks` + `mark_pending_diag_lines`: il warning di lettura proprietà si timbra con la riga dell'op che LEGGE all'ACCODAMENTO; `flush_diags` preferisce la marca (il flush può cadere sulla riga dopo). Carve-out 09 **CANCELLATA nello stesso commit** (KS-ST-103-3): fixture 13/13 a diff ZERO puro. **🔵 MIGLIORIA: passa `nullsafe_operator/015.phpt`** (warning-line test, byte-id oracle nei 2 modi) ⇒ riferimento corpus **1418 → 1417 per NOME**. **Seconda pietra H-D** (timebox): il mem-census su calls scopre **~2 alloc + ~2 free PER CHIAMATA** (~35 B/alloc, churn bilanciato invisibile alle pagine — RC-LE-103-1 concretizzata); canale NOMINATO per l'apertura H-D |
| **Gate cumulativi** | Batteria 1739/0 · fixture 13/13 (senza carve-out) + 5/5 MOVE nei 2 modi · corpus **1417×2 per NOME** + diff per-test off↔on ZERO · **coppia WP bimodale**: media 0 failnames ×2, full = SOLO delta pre-esistente `wp_is_stream` identico nei 2 modi, full CPU 1,891/1,894 (banda tra-sere), peak full off 1989,88 / on 1942,05 MiB · ri-baseline R=5: **prop 11,5 · arith 12,3 · calls 7,7 · str 6,6 · arr 4,6 · re 3,6** (±0,4 bidirezionali = tra-sere; nessuna leva perf spedita) |

## 🔵 Scoperte

1. **Il call-path ALLOCA**: ~2 alloc + ~2 free per chiamata su calls.php
   (mem-census diretto) — il verdetto «alloc/iter=0» era SOLO di prop; il
   churn bilanciato è invisibile alle stats a pagine (Leijen aveva ragione
   due volte). Canale nominato per H-D.
2. **La banda rumore full-peak PHPR è ~1,8%** (34,64 MiB su ~1897) — molto
   più stretta del ~10% dell'oracle: la gamba phpr È bisecabile sul peak.
3. **Il fix di fedeltà §3.13 ha ripagato in compatibilità**: un phpt del
   corpus congelato è passato (1418→1417) senza che nessuno lo cercasse.
4. **Lo statico stavolta ha retto**: le attese del census pila (23
   transiti/iter) confermate ESATTE dal dinamico — quando il sentiero è
   fast-path puro, la lettura del codice basta; le code costanti erano
   tutte warmup IC.
5. **Il residuo fuori-funnel esiste davvero**: sotto ON `prop_init`
   conserva `Binary(Add)` generico (attesa Hejlsberg confermata al primo
   run del dente) — pinnato per NOME, non più invisibile.

## ⭐ Lezioni

- ⭐⭐ **Un verdetto «≈0» vale solo sul giudice che l'ha prodotto**: 
  alloc/iter=0 su prop NON si estendeva a calls (2/iter) — ogni categoria
  esige il SUO census, mai l'estrapolazione.
- ⭐⭐ **Lo strumento giusto trova ciò che quello sbagliato certificava
  assente**: le stats a pagine avevano «confermato» zero churn; il
  contatore di eventi l'ha smentito alla prima categoria nuova.
- ⭐ **Un fix di fedeltà può mordere il corpus in positivo**: il set fail
  è sceso per NOME — il riferimento si aggiorna con la miglioria
  DOCUMENTATA, mai in silenzio.
- ⭐ **Il hook che blocca `.rs` nel comando morde anche i messaggi di
  commit**: il messaggio si scrive su file con Write e si committa con
  `-F` (senza nominare i sorgenti nel comando).

## Stato binari e processi

- phpr: **d0b01362433b3039** @ HEAD (fa fede HEAD; hash churna col relink)
  — DEFAULT flag-ON; contiene is_gc_container + fix §3.13 + denti. Stash
  ADDITIVO `phpr-s102`. Batteria 1739/0. Corpus 1417×2 per NOME + diff ZERO.
- php-server: pin gradato resta **2c4242b6** (runtime S-101). ⚠️ Il runtime
  è cambiato in S-102 ⇒ **build con ricetta + collaudo del pin NUOVO =
  debito NON condizionato, primo atto S-103** (dottrina Pedersen; il
  49a91e4d della build workspace è effetto collaterale SENZA feature
  axum-server: NON è un pin e non si registra).
- **A/B peak IN VOLO** a fine sessione (`peak-ab-out/`, flag `ab.done`):
  10 run full ABAB, pin verificati FAIL-CLOSED; nessun'altra run pesante
  va lanciata finché non chiude.
- Uploads ripristinata dalla guardia dopo ogni batteria (backup conservati);
  MySQL wp8 su. Harness di sessione: `wp102-harness/`.
