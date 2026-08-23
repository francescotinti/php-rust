# WP_SESSION_156 — leva HD2-hostcall PROMOSSA (pin NUOVO s156); census post-CE1 rifondato; dente A4 morso, salita dichiarata
**In una frase**: eliminata l'allocazione-per-chiamata dei sei controlli
d'esistenza più usati (classi/funzioni/metodi e backtrace), misurata e
promossa a default coi collaudi verdi; il censimento nuovo dice dove incide.
**SCOREBOARD** (pin NUOVO s156 phpr 42efea3e34feb390 + server ef89630f9c7408c3):
arith 5,4 ↓ · prop 5,5 ↓ · calls 4,7 ↓ · str 4,2 ↓ · arr 3,2 ↓ · re 2,6 ↑
(micro R=5 di promo; rif. s154: 5,5/5,6/4,8/4,3/3,3/2,5) · WP t6 1,771 (non
rimisurato; coppia al pin s156 DOVUTA S-157) · ORM 6,972–7,053 · dbal
7,385–7,422 · corpus 1412 · batteria 1748/0/2 (cap loc NUOVI 25742/6815) ·
**leve spedite: 1 (HD2-hostcall, PROMOSSA)** · incidenti 19 (=).

## Esiti secchi
1·Census ORM probe s155 rc=0 VALIDO (identità §3 ×2, repliche identiche):
  testa post-CE1 array_map 7,68M · class_exists 7,28M · bt 6,15M= · gdc
  4,56M=; attese (b)/(d) FUORI → istruttoria: scope s149 annidante, il
  run_loop dell'AUTOLOADER resta sul nome ⇒ chiamate ce ∈ [1,23M;2,46M],
  residuo miss/autoload E ∈ [4,82M;6,05M] (fetta futura per NOME).
2·Funnel CE1(b) apporzionato (Δ per-nome s154→s156): ce −2,46M ·
  __reflect_class_real_name −1,28M · __reflect_class_loc −452k · minori
  −343k ⇒ l'oltre-attesa ORM di S-155 ha i numeri (≈45% fuori da ce).
3·Leva HD2-hostcall (args-Vec CallHostBuiltin, arità ≤4, 6 nomi convertiti):
  criterio+attesi blind pre-registrati (secondo attore ATTESI-OK), disasm bl
  6014→6033 (+19); smoke +20,5 → R=5 **D=+16,0** (soglia 4,0, rumore 3,0/1,0,
  UB 13,8+3,0 in banda), 13 guardie ok; suite dichiarata sotto-risoluzione
  (micro-judged). Reperti: riconc. smoke↔R5 fuori banda 0,5 · backtrace al
  bordo (−8,3 = 2 tick = rumore) · conferma post-pin D=+5,0 segni 5/5.
4·Promozione t1 STOP dente loc A4 (mod.rs +30, run.rs +29) → salita
  DICHIARATA (solo file test, candidato al byte); t2 rc=0: batteria
  1748/0/2 · corpus zero flip · fixture 10+fx-ce · ORM 16== · hk 0E/0F ·
  pin phpr+server s156.
5·Fix summary dbal (az.rev.#4) DIAGNOSTICATO: tr muore su byte non-UTF-8
  prima di `Tests:` ⇒ LC_ALL=C+grep -a nella prossima copia coppia; reperto
  nuovo: dbal phpr 3921/626 vs oracle 3929/594 da dichiarare alla coppia.

## ⭐ Lezioni (max 3)
- ⭐⭐ L'attesa per-NOME sul census si deriva dal PERIMETRO dello scope, non
  dalla semantica del builtin: due sessioni, due termini mancati (plumbing,
  poi miss/autoload annidato).
- ⭐⭐ Su leva misurata e vinta il dente loc si emenda sul file di TEST
  (salita dichiarata): il candidato resta al byte e l'A/B resta valido.
- ⭐ La conferma post-pin arbitra solo il SEGNO (assoluti mossi ~12 dalla
  finestra su ENTRAMBI i bracci); il giudice resta l'ABAB della finestra.
