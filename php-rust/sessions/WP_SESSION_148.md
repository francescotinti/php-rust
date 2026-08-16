# WP_SESSION_148 — SECONDO ATTO: l'other 57,9% ha i NOMI (testa = temp dei builtin 165,6M; hashbrown/pool-Frame MORTI per conteggi); coppia t2 fuori banda dal BASSO ⇒ t3

**In una frase**: il censimento a tag ha dato un nome alle allocazioni ignote
— il 61,5% vive dentro i builtin (~6,7 alloc a chiamata), i due candidati
«ovvi» del concilio (hashbrown, pool frame) muoiono al tavolo senza codice; la
replica della coppia WordPress esce dal basso della banda e chiede la terza.

**SCOREBOARD** (pin s145 a89faf32+4a9adc51 INVARIATO): arith 5,5 → ·
prop 5,5 → · calls 4,8 → · str 4,3 → · arr 3,2 → · re 2,5 → (micro n.r.: pin
invariato) · **WP full t2: 1,722–1,742 (N=5 pulite) FUORI BANDA bordo BASSO
(1,722 vs limite 1,729) ⇒ per criterio NESSUN claim, rif RESTA 1,765–1,788/
0,036, t3 DOVUTA** · media t2 2,454–2,569 · ORM/dbal n.r. (rif S-147) ·
**leve perf spedite: 0 — DICHIARATO** (attribuzione-ordinata dal secondo
atto; sonda-prezzo = passo successivo del criterio p.8).

## Esiti secchi
1·**p.1 CENSUS ATTRIBUZIONE per TAG rc=0** (criterio+parser golden 10/10
  PRIMA; identità Σtag==galloc_n ESATTA ×2; repliche worst 0,056%; parità 16
  nomi; workload==s144 a −0,73%): other 269,3M = **hostcall 165,6M (61,5%)**
  > none/RESIDUO 94,6M > arrgrow 5,7M > frame 3,0M > gc 0,36M. **KILL per
  conteggi (soglia 24,8M) AL PERIMETRO DEI TAG (rett. rev.): growth-hashbrown
  0,23×, pool-Frame 0,12× (FramePool già ricicla), gc — MORTI.** Il 69,6% di
  TUTTE le alloc grezze avviene nell'estensione dinamica dei builtin.
2·**Anatomia (INDIZIO)**: ~6,7 alloc non-attribuite per chiamata nei CORPI;
  plumbing nominato pop_keys/split_off (1 Vec a CallHostBuiltin, 11 siti —
  l'args-Vec di S-104 ha il canale); shape ≤48 B 107,9M + ≤16 B 98,8M.
3·**p.2 coppia t2 @ s145** (az.rev. S-146 #1): 5/6 pulite (leg1 ictx 202%
  SEGNALATA); leg3 1,722 sotto il limite basso di 0,007 ⇒ esito
  pre-registrato «fuori banda altrove»: nessun claim, t3 dovuta. Banda di
  finestra t2 = 0,020 (t1: 0,090) ⇒ spread CROSS-finestra (unione 0,101).
  Deriva (az.#3): N=5<6 ⇒ SOTTO-CAMPIONATA; peak t2 ASCENDENTI (opposto
  t1), lettura peak: MISTO. Az. S-146 #4 (FR1): slittata a S-149, dichiarato.

## ⭐ Lezioni (max 3)
- ⭐⭐ Una partizione COMPLETA con identità esatta (Σtag==galloc_n) vale più di
  N contatori parziali: l'other è diventato NOMI in una sessione sola.
- ⭐⭐ Due candidati nominati dal concilio erano sotto soglia di un ordine di
  grandezza: contare PRIMA di prezzare uccide gratis (kill a conteggi).
- ⭐ Il rumore WP è TRA le finestre (0,090/0,020, unione 0,101): la banda
  canonica si rifonda multi-finestra, mai da una sola.
