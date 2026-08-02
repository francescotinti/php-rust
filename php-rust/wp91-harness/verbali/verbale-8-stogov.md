# VERBALE — Dmitry Stogov, sedia 8, Concilio WP-91

**VERDETTO: CON EMENDAMENTI** — le 18 fixture sono un nucleo sano ma il
contratto ha una lettera REFUTATA dall'oracle (by-ref), il pin .out è un
bersaglio byte-impossibile per costruzione, e la sede lowering-only
diverge sui condizionali. Tutte le prove rieseguite dal vivo su 8.5.7.

## Q1 — Copertura: 11 BUCHI, pinnati dal vivo

Coperte: r1 (n1/p1), r3 (n3/p3), r6/r7/r9 (n6/n7/n9), tentative/RTWC/never
(p5/p6/p7), esenzioni dirette (p8/p9). Buchi (oracle rieseguito, stdout+rc):

1. **r2 by-ref — LETTERA DEL CONTRATTO REFUTATA**: «by-ref tipo ESATTO» è
   FALSA. `int &$x`→`int|string &$x` = **alive/0** (widening LEGALE);
   narrowing fatal («…must be compatible with P::m(string|int &$x)» —
   nota l'ordine canonico Zend dell'union nel messaggio); &-mismatch fatal
   in ENTRAMBE le direzioni (togliere E aggiungere &). Esatto = la
   REF-NESS, il tipo resta contravariante.
2. r2 required-grows: `m($a)`→`m($a,$b)` fatal «Declaration of C::m($a,
   $b) must be compatible with P::m($a)» — nessuna fixture.
3. r2 variadic assorbe: `m($a,$b,$c)`→`m(...$rest)` alive/0 — nessuna.
4. r4 grafia `?int` vs `int|null`: rc=0 — p4 testa il subset, non
   l'equivalenza di grafia.
5. r1 mixed=TOP narrowing (`mixed`→`int` alive/0) e **void esatto**
   (`void`→`int` fatal) — nessuna fixture.
6. r5 direzione inversa: static→instance fatal con messaggio DIVERSO da
   n5: «Cannot make static method P::m() non static in class C».
7. r8 readonly: fatal «Cannot redeclare readonly property P::$x as
   non-readonly C::$x» — nessuna fixture.
8. r8 tipo omesso/aggiunto: figlio untyped su parent tipato = messaggio
   di n8; figlio tipato su parent untyped = messaggio DISTINTO «Type of
   C::$x must be omitted to match the parent definition in class P».
9. **Esenzione INVERSA ctor**: interface-ctor e abstract-ctor VENGONO
   controllati (fatal «Declaration of C::__construct(string $y) must be
   compatible with I::__construct(int $x)», idem vs abstract). p8 da sola
   induce a esentare TUTTI i ctor — trappola viva.
10. Timing assente dal set: tutte le 18 sono parent-first incondizionate
    (v. Q3, t1-t4).
11. Nome irrisolvibile: l'oracle FATALA («…C::m(): B must be compatible
    with P::m(): A» con B mai dichiarata, nome stampato com'è scritto) —
    lo «skip conservativo» dell'appendice è divergenza da dichiarare per
    NOME, non fedeltà.

## Q2 — Pin: braccio ancorato, bersaglio NO

Binchk: la lane probe è onesta su ciò che prova (binario+ricetta
persistono) ma NON ancora il comando della fixture: probe e braccio
usano flag DUPLICATE a literal (ds35-verify.sh:47 vs :65) — una flag
caduta alla :47 (la recidiva WP-89) passerebbe «ok-via-probe». Header
.out riga 2 nomina 2 flag su 3. **Il pin troncato NON basta ed è
byte-impossibile**: l'oracle emette stdout=4 righe (display «\nFatal
error:…») + stderr=3 (log «PHP Fatal error:  …», doppio spazio); `2>&1|
head -3` tiene SOLO il blocco stderr, ma phpr emette SOLO il blocco
display (n7: stdout byte-identico via od, stderr VUOTO). Un'
implementazione perfetta non matcherebbe MAI il .out attuale. Serve pin
integrale a canali separati.

## Q3 — Sede: DUE posti, non uno

Pinnato vivo: t1 (parent DOPO) e t4: persist fatala PRE-output (niente
«pre|»), plain post — lowering-time è GIUSTO per il set hoisted
(top-level incondizionate, stesso perimetro §3.3-ter). MA t2
(condizionale eseguita): fatal DOPO «pre|» su ENTRAMBI i bracci; t3
(condizionale non eseguita): exit 0 ovunque. Lowering-only fatala dove
l'oracle esce 0 — il caso peggiore. Sede = lowering per le hoisted +
bind-in-registry per condizionali/dinamiche (incluse include/eval).

## Q4 — A-DS47/KS-DS-90-2

Sweep del catalogo eseguito: tutte le citazioni persist (§3.3/ter/quater)
ancorano a ds40-verify o ds35-verify, entrambi con binchk fail-closed —
nessun orfano. Residui: flag-vector condiviso probe/braccio e header a
3 flag (v. Q2).

## Emendamenti

- **A-DS48**: contratto r2 emendato — by-ref: ref-ness ESATTA, tipo
  contravariante (widening legale); messaggi con union in ordine
  canonico Zend.
- **A-DS49**: fixture v2 PRIMA del codice — 11 nuove per NOME (buchi
  Q1: byref×3, required-grows, variadic, grafia nullable, mixed, void,
  static-inversa, readonly, prop-omitted, iface/abstract-ctor,
  t1-t4 timing, nome irrisolvibile).
- **A-DS50**: pin v2 = stdout integrale + stderr SEPARATI; decidere per
  NOME se phpr modella la log-copy stderr o la dichiara a catalogo.
- **A-DS51**: sede a due posti (lowering hoisted + bind registry);
  irrisolvibili = fatal fedele t4 oppure divergenza a catalogo per NOME.
- **A-DS52**: ds35-verify v2 — PERSIST_FLAGS unico condiviso
  probe/braccio; header .out con le tre flag per NOME.

## Kill-switch

- **KS-DS-91-1**: implementazione che fatala su t3 (condizionale non
  eseguita) o anticipa t2 pre-output ⇒ REJECT.
- **KS-DS-91-2**: implementazione che esenta il ctor vs
  interface/abstract-ctor o fatala il by-ref widening ⇒ REJECT; gate di
  merge = fixture v2 per NOME, non solo le 18.
- **KS-DS-91-3**: pin fatal consumato come bersaglio byte-fedele se
  catturato `2>&1`+troncato ⇒ UNANCHORED.

Firmato: **Dmitry Stogov**, sedia 8, Concilio WP-91 — 2026-08-03.
