# CONCILIO a 9 — S-146 — convocazione su B3/filone conteggi (KS-B4)

Convocato per CAMBIO DI ROTTA da sonda (REGOLE §7): la sonda-B S-145 ha fatto
scattare **KS-B4** (regola pre-registrata s144-criterio-B.md p.3): churn
memcpy-dominato 69,5% ≥ 60% ⇒ **B1 uniform-rc e B2 root-at-decrement NON si
aprono**; il filone CONTEGGI (B3) torna al concilio — «non si improvvisa»
(s144-progettazione-B.md §3-B3).

## Quesito (uno solo, con sotto-punti)

**Quale leva del filone conteggi si istruisce, con quale perimetro semantico,
quale forma d'emissione, quali giudici/kill-switch pre-registrati, e in quale
ordine di istruttoria?** Sotto-punti obbligatori per ogni sedia:
a) forma d'emissione (design96 apertura #1: un `LoadSlot` con flag `take`
   deciso a compilazione NON è un corpo caldo in più — forma MAI valutata;
   il braccio nuovo resta sotto il tetto WP-39..44 / A-LB-97-1);
b) perimetro semantico fedele (Stogov S-96: in Zend i CV non si consumano;
   morte anticipata osservabile anche senza `__destruct` — spl_object_id,
   WeakReference, risorse; nucleo-stringhe = perimetro senza identità);
c) serve un censimento liveness F1 SU ORM? (i conteggi S-95/96 sono sul
   media group WP, non su ORM);
d) alternative nella famiglia «muovere MENO» oltre TakeSlot: estensione
   borrow-first/through-borrow ai siti consumatori (precedenti SPEDITI:
   HC1 S-140 hint-check senza clone; L-FR1 S-145 dim-read senza clone
   dell'array — semanticamente invisibili, zero liveness) · «arena-conteggi»
   (nominata in NEXT_SESSION p.2, MAI definita: definirla o archiviarla);
e) che cosa compra B3 e che cosa NO (Matsakis R4 S-143: la scommessa compra
   la TAPPA ≤3×, non la parità; il perimetro modellato della sonda = 1,52 s
   sul gap ORM 37,6 s — il resto del churn famiglia ~4,4 s è glue fuori
   modello, nessun claim oltre la risoluzione).

## Numeri di fatto (dai verdetti, per grado)

- **Sonda-B S-145** (`wp145-harness/s145-sonda-b-verdetto.out`, SONDA):
  partizione churn modellato **memcpy 69,5% (1,06 s) · inc-dec 14,1%
  (0,21 s) · nota 16,4% (0,25 s)**; prezzi per-movimento (coppia clone+drop):
  scalar 2,88 · str 3,85 · arr 3,84 · obj 3,36 ns; conteggi ORM: **367,6M
  movimenti** (scalar 91,1M · str 104,1M · arr 60,9M · rc obj+ref+other
  111,5M); gcnote_total 238,6M (== dossier S-141); pair a banda (gate 5%
  MAI ricollaudato — az.rev. S-145 #5): zcell 8,71–8,80 · arr0 11,57–11,79 ns.
- **Tranche-2 S-144** (`wp144-harness/s144-census-tranche2.out`, CENSUS):
  galloc 471,3M/run; quota_obj_max_loose **2,4% < 25%** ⇒ A morta per misura
  (regola S-143 p.3); quota_str **27,6% (129,9M)** · arr 9,4% · residuo
  other 57,9% DICHIARATO fuori-budget; zval_size=16.
- **Profilo oracle S-144** (`wp144-harness/s144-profilo-oracle-verdetto.out`,
  INDIZIO + az.2 whole-stack): churn_zval **IN BUDGET** (oracle 0,20–0,26pp
  vs phpr 10,3%; tie-break bilaterale S-129 tassa ~10×/statement) · memops
  **FUORI BUDGET** (oracle 7,8–8,1% vs phpr 12,6%, rapporto 62–66% ≥50%) —
  riaprirli richiede attribuzione Zval-move dedicata (aperture per NOME).
- **Liveness S-95/96** (`wp95-harness/design95-liveness.md` +
  `wp96-harness/zvalcensus-recount.out`, media group WP): slot_reads_rc
  53,6M · would_take_rc **47,1%** · safe/would_take **90,2%** (P2 ok) ·
  nucleo str 18,7% (banda MEDIA 0,84–1,21% CPU) · safe_ref 0,013%
  (guard di tipo quasi mai preso, MAI superfluo). Verdetto design96:
  **TakeSlot come braccio nuovo NON scritto** (pedaggio ~ guadagno);
  aperture: forma-flag non valutata · O1 outlining (Bak) prerequisito
  dichiarato per ogni corpo caldo in più · moltiplicatore canale 4,5–6,5%
  è SCREEN (R=1).
- **S-140** (`sessions/WP_SESSION_140.md`): 44% dei clone/drop INLINE da
  run_loop; CHURN suite 32,3–32,6% vs DIMPROP 6%.
- **Gap ORM**: coppia net 8,59–8,71 (37,6 s); giudice della scommessa =
  coppia ORM 2/lato net (banda ±0,7%).

## Vincoli vigenti (si citano, non si rivotano)

- KS-B4 scattato per misura: B1/B2 chiusi SENZA nuovo concilio (NEXT_SESSION
  «NON riproporre»). I 6 veti Q3 S-143 confermati 9/9 (alloc-removal senza
  costo sostitutivo · contenitori sul call path · NaN-boxing · SSO inline ·
  gc note-time WP-21 · verdetti su build emendata). Binding output-capture
  (Pedersen/Stogov 9/9) INTATTO. A = solo pool+refcount+handle-gen se mai
  riproposta (S-143). Batteria+corpus 1414×2 per NOME + ORM 3E/13F + fixture
  bilaterali = gate semantici invariati; fail NUOVO per NOME in
  weakrefs/destructor ⇒ STOP fetta.
- REGOLE.md §3 (criterio prima, ABAB, soglie fondate) e §4 (attribuzione).
- Az.rev. S-145 vincolanti #3/#4: guardie con giudice DENTRO lo script;
  ogni giudice/guardia nominata nel criterio deve esistere per nome.

## Fascicolo (ordine di lettura per le sedie)

1. `wp145-harness/s145-sonda-b-verdetto.out` (esito che convoca)
2. `wp144-harness/s144-criterio-B.md` (regola madre pre-registrata)
3. `wp144-harness/s144-progettazione-B.md` (B su carta; §3-B3)
4. `wp144-harness/s144-census-tranche2.out` + `s144-profilo-oracle-verdetto.out`
5. `wp95-harness/design95-liveness.md` (A-ZV2: fasi, predizioni, trappole PHP)
6. `wp96-harness/zvalcensus-recount.out` + `wp96-harness/design96-confronto-piano-b.md`
7. `wp143-harness/concilio/sintesi.md` (deliberato S-143 + regola di decisione)
8. `sessions/WP_SESSION_145.md` + `NEXT_SESSION_WORDPRESS.md` (stato + veti)
9. `REGOLE.md`
(Fatti di codice: s144-progettazione-B §1 è la fotografia dal SORGENTE,
verificata S-144 — le sedie NON aprono i .rs in questa fase: finestra di
misura attiva, niente LSP/build/run — veto «misure con LSP in volo».)

## Protocollo (due fasi, token-lean)

Fase 1: 9 bozze INDIPENDENTI (nessuna sedia vede le altre) in
`wp146-harness/concilio/<sedia>.md`; mandato: REFUTARE, mai benedire.
Output per sedia: VERDETTO (CONCORDO / CON EMENDAMENTI / MI OPPONGO sul
quesito e su ciascun sotto-punto a–e) + emendamenti numerati R1..Rn
(cosa/perché/misura) + kill-switch pre-registrabili + (Gregg) mandato
inverso: «cosa sappiamo oggi che ieri non sapevamo». ≤600 parole per
verbale. Fase 2: team tematici con relatori; verbali individuali =
fonte VINCOLANTE. Sintesi FONDAMENTALI-first in
`wp146-harness/COUNCIL_S146_REVIEWS.md`.
