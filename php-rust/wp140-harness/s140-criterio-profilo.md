# S-140 — criterio PROFILO SUITE ORM (pre-registrato PRIMA del run)

Oggetto: profilo monobinario phpr (pin s138 fa17dabd) sulla suite doctrine/orm
— INDIZIO quantificato per FAMIGLIE nominate, MAI cifra di tempo (il sampling
perturba). Decide la leva S-140: churn vs dim-read (REPERTO S-139).

1. **Strumento**: `sample` 2×18 s per replica (skip 4 s bootstrap), 2 repliche
   (run interi separati, workspace ri-untarrato); quota = top-of-stack cumulata
   su TUTTI i campioni della replica; base script DICHIARATA: s126-orm-profile.sh.
2. **Famiglie (lista CHIUSA, primo match vince, pattern sul simbolo)**:
   vm_inline=`run_loop` (corpo interprete INLINED, multi-op: NON attribuibile,
   quota riportata come osservazione) · churn_zval=`Zval`∧(`clone`|`drop`) ∨
   `drop_in_place` · gc=`gc_` ∨ `collect_cycles` ∨ `sweep` ∨ `demote` ·
   alloc=`mi_malloc`|`mi_free`|`_mi_`|`malloc`|`free` · map=`PhpArray`|
   `hashbrown`|`RawTable`|`Hasher`|`sip`|`KeyIndex` · prop_dim=`prop_`|
   `field_`|`slot_of`|`resolve` · calls=`enter_callee`|`bind_params`|
   `recycle_frame`|`Frame`|`dispatch_instance_call` (osservativa, FUORI dai
   macro-canali) · memops=`memmove`|`memcmp`|`memcpy`|`memset` (copia:
   attribuzione ignota, FUORI dai macro-canali, riportata) · str=`PhpStr` ·
   compile=`compile`|`parse`|`lower`|`mago` · refl=`reflect` · resto=other.
   Lista calibrata a secco sui campioni STORICI S-126 (pin s125) PRIMA della
   registrazione; la decisione viene SOLO dal run nuovo @ s138.
   Quota = sui campioni top-of-stack ATTRIBUITI (il tail <5 è collassato da
   `sample`, dichiarato).
3. **Macro-canali**: CHURN = churn_zval+gc+alloc+map · DIMPROP = prop_dim
   (solo simboli outlined; vm_inline e memops FUORI da entrambi). Se
   vm_inline > 40% la risoluzione del profilo è dichiarata LIMITATA e la
   decisione richiede dominante attribuita ≥ 20% comunque.
4. **Decisione PRE-REGISTRATA**: leva → CHURN (sotto-canale = famiglia max
   dentro CHURN) se quota(CHURN) ≥ 2×quota(DIMPROP) E ≥ 20%; leva → dim-read
   se quota(DIMPROP) ≥ quota(CHURN); zona grigia → NESSUNA decisione dal solo
   profilo: istruttoria supplementare dichiarata a verbale.
5. **Validità**: pin hash esatto; parità suite per NOME vs baseline 16
   (`wp125-harness/orm-baseline-failnames.txt`) su OGNI replica — gate a
   NOMI, nessun cmp byte-id vs oracle (dry-run az.rev. #4: il catalogo
   divergenze non è toccato perché non c'è confronto byte con l'oracle);
   rank top-3 famiglie STABILE tra le 2 repliche, altrimenti la decisione
   scatta solo con quota dominante ≥ 25% sulla replica pulita.
6. **Conseguenza replica DISTURBATA (az.rev. #3, pre-registrata)**: replica
   SEGNALATA se `pgrep rustc|cargo` è vivo a inizio o fine di una sua finestra
   sample → ESCLUSA dal rank; 1 sola replica pulita ⇒ soglia 25% (p.5);
   0 pulite ⇒ profilo NULLO, si ripete. N minimo per decidere = 1 replica pulita.
7. **Finestra protetta (az.rev. #2, dichiarato QUI)**: NIENTE push durante la
   finestra; il runner CI onora `phpr-measure.lock` a inizio job
   (`ci/ci-runner.sh` r.95 `quiet_wait`, pattern `harness/s1xx-` incluso);
   la finestra apre SOLO a job CI corrente DONE nel feed e nessun
   rustc/cargo/rust-analyzer vivo; lock creato dalla SESSIONE, lo script
   lo VERIFICA soltanto (veto lock-con-trap-altrui).
