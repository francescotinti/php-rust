# NEXT_SESSION — LA SPINA DORSALE: il nucleo interprete, misurato per categoria

⏱ **FONDAMENTALI**: ultima misura full/media WordPress = **WP-102 (1
sessione fa)** · ultima campagna sull'OGGETTO = **S-103 (questa: prefissi
H-C2 consumati + cifra netta H-D — misure dell'oggetto, nessuna leva)** ·
**sessioni-senza-Δ-oggetto = 2** (S-102 guardie, S-103 prefissi — prezzo
ordinato dai concili WP-103/104; ma ORA i prerequisiti della leva H-C2
sono TUTTI consumati: S-104 non ha più prefissi davanti).

**Ultima sessione (S-103, 2026-08-06)**: l'ordine WP-104 consumato INTERO.
1) **A/B peak: il verdetto meccanico «RUMORE» RIBALTATO dall'audit** —
RUN VOID (spread 90/99 MiB > tetto 51,96; il launcher usava lo spread
intra-run come banda); 5/5 coppie stesso segno B>A = indizio crescita;
**rerun R=7 con warmup pre-registrato IN VOLO** (daemonizzato 16:23,
verdetto con banda fase-1 NEL launcher: `wp103-harness/peak-ab-out/`).
2) **Server 31aa7c2e GRADATO** col launcher EMENDATO (warning-line cb2:
righe 36/41 = oracle al confine HTTP; errore-poi-successo; interleave
workers=2; fails=0×2 + cross-mode). 3) **Pacchetto ricevitore**: fixture
19a/19b PASS ×2 al primo colpo (soglia esatta + base=1), tavola emendata,
OBS-1..12 in codice; assert A-ST-104-4 atterrato sull'invariante VERA
(nested-Ref nel descend) — la premessa «il chiamante scartoccia sempre» è
REFUTATA dai chiamanti reali. 4) **Prefissi H-C2 COMPLETI**: banda-layout
0,67 ns/iter (leva-nulla, semantica nulla provata) · **drop-census 11
DropS + 3 DropC/iter ESATTI** (attesa v2 confermata a zero scarti) ·
criterio scritto (pavimento 4 ns/iter, `size_of::<Zval>()==16` a
compile-time). 5) 🔵 **Cifra netta H-D: 1 alloc × 32,0 B + 1 free per
chiamata, realloc≡0** — il denominatore S-102 era GONFIATO 2× (calls.php
= 20M iter; anche i 10 gc_note/iter ⇒ ~5). 6) Igiene 8/8: 🔵 generator-
in-cycle MORDE (fixture rossa, buco A-HO-103-2 provato); 🔵 §3.12
rititolata typed-LVALUE (azzera anche SENZA ref, 4/4 specie); §3.11 =
tutto il canale RMW; §3.13 chiusa (claim: famiglia PropGet, 5/~435);
set-che-scende scritta; hash fail-closed+mode-probe nei 3 fixture-gate;
dente sottoprocesso emendato (=0 discriminante, prop-init, ==1);
gh-status-sync (2650/1417). Dettaglio: `sessions/WP_SESSION_103.md`.

**Rotta (utente 2026-08-04)**: dritti al PHP; WordPress = collaudo di
PARITÀ. OBIETTIVO **X = nucleo interprete ≤ 3× l'oracle sulle categorie
pure**; giudice = le sei micro-categorie di `wp97-harness/micro/`.

## Baseline CORRENTE del giudice (S-103, sera-2 stesso-pin d0b01362, R=5)

| categoria | rapporto | ipotesi attiva |
|---|---|---|
| aritmetica | **12,3** | residuo = corpi non-Binary + pila operandi |
| proprietà | **11,5** | **H-C2 PRONTA AD APRIRSI: zero prefissi davanti** — canale contato 11 DropS/iter (+3 DropC fuori canale), criterio in `wp103-harness/hc2-criterio.out` (pavimento 4 ns/iter, banda [8,22], banda-layout 0,67, fast-out SOLO via is_gc_container, Δ per sito×specie) |
| chiamate | **7,4** | **H-D: cifra netta NOMINATA (1 alloc × 32,0 B + 1 free/chiamata, realloc≡0, bucket (16,32] uniforme)** — leva GATED sul SiteTag per-sito (residuo≡0); indiziati ret_cell Rc O args Vec (UNO solo, non entrambi) |
| stringhe 6,6 · array 4,4 · regex 3,7 | | — (regex = parte sana) |

(banda tra-sere: 2/3 punti stesso-pin, Δ ≤±0,3 — il terzo punto la chiude)

## LE IPOTESI — stato dopo S-103

- ~~H1, H-A1, H-B1, H-B2~~ chiuse; H-C1a+b spedite e sotto guardia
  COMPLETA (audit + fixture 14-18 + 19a/19b + OBS in codice).
- **H-C2 (proprietà)**: prefissi TUTTI consumati — la sequenza residua è
  SOLO: implementare il fast-out scalare → A/B da sola vs criterio →
  gate pieno (fixture 13+5+19 + batteria + corpus 1417×2 + coppia WP).
- **H-D (chiamate)**: cifra nominata; resta SiteTag TL RAII con
  residuo≡0 (design `wp103-harness/hd-prep-design.md` §3) → attribuzione
  → solo poi leva. ⚠️ tavola bi-regime S-102 da correggere ×½
  (denominatore).
- **Generator-in-cycle**: fixture rossa
  (`wp103-harness/recv-fixtures-gen/`) = arbitro del fix (container +
  descend delle catture, o birth-track). Divergenza REALE che perde
  memoria per tutta la richiesta.
- **H-C1c**: resta GATED su fixture/giudici per SPECIE (KS-ST-103-2).

## Regole di metodo (invariate)

1. Il giudice è la micro-categoria. 2. WordPress è un collaudo di PARITÀ
(si esegue quando cambia emissione O runtime — S-103: runtime parity-null
PROVATO da corpus/batteria/server, coppia non rieseguita e debito
NOMINATO: la prima leva vera la salda). 3. Criterio scritto PRIMA.
4. Apparato solo se blocca; timebox ½ sessione.

## Stato gate

- **phpr (pin release)**: **f45a5d199ab34132** @ HEAD 56a2174 (hash
  churna col relink: fa fede HEAD) — DEFAULT flag-ON; contiene OBS-1..12,
  debug_assert nested-Ref, dcn!/null-lever/memcensus H-D (TUTTO
  parity-null). Batteria **1739/0** · fixture **13+5+2 coi denti nuovi**
  (i gate esigono `PHPR_PIN_ATTESO` o VOID) · corpus **1417 per NOME ×2
  modi** (`wp103-harness/corpus-gate/`). Stash ADDITIVO `phpr-s103`.
- **php-server**: pin **31aa7c2eef899cce** @ HEAD 37312e8 (ricetta piena)
  GRADATO minimo-emendato fails=0×2 (`wp103-harness/collaudo-out-*/`,
  launcher `s103-collaudo-server.sh`); stash `php-server-s103`. Il
  da5c2948 di metà sessione: superseded (mai stashato, sovrascritto —
  lezione: stash NEL momento del grading). Registro: `PIN_REGISTRY.md`.
- **Launcher S-103** (`wp103-harness/`): `s103-collaudo-server.sh` ·
  `s103-recv-fixtures.sh` (19a/b) · `s103-corpus-gate.sh` ·
  `s103-stack-census.sh` (con drop) · `s103-peak-ab.sh` (R=7, banda
  fase-1 nel verdetto) · fixture-gate 13/5 emendati (hash+mode-probe).
- GATE72 CLI baseline trasversale (corpus 1417 · refl 290 · ORM 3E/13F ·
  hk 1665). gh-status-sync ESEGUITO (2650 pass/1417 fail pubblicati).
- ⚠️ `~/Claude/php-rust-output/debug/` si RIGENERA: rimuoverla nel
  pre-flight.

## Voci APERTE per NOME

- **A/B peak R=7 IN VOLO** (fine ~21:00 del 2026-08-06): `wp103-harness/
  peak-ab-out/ab.done` + `ab-verdetto.out` (verdetto meccanico con banda
  fase-1 34,64 e tetto 51,96 GIÀ nel launcher). **Lettura = primo atto
  S-104**; se VOID anche a R=7 ⇒ la metrica full-peak va ridisegnata
  (misura per fase?) prima di ogni bisect.
- **Coppia WP** dovuta alla PRIMA leva vera (debito S-103 nominato).
- **SiteTag H-D** (attribuzione della 1×32B) — design pronto.
- **21,2% run_loop senza nome** — TERZA rotazione: il concilio WP-104 la
  voleva candidata a ordine S-104.
- divergenze §3.11/§3.12 (perimetri MISURATI, probe in
  `wp103-harness/censimento-311-312/`) + generator-in-cycle (fixture
  rossa) — da decidere: fedeltà o assenza consapevole.

## Che cosa è SOSPESO (non abbandonato)

- **A-ZV2** (liveness+TakeSlot); rollout Add forme registro (chiuso salvo
  misura ≥ pavimento); roadmap footprint (riferimenti S-102: full peak
  on 1942,05 / off 1989,88 MiB).

## NON riproporre

Tutti i NON-riproporre WP-83..102 restano. Nuovi da S-103:

- **citare il verdetto meccanico di un launcher la cui banda non è
  pre-registrata ALTROVE** (il verdetto «RUMORE» S-102 confrontava col
  proprio spread: si assolveva da solo — la banda vive nel criterio,
  mai nel run).
- **denominatori di micro-giudici presi dalla memoria di sessione** (il
  2× di calls: si rilegge il SORGENTE del giudice prima di firmare una
  cifra per-iter).
- **assert piazzato senza enumerare i chiamanti reali** (l'assert Ref nel
  predicato avrebbe morso gc_note e le catture by-ref).
- **pin gradato e non stashato NEL momento del grading** (da5c2948
  sovrascritto).
- **output di collaudo/run committati nel repo** (26 MB di srv.log
  rimossi: prima di `git add -A` si guarda la convenzione).
- ereditati: verdetti «≈0» estesi a giudici diversi; gamba alloc a
  pagine; attesi in ns da conteggi; .rs nei comandi git (Write+commit
  -F); narrazione cross-sessione di Δ non-A/B; .done rosso citato verde.

---
**Riscritto**: rotazione S-103 il 2026-08-06. Apertura/chiusura = skill
`apri-sessione`/`chiudi-sessione`. Harness di sessione: `wp103-harness/`.

## §S-104 — ORDINE (BOZZA pre-concilio; il Concilio WP-105 lo emenda e VINCOLA)

1. **Verdetto A/B R=7** (lettura + audit finestra — le regole sono GIÀ
   nel launcher; se VOID di nuovo: ridisegno della metrica, non un terzo
   rerun cieco).
2. **LEVA H-C2** (drop fast-out scalare, zero prefissi davanti):
   implementazione col predicato unico → A/B DA SOLA vs
   `hc2-criterio.out` → gate pieno (fixture 13+5+19 + batteria + corpus
   1417×2 + **coppia WP bimodale**, che salda anche il debito S-103).
3. **H-D SiteTag** (residuo≡0) → attribuzione della 1×32,0 B → leva solo
   se la cifra la giustifica dal SUO criterio pre-registrato.
4. **Generator-in-cycle**: decidere fedeltà (container+descend / birth-
   track) o assenza consapevole — la fixture rossa arbitra; catalogare.
5. **Igiene (timebox ½)**: banda tra-sere punto 3/3 · tavola bi-regime
   calls corretta ×½ · censimento gc_note calls (~5/chiamata) ·
   21,2% run_loop (se il concilio la ordina).

Pre-flight S-104: pin phpr **f45a5d199ab34132** @ HEAD (fa fede HEAD) ·
server **31aa7c2e** gradato · corpus 1417 per NOME ×2 SUL PIN · default
flag-ON · debug/ si rigenera: rimuoverla · **NON lanciare run pesanti se
`wp103-harness/peak-ab-out/ab.done` manca**.
