# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-172 = LEVA L-SL2 «forma sigillata Long» fetta 2 = PROP PROMOSSA (pin
NUOVO s172)**: P1 = probe sigillato nel bigramma fuso PropGetSlotRecv+BinaryTCPropSetPop (guardie
IC get/set VERBATIM, long_arith_i64, store in place su prop Long via get_slot_mut, miss ⇒ sentiero
fuso storico) + P2 = BinarySTDst con catena i64 (stack.last Long, slot Long), pop + store in place,
corpo esatto `binary_st_dst_slow` #[cold]; zero unsafe. A/B R=5 tre bracci: **prop-dq 72,60→57,53
(P1, D +15,07) →52,87 (P1+P2, D +19,73; 3,78× l'oracle contro 5,19×)**, attese [8;20]/[12;30]
CENTRATE, C−B +4,67 = direzione (C mai alternato: nessuna cifra a P2), guardia arith-dq −0,3/−0,5
(nessuna regressione); promozione rc=0 al tentativo 2 + ripresa; **micro prop 5,2→3,8, arr 3,2→3,0**
· az.rev. S-171 CHIUSE: mutante abortivo rc=0 (M1 rompe 9 righe, M2 13, fuori dominio intatte),
fx-sl1 estesa e CORRETTA (slot by-ref residui = metà presidio vuoto), dump ops archiviato, nota
census · leve: 1 · incidenti: 2 (#1 build C==B al byte per mtime di `git archive`; #2 push manuale
in collisione col push di pin-phpr.sh) · revisione S-172 (lente PROCESSO, wp172-harness/revisione.md):
REGGE CON RILIEVI · sessioni senza misura: 0.

## Scoreboard (pin NUOVO s172 phpr 5f2dff7d17ebed79 + server 812f7962952da67f)
**arith 2,7 · prop 3,8 · calls 4,7 · str 4,1 · arr 3,0 · re 2,5** · mc2 ~155 / mc3 181 (non
rimisurati) · arith-dq 23,4 vs 8,64 · prop-dq 52,87 vs 14,00 (conferma post-pin +20,07 5/5) ·
dispatch 1,75/op · **WP t18 1,769 in banda [1,738;1,799] (compatibile, nessun claim; t17 1,776;
leg1 segnalata ictx, N=5 pulite) · ORM t18 «ORM_T18»** · corpus 1412×2 · batteria 1748 · denti: run.rs 7200
(cap dichiarato +109) · mod.rs 25909 · host.rs 7726 · coda CI: 12 job trattenuti dal lock (potare a HEAD in chiusura).

## §S-173 — ordine
1. Coppia t18 LETTA in S-172 (WP compatibile; ORM «ORM_STATO»): nessuna istruttoria; resta dovuta la
   ri-fondazione PRE-registrata della banda sentinella ORM (4,82 e 4,95 = estremi osservati).
2. **Az.rev. S-172** (revisione, PRIMA di generalizzare): (a) mutante abortivo su P1 e P2 contro
   fx-sl2 (copia di s172-mutante-sl1.sh: `.map(|v| v.wrapping_add(1))` sul sealed di P1 e sul fast
   di P2; righe rotte NOMINATE per etichetta, fuori dominio intatte); (b) A/B ALTERNATO B↔C prima di
   accreditare una cifra a P2 (oggi solo direzione); (c) PIN_REGISTRY dei bracci: `--braccio` registra
   HEAD, non il commit sorgente (c419f29a/59ca87fb): emendare pin-phpr.sh (argomento commit) ;
   (d) forme scoperte di fx-sl2: oggetto lazy, enum (senza warning), scope privato via `$this`
   (ThisProp*), `?float` e promozione costruttore per §3.30.
3. **Fetta 3 (prop, residuo 38,9 ns/iter vs oracle)**: census a forme del residuo — dispatch 14 +
   guardie IC/borrow (RefCell ×4 + ic.get ×2 per iterazione) + Sweep×2 + push/pop del Long fra
   PropGetSlot e BinarySTDst; candidate: peephole PropGetSlot+BinarySTDst (niente pila) e guardia IC
   a un solo borrow; criterio proprio (giudice prop-dq, guardie arith-dq/arr a sola regressione).
4. **Generalizzazione (calls 4,7 → str 4,1)**: forme DIVERSE (frame: Call/BinarySS/Ret; stringhe:
   ConcatNConst + CallBuiltin substr): census dal dump PRIMA, mock magro SOLO se il corpo non è
   leggibile, criterio per categoria, mai aggregato.
5. xctrace #2 (az.rev. S-170, APERTA): pin/B/m13 su dq SOLO con Data ≥20G e purge ktrace in
   `trap EXIT`; `xctrace record` crashato su B: provare `--time-limit`/attach.
6. Quesiti residui: c0 positivo · Sweep/iter 2,9 · (b) T2/A2 · census server (22° slitt.) · ratifiche §3.

## Aperture per NOME
mutante P1/P2 · alternanza B↔C · PIN_REGISTRY bracci · fetta 3 prop (peephole, borrow IC) · calls
→ str · assign-form (BinarySCSC+BinaryDst non coperta da L-SL1) · §3.30 perimetro (default
tipizzati) · xctrace pin/B/m13 · residuo slot 2,75/op · tupla guard · Sweep/iter · F1/F2 (SOSPESE)
· autoload statiche · sonda strmap · banda sentinella ORM (4,82) · gamba server census · §3.28 ·
§3.29 · §3.27 · §3.26 · §3.25 · §3.24+§3.23 · slot-load · §3.22 · depr. float→int · warning ×2 ·
div. RMW · objmap → GC · evalcls 316,9× · refl 42,4× · re +2 · get_gc · latin1 · dbal 10 nomi.

## NON riproporre (i veti restano)
**S-172: push a mano mentre una catena con pin-*.sh è in corso (collisione sul ref → rc=1) ·
`git archive` in un target cargo condiviso senza touch dei sorgenti (mtime del commit ⇒ nessuna
ricompilazione: C==B al byte) · fixture single-shot per un probe con IC (IC fredda = presidio vuoto:
ogni forma in loop ≥2) · righe di fixture dopo un `unset($r)` sulle stesse variabili (slot Ref
residuo) · attese `a || b && c` senza graffe · `git show <sha>:crates/...` (radice git = cartella
padre: usare `:./`).** S-171: lock senza TOKEN · attese/lanci con `phpr` nell'argv (symlink neutri)
· xctrace senza trap EXIT e Data ≥20G · attesa bl «+2..+4» per corpi outlined · build/run durante
la coppia · fixture con diag nel gate bilaterale (`-d log_errors=0`). S-170: driver/pavimento
senza `[ -e ]` · patch senza `--relative` · wrapper con path non quotati · `rm -rf` di target
intere · promuovere mock unsafe. Trasversali: NaN-boxing/fn-table/arena (⚖️) · BOLT/PGO · pin senza
collaudo · rc da pipe · promozione sotto banda · cifre composte tra binari diversi.
**Riscritto** 2026-09-08 notte (chiusura S-172; storia in `sessions/` · `gaps/`).
Pre-flight S-173: pin phpr **s172 5f2dff7d**17ebed79 + server **812f7962** (SOLO via pin-*.sh; stash
bracci `phpr-s172-sl2-B/C` NON pin) · Data ≥10G (≥20G se xctrace; il corpus-gate consuma ~5G
TRANSITORI su Data: rientrano a fine gate) · MySQL wp8 con l'elenco · uploads sotto guardia ·
corpus 1412 · lock misura da CREARE COL TOKEN `s173` · CI: coda potata a HEAD in chiusura, runner
in disk-low finché Data <10G · coppia dovuta SOLO se il pin cambia · lettura: REGOLE.md → QUI →
wp172-harness/s172-verdetto.out + revisione.md → s172-leva-verdetto.out → s172-criterio.md →
wp171-harness/s172-azrev-verdetto.out → WP_SESSION_172 → PERF_MAP.
