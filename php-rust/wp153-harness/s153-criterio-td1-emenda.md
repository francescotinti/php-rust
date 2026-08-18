# s153-criterio-td1 — EMENDA §7-bis (braccio A = GEMELLO; rieseguo lo smoke emendato)

**Reperto**: il rebuild post-revert dà `f95a1067f528e147` ≠ pin `cbbe7173...`.
Causa istruita: dal pin s150 DUE commit toccano `crates/` (f25ced1 dente A4
batteria; 5c646e5 segmenti sonda feature-gated) — con feature OFF sono
semanticamente inerti ma spostano righe/layout (il veto trasversale
«byte-identità come gate di edit .rs post-pin» già lo prevede). ⇒ il braccio
A dello smoke-td1 (binario pin) NON era il gemello di B (tree corrente+edit):
D poteva contenere drift di layout dei due commit, non solo L-TD1.

**Emenda** (catturata nell'atto, PRIMA di ogni uso di record del verdetto):
- A ⇒ **gemello** `phpr-s153-gemelloA` = build del tree corrente SENZA edit,
  hash `f95a1067f528e147`, stash fatto; B ricostruito dallo stesso tree + i
  SOLI edit L-TD1 (diff 38+/9−, atteso 297cffc90664f03a se riproducibile,
  hash dichiarato al run).
- Smoke RIESEGUITO con TAG nuovo `smoke-td1g` via `s153-ab2.sh` (copia di
  s153-ab.sh col solo cambio di A e della sua guardia hash). Tutto il resto
  del criterio INVARIATO (giudice, soglia, UB, companion, guardie, attesi
  s153-smoke-atteso.md §3-5 già promossi dal secondo attore).
- Il verdetto smoke-td1 (D=−5,0) resta agli atti come TENTATIVO CONTAMINATO
  dal braccio non-gemello: non cita.
- Nota per OGNI A/B futuro a pin invariato ma tree avanzato: il braccio A si
  COSTRUISCE (gemello dalla ricetta sul tree corrente), non si pesca dallo
  stash del pin.
