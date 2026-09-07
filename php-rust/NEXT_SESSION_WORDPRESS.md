# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-171 = LEVA L-SL1 «forma sigillata Long» fetta 1 PROMOSSA
(pin NUOVO s171)**: BinarySCSCDst = catena i64 GENERICA (arm Long di binary_fast verbatim,
un match per op) + store in place + corpo esatto `#[cold]`; CmpJmpSC a bool diretto; IncDec
`checked_add` in place; zero unsafe. A/B R=5 tre bracci: **dq 46,76→23,44 (D +23,32;
2,71× l'oracle)**, **e2 14,72→10,64 (+4,08)**, **B−m13 −1,04**: la forma generica
riproduce il tetto driver-shaped (attesa «≈17» della rev. S-170 caduta in direzione
favorevole). Gate pieni rc=0; **micro arith 5,4→2,7 = TAPPA ≤3× RAGGIUNTA su arith** ·
leve: 1 · incidenti: 1 (xctrace crash → ktrace 8,4G, Data 21→2G) + 3 difetti copione curati
· revisione S-171 (lente SEMANTICA, wp171-harness/revisione.md): REGGE CON RILIEVI —
fast path NON provato «preso» sulla fixture (serve mutante abortivo), typed-ref della
fixture nominale, forme non-driver senza dump, census cambia significato (dcn/slot_read).
· sessioni senza misura: 0.

## Scoreboard (pin NUOVO s171 phpr b360b2933eddfe18 + server b3ddaede545ba894)
**arith 2,7 · prop 5,2 · calls 4,8 · str 4,1 · arr 3,2 · re 2,5** · mc2 ~155 / mc3 181
(non rimisurati) · arith-dq 23,44 vs 8,64 · E2 10,64 vs 3,48 · dispatch 1,75/op ·
**WP/ORM: coppia t17 in corso al lancio di questa rotazione (pair-out/pair171-t17.done,
orm-out/rimisura.done; verdetti s171-pair-verdetto-t17.out / s171-orm-coppia-verdetto.out)**,
rif. precedenti WP 1,746-1,749 · ORM [7,023;7,053] · corpus 1412×2 · batteria 1748 · denti:
run.rs 7091 (cap dichiarato) · mod.rs 25909 · host.rs 7726 · coda CI: da potare a HEAD.

## §S-172 — ordine
1. **Esiti coppia t17** (se non letti in chiusura S-171): giudizio a mediana [1,738;1,799],
   ORM banda sentinella; una regressione FUORI banda blocca il p.3.
2. **Az.rev. S-171** (revisione, PRIMA di generalizzare): (a) mutante abortivo su B
   (`Some(r)→Some(r+1)` in BinarySCSCDst, `long_cmp_i64` negata) contro fx-sl1: righe
   rotte NOMINATE (dq100, bitops, lt-loop, dec-loop), revert al byte; (b) fx-sl1 ESTESA:
   typed reference VERO (`class C{public int $p;} $r=&$o->p; $r += …` + overflow →
   TypeError/float) e Shl con r∈[1,63) su l negativo; (c) dump ops di fx-sl1 con conteggio
   BinarySCSCDst/CmpJmpSC/IncDecSlotJmp per riga, archiviato; (d) nota census: il calo di
   dcn/slot_read su CmpJmpSC/BinarySCSCDst è NON-materializzazione (L-SL1), non meno lavoro.
3. **Generalizzazione a forme (REGOLA 8 → fetta 2)**: census S-164 delle categorie
   prop/calls/str: quali handler caldi hanno la stessa forma «Zval temporaneo + funnel
   Option<Zval> + clone/drop»; per OGNI categoria: criterio proprio (giudice = micro della
   categoria + guardie a sola regressione), mock magro PRIMA (come S-170) SOLO se il
   corpo non è già leggibile, poi leva safe con corpo esatto `#[cold]`; misura per
   categoria, mai aggregato. Candidata prima: **prop 5,2** (scarto maggiore dopo arith).
4. **xctrace #2** (az.rev. S-170, APERTA): pin/B/m13 su dq, SOLO con Data ≥20G, purge
   ktrace in `trap EXIT` (un record ≈8G anche se crasha); `xctrace record` è crashato su
   B («Trace/BPT trap»): provare `--time-limit` o template senza --launch (attach).
5. Quesiti residui: c0 positivo (kernel ILP-ricco) · Sweep/iter 2,9 · (b) T2/A2 · census
   server (21° slitt.) · ratifiche §3.

## Aperture per NOME
generalizzazione a forme (prop → calls → str) · mutante abortivo fx-sl1 · typed-ref vero ·
dump forme non-driver · census dcn/slot_read · xctrace pin/B/m13 · disco Data (10G
consumati da NON-progetto durante la promozione: capire in pre-flight) · residuo slot
2,75/op · tupla guard · Sweep/iter · F1/F2 (SOSPESE) · autoload statiche · sonda strmap ·
banda sentinella ORM (4,82) · gamba server census · §3.28 · §3.29 · §3.27 · §3.26 · §3.25 ·
§3.24+§3.23 · slot-load · §3.22 · depr. float→int · warning ×2 · div. RMW · objmap → GC ·
evalcls 316,9× · refl 42,4× · re +2 · get_gc · latin1 · dbal 10 nomi.

## NON riproporre (i veti restano)
**S-171: lock senza il TOKEN che il copione cerca (`s171`, non «S-171») · attese/lanci con
`phpr`/`php-server` nell'argv (quiescenza `pgrep -f` = falso positivo: symlink neutri) ·
xctrace senza `trap EXIT` di purge e senza Data ≥20G · attesa bl «+2..+4» per corpi
outlined (le chiamate ESCONO da run_loop: Δ negativo) · build/run di phpr durante la coppia
(quiete per gamba) · fixture con diag nel gate bilaterale (CLI oracle duplica su stderr con
log_errors: `-d log_errors=0`).** S-170: driver/pavimento senza `[ -e ]` e collaudo ·
patch senza `--relative` · wrapper con path non quotati · `rm -rf` di target intere ·
promuovere mock unsafe. Trasversali: NaN-boxing/fn-table/arena (⚖️) · BOLT/PGO · pin senza
collaudo · rc da pipe · promozione sotto banda · cifre composte tra binari diversi.
**Riscritto** 2026-09-07 sera (chiusura S-171; storia in `sessions/` · `gaps/`).
Pre-flight S-172: pin phpr **s171 b360b293**3eddfe18 + server **b3ddaede**545ba894 (SOLO
via pin-*.sh; stash bracci `phpr-s171-sl1-B` e `phpr-s170-*` NON pin) · Data ≥10G (≥20G
se xctrace; oggi il disco è sceso a 2G per consumo esterno al progetto: `du` di ~/Library
e /private/var PRIMA di misurare) · MySQL wp8 con l'elenco · uploads sotto guardia ·
corpus 1412 · lock misura da CREARE COL TOKEN `s172` · coppia dovuta SOLO se il pin
cambia · CI: coda potata a HEAD in chiusura S-171, leggere il feed · lettura: REGOLE.md →
QUI → wp171-harness/s171-verdetto.out + revisione.md → s171-leva4-verdetto.out →
s171-criterio.md → WP_SESSION_171 → PERF_MAP.
