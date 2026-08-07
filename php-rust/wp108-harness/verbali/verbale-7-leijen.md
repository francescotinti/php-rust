# Verbale sedia 7 — LEIJEN (allocatore, footprint, census) — Concilio WP-108

## VERDETTO

H-A1 **NON refutata nel merito**: A/B 5/5, co-primario strutturale, parità
salda. Ma la sessione lascia **tre debiti census** che, non sanati, rendono
inaffidabile la prossima lettura. Nessuna refutazione capitale.

## Istruttoria

1. **Postura census del criterio H-A1**: `ha1-criterio.out` non nomina
   l'allocatore — né co-primario (H-D pretese 0,0000) né census-null
   dichiarato. Per una leva che elide dispatch/transiti l'omissione è
   *difendibile* (pila `Vec` a capacità ammortizzata) ma è rimasta TACITA;
   l'istogramma bl mostra pure `grow_one −1` non commentato. KS-LE-107-1
   non è stato violato (nessuna lettura census in S-106, l'unico atto — il
   reindex op_index N_OPS=187 — è dichiarato) ma nemmeno esercitato.
2. **Braccio `BinarySTDst`** (run.rs:1207-1233): zvalcensus e dcn!
   specchiano il tris ESATTAMENTE (note_slot_load_site + note_recv_load da
   LoadSlot; note_prop_val×2 e dcn lhs/rhs/dst-old da BinaryDst) — bene.
   scn! invece riporta la macchina NUOVA (giusto: il census conta ciò che
   accade): il tris valeva 3 op + 6 transiti (Push1, Len1, Elem2, Pop2),
   il fuso 1 op + Pop1 ⇒ **−5 transiti/iter**. Le attese statiche S-102
   (23 transiti/iter su prop) sono DECADUTE — e prop è 2° beneficiario
   nominato. Nessuno le ha ri-registrate.
3. **`GA_ARGPLACE_DECAY`** (memcensus.rs:1440-1450): chiamante gated
   `mem-census` (calls.rs:308-9) ✓; ma (a) static process-global in
   php-types come GA_ARITY — stessa specie del residuo R-5; (b)
   **`argplace_decay_hits()` non ha ALCUN lettore**: non nel dump
   zvalcensus, nessun gate «atteso 0» — D-12 diceva «rumoroso», ma il
   rumore esige un ascoltatore: oggi è un commento con atomics; (c) nella
   build di parità release il degrade a NULL resta SILENZIOSO (né assert
   né contatore): D-12 copre debug e census, NON il pin — va detto.
4. **`GA_ARITY` ha ora DUE siti alimentanti** (calls.rs:327 bind_params +
   run.rs:2637 direct-bind) mentre docstring ed etichetta del dump dicono
   «vista da bind_params»: il contratto del contatore è derivato. Il rerun
   arità D-5 senza manifest a due siti leggerebbe un istogramma che non è
   più quello di S-105 G2.
5. **Footprint**: nessun peak in S-106 = corretto (rotta sospesa). Ma i
   peak citati (1942/1990 MiB) sono di S-102: 4+ leve fa. Rischio
   decadimento = attribuire domani un Δ-peak attraverso leve mai misurate.

## Emendamenti

- **R-LE-108-1** — Ogni criterio di leva dichiara la POSTURA census per
  NOME: co-primario, O census-null atteso motivato, O non-applicabile.
  Il silenzio di ha1-criterio.out non fa precedente.
- **R-LE-108-2** — Attese stack-census S-102 DECADUTE: ri-registrazione
  per NOME sul census-build del pin corrente PRIMA della prossima lettura
  census pila (R-3 vige); ogni confronto col 23/iter storico = VOID.
- **R-LE-108-3** — R-5 ESTESO a GA_ARGPLACE_DECAY; riga nel dump
  zvalcensus + gate «atteso 0» in fixture census nello stesso commit;
  dichiarare che il pin di parità resta silenzioso sul degrade.
- **R-LE-108-4** — Contratto GA_ARITY corretto (due siti) o famiglie
  separate; il manifest del rerun D-5 dichiara la composizione.
- **R-LE-108-5** — Alla riapertura footprint: primo atto = re-baseline
  peak sul pin corrente; 1942/1990 = cifre STORICHE non citabili.

## Azioni

- **A-LE-108-1**: dump+gate argplace_decay (con R-LE-108-3).
- **A-LE-108-2**: ri-registrazione attese stackcensus (arith E prop).
- **A-LE-108-3**: feature-gate statics GA_ARITY/GA_ARGPLACE_DECAY senza
  churn del pin (R-5 esteso).

## KS

- **KS-LE-108-1** — Un contatore senza lettore non è un dente: ogni
  contatore census nasce con la sua riga di dump E il suo gate nello
  stesso commit, o non nasce.

## Ordine S-107

Sequenza 1-5 **condivisa** con due innesti: punto 1 (hit/miss census)
incorpora il manifest a due siti (R-LE-108-4) e imbarca A-LE-108-2 nello
STESSO run census (D-5, nessuna finestra nuova); punto 5 allarga R-5 a
GA_ARGPLACE_DECAY. §3.15 in testa: nessuna obiezione census.
