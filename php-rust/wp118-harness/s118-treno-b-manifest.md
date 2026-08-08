# s118-treno-b-manifest.md — treno B: manifest per NOME (cap 5), committato PRIMA di ogni lavoro (S-118)

Istruttoria siti: censimento C1 borrow-non-clone su run.rs (agente S-118, in
calce). Fedeltà per-vagone, perf per-treno; netto ≥ −banda-v2(cat) per OGNI
categoria; bisezione pre-registrata: stacca l'ULTIMO vagone, max 2 giri.

## Vagoni (cap 5, per NOME)
- **V1 · H-P1a**: `Op::PropGetSlot` (run.rs:4166) — borrow al posto del clone
  Rc del ricevitore su IC-hit (`reg_load_slot` bump+drop che muore a fine
  arm). Bersaglio: prop (PropGetSlot = 12,5% op del giudice). +3,33 firmata
  S-113 (con V2, sotto vecchia banda).
- **V2 · H-P1b**: `Op::PropGetSlotRecv` ramo NON-fuso (run.rs:4261) — stesso
  borrow; sotto L-A il ramo è il solo-miss ⇒ atteso ~0 sul giudice, vagone di
  coerenza col gemello V1. Ricostruzione ESISTENTE: commit 8bb395c (S-114).
- **V3 · C1-IncDec**: `Op::PropIncDec` (run.rs:4495) — `deref_clone`
  incondizionato → guardia `matches!(obj, Zval::Ref(_))` (prior art H-C1b).
  FUORI dai sei giudici: solo guardie, gain atteso su WP reale.
- **V4 · C1-Isset**: `Op::PropIsset` (run.rs:4687) — stessa guardia-Ref.
  Fuori dai giudici.
- **V5 · C1-MCall**: `Op::MethodCall` (run.rs:4913) — guardia-Ref sul
  `recv.deref_clone()` (il frame vuole l'handle POSSEDUTO: guardia, non
  borrow). Fuori dal giudice calls (census: Call/CheckArity, no MethodCall).

## Treno-1 (questa sessione, se il muro orario regge) = V1+V2
Rigiudizio H-P1 sotto A′/banda-v2 (playbook L-A S-117): cherry-pick -n
8bb395c su HEAD nel target dedicato riusabile; admission dump per NOME;
criterio ≤10 righe PRE (soglia prop max(4; banda-v2 3,33), guardie
non-bersaglio a solo-regressione ≥ −banda-v2(cat), R=5 ABAB, quanto-guard);
UN A/B treno-vs-pin. V3-V5 = vagoni istruiti per S-119 (treno-2), con
l'ordine che uscirà dai contatori C-lite.

## Esclusi per NOME (motivati, dall'istruttoria)
Gemelli freddi di V5 (run.rs:4969/4983/4995/5011/5018): churn senza metro.
`ThisMethodCall` 4933, magic/hook, `PropGetSlotRecv` push rv 4259: clone
NECESSARIO (ownership al frame / ordine di pila S-113). `prop_get/set_entry`:
già guardati (H-C1b). Percorso set IC-hit: zero siti (conferma s113).
