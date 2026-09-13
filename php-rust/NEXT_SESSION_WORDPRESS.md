# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-174 = LEVA «Sweep-in-op» AMMESSA a criterio-bis, NON promossa (Data 4G)**: B =
predicato inerte di `Op::Sweep` estratto in `sweep_idle` (un solo testo, il handler lo richiama) e
`sweep_skip_next` interrogato a FINE op dai soli sentieri in place su Long (BinarySCSCDst, BinarySTDst,
peephole P3, probe P1/P4) ⇒ uno Sweep che non farebbe nulla si scavalca senza dispatch; C = B + back-edge
fuso (IncDecSlotJmp → CmpJmpSC con `long_cmp_i64` verbatim su slot Long/const Int); zero unsafe; spente
sotto census. A/B R=5 a QUATTRO bracci ruotati (A=pin, Z=gemello del pin, B, C): **corsa 1 rc=4** (arith-dq
bersaglio: C +3,64/+3,56 5/5 ma < pavimento 4; prop-dq guardia: C [5,13;5,20]) ⇒ **criterio-bis** (prop-dq
co-bersaglio, attesa [2;6] già dichiarata, commit PRIMA del lancio) ⇒ **corsa 2 rc=0: prop-dq C = CIFRA
[5,07;5,60] (2,95×→2,60×), arith-dq C +3,52/+3,48 direzione (2,82×→2,41×), C−B +1,96 direzione, nessuna
regressione ⇒ AMMESSA braccio C** (stash phpr-s174-sw-C b6c4b5876971d76d @883cb598; B 56dc1af8 @9c9e6a61).
Mutante MS/MC: corsa 1 rc=1 (attese a loop non discriminanti; p1-loop idempotente) → corsa 3 (loop con echo, coppie per sito) → corsa 4 rc=0: MS 11/11 ROTTE = i 5 siti provati (BinarySTDst, P3, P1, P4, BinarySCSCDst) coi controlli p3-site-ctl/p1-site-typed INTATTI, MC 29/29 ROTTE, nessuna presa fuori dominio; lezioni: `$l = $l + (…)` NON è BinarySCSCDst (serve `+=`), classe con prop tipizzata = probe P1/P4 mai preso · az.rev. S-173 TUTTE chiuse (banda tra run e |A−Z| nei .out, mediane appaiate,
0,94 mai più importato, decomposizioni = ipotesi, -each nominati) · gh-status-sync fatto (numeri identici,
data/pin S-173) · leve: 1 · incidenti: 1 (copione bis: RC inglobato da un commento; sw2.rc da ri-analisi
dichiarata) · revisione S-174 (lente SEMANTICA): REGGE CON RILIEVI (wp174-harness/revisione.md): il codice regge per costruzione (un solo predicato letto DOPO l'effetto, back-edge verbatim); rilievi (1) artefatti fx-sw1 in ab-out erano v1, (2) MS non provava forme a loop né i siti P1/P4/BinarySCSCDst, (3) criterio-bis: regola scelta a dati visti, arith-dq resta SOLO direzione. Replica S-174: (1)(2) CHIUSE in sessione — fixture v4 (loop con echo nel corpo, coppie per SITO, forma esatta di arith-dq `$l += …`, classe PLAIN P2: la P tipizzata è fuori perimetro) bilaterale oracle==pin==B==C in ab-out; mutante corsa 4 rc=0 (MS 11/11 ROTTE coi 5 siti, controlli p3-site-ctl e p1-site-typed INTATTI; MC 29/29); (3) accolto. · sessioni senza misura: 0.

## Scoreboard (pin INVARIATO s173 phpr da4921a52eba0187 + server 2d2adc4549a820e0)
**arith 2,8 = · prop 3,0 = · calls 4,8 = · str 4,1 = · arr 3,1 = · re 2,5 =** (micro non rimisurate) ·
**candidato C: arith-dq 24,04→20,40 / 24,44→20,92 (2,35-2,41×) · prop-dq 41,33→36,13 / 42,07→36,47
(2,58-2,60×)** · coppia t19 WP 1,772 · ORM net [6,988;7,012] (pin invariato: non dovuta) · corpus 2655
(1412 fail congelati) · batteria: non rieseguita (nessuna promozione) · denti: run.rs 7361 (cap alzato
DICHIARANDO 7274→7361 in 9f349b0f, +87 netti) · CI: lock s174 ⇒ job in skipped-busy dalla sera del 13.

## §S-175 — ordine
0. **PRE-FLIGHT BLOCCANTE: Data ≥10G** (oggi 4-5G; nessun peso phpr da potare: i pesi sono utente —
   Caches Google 2,2G, com.openai.codex 1,9G, Claude vm_bundles 9G, Parallels 5G). Senza spazio la
   promozione NON parte (build canonica su target potato ~5G + batteria + corpus-gate).
1. **Promozione del braccio C** (`wp174-harness/s174-promozione.sh b6c4b5876971d76d C`, PROMO_SP su
   /private/tmp/phpr-s175-promo; PRE: s174-mutante-sw-verdetto.out rc=0, stash s174-sw-C, crates/ pulito):
   build ricetta → identità a contenuto (soli LC_UUID/firma) → batteria (dente 7361) → re-hash → pin-phpr.sh
   s175 → corpus-gate ZERO flip → fixture chain + 20 gate (fx-sw1 NUOVA) → micro R=5 → conferma post-pin
   arith-dq vs stash s173 → ORM 16 nomi → hk 0E/0F → pin-server.sh s175. Poi **coppia WP+ORM DOVUTA**
   (copie dichiarate S-173 col pin s175; sentinella ORM 4,94 = estremo: fuori banda ⇒ voce riaperta).
2. **Az.rev. S-174** (dalla revisione, PRIMA di nuove leve): (a) gamba «pressione GC» del predicato: blocco con `gc_status()['runs']` letto DENTRO un loop con radici ≥ bound + mutante sulla 3ª clausola, atteso ROTTO; (b) liste INT del mutante = solo blocchi discriminanti (regola scritta S-174, da verificare a ogni copia); (c) conferma post-pin su arith-dq = attesa SOLO SEGNO (D_C 3,5 < 4); (d) scoreboard/roadmap: arith-dq «solo direzione» (fatto).
3. Residuo trasversale dopo C: sul giudice arith-dq restano 20,4 vs 8,68 (2,35×): dispatch di 3 op/iter +
   predicato Sweep (3 load + 2 confronti) + corpi; candidate: (a) predicato Sweep in un solo flag «gc idle»
   mantenuto dai siti che notano/demotono (mock magro prima); (b) census dei bigrammi «op in place + Sweep»
   fuori dai giudici (WP/ORM: quanti statement finiscono con un op fuso?) per stimare la portata reale.
4. Perimetro typed (invariato da S-174 p.4): census WP/ORM di `$o->x = $o->y OP C` su classi typed PRIMA di
   una fetta 4 (IC set solo su prop int non readonly).
5. Solo dopo: calls 4,8 (census Call/BinarySS/Ret, args-Vec) → str 4,1 (ConcatNConst, substr).
6. xctrace #2 SOLO con Data ≥20G e purge ktrace in `trap EXIT`.
7. Quesiti residui: c0 positivo · (b) T2/A2 · census server (24° slitt.) · ratifiche §3 · divergenza
   dtor-in-dtor (ordine del distruttore di un temporaneo dentro un distruttore: pin ≠ oracle, pre-esistente,
   vista in fx-sw1 e tolta dal perimetro — da catalogare per NOME in PHPR_DIVERGENCES).

## Aperture per NOME
predicato Sweep → flag gc-idle · census «op in place + Sweep» su WP/ORM · dtor-in-dtor (catalogo) · IC set
typed (fetta 4) · census typed WP/ORM · calls → str · assign-form (BinarySCSC+BinaryDst) · §3.32 · §3.30 ·
xctrace pin/B/m13 · residuo slot 2,75/op · tupla guard · F1/F2 (SOSPESE) · autoload statiche · sonda strmap
· banda sentinella ORM (4,94) · gamba server census · §3.28 · §3.29 · §3.27 · §3.26 · §3.25 · §3.24+§3.23 ·
slot-load · §3.22 · depr. float→int · warning ×2 · div. RMW · objmap → GC · evalcls 316,9× · refl 42,4× ·
re +2 · get_gc · latin1 · dbal 10 nomi · pavimento 4 ns/iter su loop corti (quesito di metodo, non emenda).

## NON riproporre (i veti restano)
**S-174: commenti in coda a righe con `;` (inglobano l'assegnazione che segue) · collaudo del solo blocco
python di una copia dichiarata (si collauda la shell intera) · floors passati come UNA stringa (zsh non fa
word-splitting) · skill con `model:` nel frontmatter a metà sessione · «banda-layout» da ricostruzione della
stessa sorgente (hash identico) · promuovere una guardia a bersaglio SENZA rerun completo a criterio
committato.** S-173: attese «intatte» senza reset del dst · `perl -0pi` con `\|\|` · pulire ab-out/ durante
una build · copertura del mutante senza liste a-verdetto · cifra sotto soglia anche con 5/5. S-172: push a
mano nelle catene pin-*.sh · `git archive` senza touch · fixture single-shot con IC · righe dopo `unset($r)`
· `a || b && c` senza graffe · `git show <sha>:crates/...`. S-171: lock senza TOKEN · `phpr` nell'argv delle
attese · xctrace senza trap/Data · build/run durante la coppia. S-170: driver senza `[ -e ]` · patch senza
`--relative` · path non quotati · `rm -rf` di target intere · mock unsafe. Trasversali: NaN-boxing/
fn-table/arena (⚖️) · BOLT/PGO · pin senza collaudo · rc da pipe · promozione sotto banda · cifre composte.
**Riscritto** 2026-09-13 (chiusura S-174; storia in `sessions/` · `gaps/`).
Pre-flight S-175: pin phpr **s173 da4921a52eba0187** + server **2d2adc4549a820e0** INVARIATI (stash bracci
`phpr-s174-sw-B/C` NON pin) · **Data ≥10G (bloccante)** · MySQL wp8 con l'elenco · uploads sotto guardia ·
corpus 1412 · lock misura da CREARE COL TOKEN `s175` (il lock s174 va rimosso in chiusura S-174) · CI: job
in skipped-busy finché il lock vive · lettura: REGOLE.md → QUI → wp174-harness/s174-verdetto.out +
s174-sw2-verdetto.out + revisione.md → s174-criterio.md + s174-criterio-bis.md → s174-mutante-sw-verdetto.out
→ WP_SESSION_174 → PERF_MAP.
