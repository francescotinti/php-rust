# S-167 criterio FETTA 0 — sonda arith a 5 bracci (⚖️ concilio S-167, PRE-registrato PRIMA di ogni esecuzione)

## Oggetto e cifre di riferimento
Gap arith: pin s166 ~46,5-46,8 ns/iter vs oracle ~8,6 ⇒ **38,2 ns da nominare**.
Giudice base: `wp164-harness/arith-dq.php` N=250.000.000 (tick 0,04 ns/iter),
ns/iter=(med raw−floor)/N, R=5 interleaved, finestra quieta (lock+quiescenza+
sentinelle LS). NESSUN codice di leva prima del verdetto di questa sonda.

## Bracci
(a) **A/B PHPR_REG_LOWER on/off** sul PIN (stesso binario, env; ON=default
    S-100): D_ab = costo che il reg-lowering GIÀ paga (non ricontarlo).
(b) **gemello data-stride**: driver derivato `arith-stride.php` (SOLO 14 slot
    dummy dichiarati PRIMA del loop ⇒ $s/$i su linee di cache distinte; corpo
    del loop IDENTICO, dichiarazioni fuori dal conteggio). |D| vs arith-dq
    ≤ 0,5 ns/iter ⇒ canale pila/D-cache REFUTATO (KS-LE-167-1 parte 1).
(c) **counters CPU bilaterali** (xctrace 'CPU Counters', per-processo, su
    ENTRAMBI i motori): branch-miss/iter, L1I-miss/iter, IPC.
    **MUTANTE OBBLIGATORIO prima d'ogni lettura**: `arith-branchmut.php`
    (branch data-dipendente pseudo-casuale) DEVE mostrare branch-miss/iter
    ≥10× di arith-dq su phpr — se lo strumento non lo vede, (c) è
    INDISPONIBILE dichiarato e il verdetto si regge su (a)(b)(d)(e).
    Firma front-end: branch-miss/iter phpr ≥2× oracle O L1I-miss/iter ≥2×.
(d) **drop-census dcn!/iter** (build probe zval-census su tree==pin, target
    dedicato, run sequenziale): conteggio drop per specie per iter.
    **MUTANTE**: `arith-dropmut.php` (+1 stringa temporanea/iter) deve
    spostare il conteggio drop-stringhe di N ESATTO.
(e) **tetto-fuso** (KS-BAK-167-1): SOLO se (c) firma front-end — handler
    straight-line del corpo del driver dietro env-flag, corpus verde; il suo
    mini-criterio si PRE-registra al momento (appendice a questo file, PRIMA
    del run). Se nemmeno il fuso scende ≤3× ⇒ R1 muore (delibera R4).
+ **mock sottrattivi F0** (team-meccanismo, per la chiusura): fino a 3 build
    da tree==pin con patch MINIME dichiarate — (m1) consts-predecode,
    (m2) BinOp «cotto», (m3) hoist `frames[top]` — micro-judged R=5 vs pin,
    braccio via pin-phpr.sh --braccio, disasm bl-count, copia-gate-v2 sugli
    script derivati. I mock MISURANO (non promuovono): REGOLE timebox.

## Verdetto (meccanico)
- **Chiusura**: Σ canali nominati (a)-(e)+mock ≥85% di 38,2 (obiettivo 70%
  nominato ai TRE canali pila/front-end/interno-handler; ≥60% = firma minima).
- **GO-leva** (per F1/F2): canale dominante ≥15 ns/iter.
- **Kill**: mock <10 ns nominati ⇒ R1 esaurito (delibera R4) · stride muto E
  counters ≈ oracle ⇒ campagna non parte (KS-LE-167-1) · chiusura <90% dopo
  2 sessioni ⇒ R4 (Gregg; il gate ≥90% vale PRIMA della prima leva vera).
- Esiti SOLO da file (`f0-out/*.rc|.out`); ogni numero col suo rumore drop-1;
  quantizzati ri-risolti a tick ≤ soglia/4; verdetto in s167-f0-verdetto.out.
