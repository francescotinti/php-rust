# WP_SESSION_138 — sblocco dim-write (2 sonde + A/B tra pin) + LEVA FD1-ext RMW SPEDITA (pin s138)

**In una frase**: risolto l'enigma dell'eccedenza FD1 (l'aritmetica mescolava
giudici diversi), la nuova leva sui `+=`/`++` dentro le proprietà-array è stata
costruita, misurata (−54% e −58%) e promossa a pin nuovo con la catena verde.

**SCOREBOARD** (pin NUOVO s138 fa17dabd + server a9aded45; micro dal gate promo):
**arith 5,6 ↑ · prop 5,6 = · calls 4,8 ↑ · str 4,3 ↑ · arr 3,2 ↓ · re 2,6 =**
(±0,1 run-to-run) · rif WP on-only **resta 1,767–1,781 @ s136** (pin ruotato ⇒
coppia DOVUTA S-139, banda ON N≥5) · **leve spedite: 1** · incidenti +1 (n.14)=14.

## Esiti secchi
1·**Az.rev. S-137 #1**: disasm pin vs probe RICOSTRUITO (ricetta identica, hash
  diverso dichiarato) — artefatto-inlining **REFUTATO** (bl +63 = timer nominati).
2·**Sonda v2 arm-only** (inerzia 0,000): arm 51,9 pulito, identità v1 NON CHIUSA
  → sospetto = aritmetica CROSS-GIUDICE (118,2 su m-dimwrite − 83,3 su
  objdatains). **A/B v3 pin s135↔s136 su m-dimwrite (zero probe): D=63,3 vs UB
  69,6 IN BANDA → identità CHIUSA, blocco dim-write RIMOSSO**; coerenza-arm
  51,9+63,3=115,2 ≈ 118,2 (−3,0): lo strumentario riconcilia.
3·**LEVA FD1-ext RMW SPEDITA**: cella IC su FieldAssignOp/FieldIncDec, fast path
  peek+op-silente+field_write_walk riusato (perimetro: chiave Long|Str, entry
  presente, old Long|Double, op Add/Sub/Mul). A/B R=5: **m-dimrmw D=+173,3
  (320→147) · m-diminc D=+156,7 (270→113)**, guardie 7/7 (objdatains ±0,0),
  fixtures BYTE-ID cand==pin. Fuori-modello +110 → sonda attribuzione: v1 NON
  CHIUSA (ε fill sottostimato), **v2 monobinario col kill-switch CHIUDE (scarto
  +3,7/17,3)**. Promo rc=0: batteria 1746/0/2 inv==s125 · corpus golden ·
  fixture 9/9 · micro R=5 · ORM 16 nomi · hk 0E/0F · **conferma post-pin in
  banda (5,0/5,0)** → il pin eredita il verdetto. **Pin s138 + server s138.**
4·Reperti: divergenze pieno-vs-oracle PRE-esistenti da fixtures-rmw (undefined-
  key RMW, float-key/str-increment, notice overloaded) → catalogo per NOME ·
  **CI: 5 runner concorrenti (runner.lock non tiene) = causa dei batteria-FAIL
  del feed; runner fermati, coda 35 da smaltire.**

## ⭐ Lezioni (max 3)
- ⭐⭐ Un'identità che mescola GIUDICI diversi non è un'identità: si chiude sul
  giudice del modello — e un A/B tra pin STASHATI è la sonda più pulita che c'è.
- ⭐⭐ Il gemello di relink non eredita il verdetto A/B gratis (layout): la
  conferma-smoke post-pin è il ponte, pre-registrata prima della promozione.
- ⭐ Incidente n.14: sed di copia non matcha le righe eseguibili (quote nel
  pattern); NO-CLOBBER S-133 ha morso — copie di script SOLO con Edit mirato.
