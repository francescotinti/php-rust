# WP_SESSION_141 — census TakeSlot DELUDE + leva L-RD1 (teardown array inline) A/B PROMOSSA, catena a S-142

**In una frase**: il vecchio piano TakeSlot, contato uno a uno, muove meno di
un millesimo del tempo (chiuso); il canale vero è lo smontaggio degli array
(~2%), e la leva scritta lì — niente più una chiamata per elemento — misura un
guadagno piccolo dal segno costante, pronto da spedire in S-142.

**SCOREBOARD** (pin s140 INVARIATO; micro bilaterali non rimisurate): **arith
5,6 = · prop 5,6 = · calls 4,8 = · str 4,3 = · arr 3,2 = · re 2,6 =** · hintcall
7,3 = · WP rif **1,765–1,777 (banda_ON 0,033)**, coppia non dovuta (pin fermo) ·
**leve spedite: 0 — DICHIARATO: A/B L-RD1 PROMOSSO, catena a S-142** · incidenti 14 (=).

## Esiti secchi
1·**Census TakeSlot rc=0** (criterio pre-registrato; parità 16 nomi == ×2;
  r1==r2 al singolo evento): would_take_safe 22,3M ev/run = **0,06–0,09%
  suite**, str 0,02–0,03% < soglia 0,5% ⇒ **DELUDE** (decisione p.6). P2
  storica 85,2%: perimetro ok, manca il VOLUME.
2·**Riquantificazione** (raw sample S-140, datati): Zval-drop-glue leaf 4,5–4,7%
  suite; caller: run_loop-inline 34,5% · **Repr(array) 21,6%** · recycle_frame
  8,5%; «Repr-drop 11%» era quota tra caller, NON di suite; teardown array ≈2%
  = canale più coeso; other = coda lunga (max 1,1%).
3·**Leva L-RD1 «teardown array inline»** (criterio pre-registrato; copia-gate
  rc=0): Drop for PhpArray drena Packed/Hashed con match esaustivo (scalari
  senza call; ordine e profondità INVARIATI). Disasm: 1 solo bl residuo verso
  glue Zval vs call per-elemento in A. Banda propria 2,0 (D_null 0,0). **A/B:
  smoke +4,5 (2/2) → R=5 D=+5,0 SU SOGLIA 5,0 — PROMOSSA con QUALIFICA (bordo
  esatto, drop-1 A′=5,0; evidenza portante segni 7/7 + riconc. 0,5); sotto
  lato-basso modello (non-gate, ~0,8 ns/elem); guardie 9/9, collaterali positivi
  arr/str/re.** Catena a S-142; candidato acb5e7d in coda CI; binari A/B in
  phpr-rd1-target/keep/.
4·**CI**: GH pin s140 success; checkout v4→v5 success (warning Node20 chiuso);
  feed locale OK. Peak per POSIZIONE NON aperto (finestra) → S-142 p.2.

## ⭐ Lezioni (max 3)
- ⭐⭐ Un canale nominato dalla rotazione si RIQUANTIFICA sui raw datati prima di
  investirci: «Repr-drop 11%» (quota tra caller) valeva 1% di suite.
- ⭐⭐ Il census a conteggi (r1==r2 esatto, parità per NOME) chiude un filone in
  30 minuti: più a buon mercato di qualunque A/B sul canale sbagliato.
- ⭐ Un giudice nuovo si collauda contro il PARSER del harness (awk `$i<N` senza
  spazi): la banda t1 è stata annullata dal formato, non dalla misura.
