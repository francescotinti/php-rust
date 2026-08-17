# WP_SESSION_149 — TERZO ATTO: l'other ha UN nome (debug_backtrace 74%); BT1 A/B −96,3%; t3 COMPATIBILE, banda multi-finestra rifondata

**In una frase**: il censimento per nome ha svelato che quasi tutta
l'allocazione «ignota» dei builtin viene da UNA funzione — debug_backtrace,
che ignorava i propri parametri e costruiva sempre la pila intera — la cura è
scritta, byte-identica al PHP vero e 27× più veloce sul giudice; la coppia
WordPress è tornata in banda alla terza replica.

**SCOREBOARD** (pin s145 a89faf32+4a9adc51 INVARIATO): arith 5,5 → · prop
5,5 → · calls 4,8 → · str 4,3 → · arr 3,2 → · re 2,5 → (micro n.r.) · **WP
full t3 1,786–1,802 (N=6, prima finestra 6/6 PULITE) COMPATIBILE, rif S-142
CONFERMATO; banda_ON rifondata = unione t1+t2+t3 1,722–1,823 (0,101, 17
coppie)** · media t3 2,504–2,540 · **leve spedite: 0 — BT1 A/B VINTO
(ritmo assolto), promozione RINVIATA a S-150 per vincolo t3-prima
(dichiarato)**.

## Esiti secchi
1·**p.1 census tranche-4 rc=0** (golden 11/11 PRIMA; identità ESATTA ×2;
  unnamed/overflow 0; repliche 0,000%; parità 16 nomi): **debug_backtrace
  other=130,15M = 5,2× soglia = UNICA candidata (73,95% dell'other; n=275,0M
  = 81,9% del tag)**; tutte le altre sotto soglia anche per FAMIGLIA; scarto
  +3,2% vs s148 NOMINATO (path workdir); pop_keys second'ordine.
2·**p.2 sonda-prezzo rc=0**: pair16 6,37–6,38 · pair48 11,21–11,27 ns netti
  (repliche ≤0,52%); splitoff3 replica 5% >2% DICHIARATA. **DECISIONE scritta
  PRIMA: attesa BT1 0,83–3,10 s ≥2× ⇒ scommessa suite AMMESSA**; pop-diretti
  1×–2× (micro-judged); args-Vec ~1× (kill bersaglio-solo).
3·**Leva BT1 A/B** (options/limit onorati — anche cura FEDELTÀ): D=+19000
  ns/iter (19733→733, −96,3%), 7/7, 95× soglia, guardie 6/6 ≤1 tick, fixture
  fx-backtrace BYTE-ID, parità B pulita. Promozione S-150.
4·**p.3 t3 rc=0**: 6/6 pulite; deriva peak ρ_A=−0,886 CONFERMATA,
  peak↮rapporto (ρ_B=−0,600); leg1 PULITA — coerente con l'indagine ictx
  (firma di FINESTRA amplificata dalla baseline bassa, s149-ictx-leg1).
5·**p.4 FR1: SLITTATA ×3 (dichiarato)** — in S-150 non slitta più.

## ⭐ Lezioni (max 3)
- ⭐⭐ Un ranking per TAG può crollare su UN nome al livello successivo:
  contare per NOME prima di prezzare il plumbing.
- ⭐⭐ La firma % su mediana BASSA amplifica disturbi assoluti piccoli:
  leggere l'eccesso ASSOLUTO a corredo.
- ⭐ Una cura di fedeltà può ESSERE la leva perf: il byte-id con l'oracle è
  il gate che le unisce.
