# S-165 p.3 (az.rev. S-164 #2) — attesa census AL3 che NOMINA il +1 (PRE-registrata)

## Difetto da curare (rilievo #3 S-164)
Due assoluzioni off-by-one consecutive sullo stesso arbitro; la spiegazione
«+1 = buffer del Vec `exts` del pool al primo put_ext» è POST-HOC e non
verificata; il criterio S-164 era internamente incoerente (clausola (a)
diceva 200000, p.3 diceva 199999).

## Attesa PRE-registrata (unità hostcall_n = TUTTE le alloc sotto il hostcall)
Probe-B = tree col patch AL3 (s164-al3-edit.patch) + **UNA riga discriminante:
`Vec::with_capacity(EXT_POOL_DEPTH)`** all'init del pool `exts`
(vm/mod.rs:2812) — il buffer nasce PRIMA del primo hostcall e SPARISCE dal
conteggio. Probe-A = copia − patch (identico a S-164). Driver N=200000.
- `class_exists`: **A=200006 · B=7 · Δ=199999 ESATTO** (1 Box/iter rimosso,
  meno 1 cold-Box; il +1 del Vec NON deve più comparire).
- Ogni altro nome: Δ=0. Repliche r1==r2 su entrambi i probe.
- Esiti: Δ=199999 ⇒ il +1 di S-164 è VERIFICATO come buffer del Vec (da
  post-hoc a provato), «meccanismo nominato» del verbale AL3 CONFERMATO;
  Δ=199998 di nuovo ⇒ l'attribuzione al Vec è FALSA: si torna al sorgente,
  incidente da contare (seconda spiegazione caduta sullo stesso arbitro);
  altro ⇒ STOP e sorgente.
- COERENZA INTERNA: questa attesa vale per TUTTE le clausole del criterio
  (nessun 200000 residuo); il verdetto della leva AL3 (non pagante, p.3b)
  NON dipende da questo esito — qui si arbitra solo il meccanismo.
Esecuzione FUORI finestra misure (census builds in target dedicati,
sequenziali); esito in s165-census-al3-verdetto.out.
