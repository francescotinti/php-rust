# WP_SESSION_127 — L-OL1-F1 «stampo» SPEDITA (pin s127) · bisezione compoff chiusa

**In una frase**: ogni classe ora fotografa una volta la propria tabella di
proprietà di default e ogni `new` successivo la clona invece di ricostruirla —
creare+distruggere un oggetto costa il 20% in meno, un test GC del corpus
flippa VERDE per lo stesso meccanismo, e il composer «muto» è risolto in due
divergenze nominate.

**SCOREBOARD** (pin **s127 834f5e01**fbdb7ebc; micro R=5 dal gate di promozione):
**arith 5,4 ↓ · prop 5,5 ↓ · calls 4,7 = · str 4,2 = · arr 3,3 ↑ · re 2,5 ↓**
(vs s125; ±0,1 = rumore) · rif WP full = 1,815–1,896 (S-125 @ s124) ·
**leve perf spedite: 1 (L-OL1-F1)** · incidenti: +1 apparato (LSP in finestra
di misura, dichiarato, rerun pulito). 2026-08-10 · Fable 5.

## Esiti secchi
1·az.rev.#1+#2: regola di nomina ri-committata PRIMA dell'ammissione (c71620b);
  **additività churn CHIUSA a 3,4 ns bilaterale** (Δins 320/36,6 · Δdrop 90/0 —
  drop differito GRATIS per l'oracle); senza interpolazione il ciclo-vita è 12,3×.
2·Ammissione: census 9 alloc+9 free per new+ctor+drop (oracle ~1-2); il `[]`
  default passava dal thunk prop_init = una CALL per new; disasm 383 istr/36 bl.
3·**F1 «stampo»** (a64721b): OnceCell su CompiledClass, snapshot al Ret del thunk
  INIT_PROPS, skip InitProps, default COW condivisi; sonde oracle IDENTICHE.
  Admission2 fuori-predizione DIAGNOSTICATO da modello unico −2+w (6/6); bl 36→31.
4·**A/B R=5 PROMOSSA**: objalloc **+250,0 ns/iter (1226,7→976,7, −20,4%)**, churn
  +230 (10,2→8,9×); 8 guardie ok. EMENDA dichiarata: guardie sei-storiche sui
  giudici scalati wp123 (smoke-1 morso arr/re per METRICA, denominatore annidato).
5·**PROMOZIONE rc=0**: batteria 1746/0 inventario identico · corpus **1414**
  (flip `bug69534` VERDE dichiarato: int(8)→int(2)==oracle, stesso meccanismo
  COW; 1415→1414) · fixture 7 rc=0 · ORM 16 nomi · hk 0E/0F · server 72f47855.
6·**Bisezione compoff CHIUSA** (ba42e3c): §3.19-bis exec/system/passthru/proc_open
  SOLO diretti (dinamico «undefined», function_exists incoerente; popen assente)
  + §3.19-ter `display_errors=stderr` come OFF ⇒ Fatal MUTO. Cure S-128 a catalogo.
7·az.rev.#3-#5: cifra canonica mappa = NETTO (dbal 8,57-8,60 · coll 8,22 ind.) ·
  gate contesa in ictx/s · correzione a verbale hf (≥3 nomi unit puri).

## ⭐ Lezioni (max 3)
- ⭐⭐ Un «fuori predizione» può essere il meccanismo che si rivela: se UN modello
  spiega TUTTE le celle (−2+w su 6/6), la diagnosi è una scoperta, non un fallimento.
- ⭐⭐ Una guardia con bande nate su ALTRI giudici morde per metrica, non per
  merito: la soglia in ns/iter viaggia solo col SUO denominatore.
- ⭐ «Undefined function» su funzione che ESISTE in diretta = dispatch dinamico e
  intercetto compile-time sono TABELLE SEPARATE: sondare sempre la forma callable.
