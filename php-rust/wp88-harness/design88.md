# design88 — delibere S-88.0 p6 (Concilio WP-89): A-BB54 nested-guard + A-PP44 parser dispatch-union

Ordine Concilio WP-89 §Sintesi p6. NESSUNA riga di runtime cambia con
questo documento: A-BB54 e A-PP44 sono PRECONDIZIONI-macchina (guard
eseguibili) di usi futuri; l'attuazione degli script è parte della prima
sessione che ne consuma l'esito.

## A-BB54 — nested-guard macchina (precondizione della promozione SCOPED)

**Problema (Bak WP-89 Q4)**: lo scope «coppie ANNIDATE» della promozione
VERDICT-GRADE (delibera S-87.0 p6) vive in prosa: «simboli ⊆» regge sulla
COSTRUZIONE dei fixture, non su una verifica; i conteggi 317⊂377 NON
provano l'inclusione degli INSIEMI. Senza guard, l'appello allo scope è
invocabile ma non refutabile (KB-89-3: uso VERDICT-GRADE senza A-BB54
eseguito ⇒ VOID, declassa a MODEL-GRADE).

**Definizione operativa (vincolante)**: NESTED(S, L) ⟺
  (a) **subset per NOME, a macchina**: l'insieme dei NOMI di funzione
      dichiarati da S è ⊆ di quello di L. Estrazione: parse statico dei
      fixture (`function\s+([A-Za-z_][A-Za-z0-9_]*)`, classi/metodi
      inclusi come `Class::method`), MAI un confronto di cardinalità.
  (b) **witness in-band**: nel raw della coppia, floor_inc(S, ord=2) ≤ ε
      NOMINATO (collaudo hello⊆pad85: ε = 1.040 B osservato) E
      Δfloor(ord1→ord2) = costo del set condiviso ESATTO (collaudo:
      996.838 B, VINV al byte).

**Forma attuativa**: `nested-guard.pl <small.php> <large.php> <raw>`
  - exit 0 = NESTED verificato (subset + witness); exit ≠0 = coppia NON
    annidata ⇒ ogni scomposizione additiva su di essa resta MODEL-GRADE;
  - selftest che MORDE: (i) coppia con funzione extra nel piccolo ⇒
    refuse; (ii) witness fuori ε ⇒ refuse; (iii) coppia di collaudo
    hello⊆pad85 ⇒ pass.

**Consumo**: ogni verdetto che cita la scomposizione net(ord2) =
net_own − shared in grado VERDICT deve eseguire il guard e riportarne
l'esito in-band (classe A-SK47: flag di pulizia a monte).

## A-PP44 — parser condiviso dispatch-union (classe reqns-guard.pl)

**Problema (Pedersen WP-89 Q4)**: la riga `tag=worker_dispatch thr=N
count=M arm=union` (A-PP39, emessa su stderr nei build NON-census) è oggi
un dente SENZA consumatore (classe A-AH41): nessun verdetto la parsa; i
verdetti union futuri (VW123/VABBA) senza parser restano envelope
(KS-PP-88-2/KS-PP-89-3).

**Forma attuativa**: `dispatch-union-guard.pl <stderr-raw> <W> <nreq>`
  - parsa TUTTE le righe `tag=worker_dispatch .* arm=union`;
  - esige thr-set == {0..W-1} ESATTO (unicità + completezza, mai la sola
    cardinalità — stessa lezione A-PP42/KS-PP-89-2);
  - esige Σ count == nreq e count uniforme == nreq/W dove il protocollo
    lo garantisce;
  - selftest che MORDE: thr duplicato ⇒ refuse; thr mancante ⇒ refuse;
    somma sbagliata ⇒ refuse; mappa esatta ⇒ pass.

**Precondizioni di catena**: lo stderr union va catturato come RAW di
campagna (filename con attempt=, mai riusato, ledgerato) — senza raw
committato il parser non ha oggetto e il claim resta envelope.

**Consumo**: precondizione ESEGUITA di ogni VW123/VABBA sul braccio
union; l'esito entra nel verdetto come flag (KS-PP-89-3).
