# Team «flip-contratto» — Concilio WP-101 (sedie 1 Hoare, 2 Matsakis, 4 Hejlsberg)

Tema: contratto di modo di `PHPR_REG_LOWER` e specifica del flip a default (bozza §S-100).

## Convergenze (unanimi)

1. **Contratto di modo PRIMA di ogni riga del flip.** `enabled()` è presence-based (`is_some()`: anche `=0` ACCENDE) — trappola già latente. L'opt-out post-flip è **value-parsed e nominato** (forma proposta da Hoare: assente→ON, `=0`→OFF); vietato riusare la presence o invertirne il significato (KS-MA-101-1). Col contratto nuovo vanno ri-derivati: i due bracci del dente anti-putenv, la premessa M5 (`reg_lower.rs:597`, pena cifra 1726/0 non valida — KS-HO-101-3), i tre launcher, i bracci del funnel/`s99-corpus-gate.sh` (rischio falso verde stesso-modo = forgia silenziosa, R1 Hejlsberg). Flip senza contratto = VOID (KS-HO-101-1, KS-MA-101-1, A-HE-101-1).
2. **Bit-identità = SOLO diff del dump, mai timing.** "5,43→5,44 ⇒ bit-identico" è inferenza invalida (A-HO-101-3, R2 Hejlsberg, KS-HE-101-4/KS-HO-101-2). Prerequisito: sanare il dump cieco sugli hook (A-HE-100-4) — è lo STRUMENTO, va primo. Dump-diff off/on sullo stesso albero candidato; i due bracci del funnel devono provare emissione DIVERSA su una probe (hash pubblicati, KS-HE-101-1) e la probe esce da `{main}` (A-HE-101-4).
3. **Wildcard su `Op` = bloccanti.** `visit_addrs _ => {}` (R3 Hejlsberg, KS-HE-101-2) e `bin_op_of _ => None` (R3 Matsakis, ledger leak A-MA-100-2): match esaustivi prima del flip, stessa classe S-96.

## Conflitti / posizioni

- **H-B2 sui siti stack non fusi (R2 Matsakis)**: il flip ritira il −16,2% dove l'emissione on non copre. Il team lo registra come **precondizione d'ordine, non conflitto col flip**: prima del flip si decide CON MISURA (estendere BinaryAdd ai residui della pipeline on, o rinuncia pre-registrata) e il giudice del punto 4 si sposta sui residui post-flip (A-MA-101-3). Hoare/Hejlsberg: nessuna obiezione, non coperto dai loro perimetri.
- **Unit-cache key**: Hoare la vuole nel contratto (continua a distinguere i modi); Hejlsberg la dice cintura ridondante mai collaudata (A-HE-101-3). Riconciliazione: un controllo positivo decide — provata o riclassificata, mai contata "soddisfatta per documentazione".
- **Profondità del sigillo**: Hoare chiede chokepoint di TIPO (token di boot stile VmGate, A-HO-101-1) prima del flip; Matsakis/Hejlsberg si fermano al ri-collaudo del dente sulla semantica nuova. Dissenso registrato: minimo comune = dente per ogni `[[bin]]`; il chokepoint resta emendamento Hoare.

## Priorità proposte per l'ordine S-100

1. Contratto di modo value-parsed + ri-derivazione dente/M5/launcher/funnel (A-HO-101-2, A-HE-101-1).
2. A-HE-100-4 (dump sanato) — sblocca tutto il resto.
3. A-HE-100-2 + A-MA-101-2 (match esaustivi, NON COMPILA).
4. A-HE-100-1 (tripwire col dump sanato) + A-HE-100-3 (differenziale BinaryAdd≡Binary(Add)).
5. Decisione misurata H-B2 sui residui on (A-MA-101-3).
6. Flip; poi KS-HE-101-3: ogni rotazione pin post-flip collauda il braccio OFF (RC-1 invertita, R4).
