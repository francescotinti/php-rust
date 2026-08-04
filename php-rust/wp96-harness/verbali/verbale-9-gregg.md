# Verbale 9 — Brendan Gregg (mandato INVERSO: giudico l'oggetto, non l'apparato)

## VERDETTO: **LA MISURA C'È, LE QUATTRO LETTURE SONO SBAGLIATE.**

Il contatore full/media **è davvero azzerato**: coppia stessa-sera, ordine
oracle-prima, conteggi identici (30472/4558029/86W/73S), raw committati,
guardia DB+uploads. Non è dichiarazione di comodo. Ma S-94.0 ha letto
**rapporti contro una banda**, mai **assoluti contro assoluti** — e i suoi
stessi raw, incrociati con `gaps/REPORT_GAP_64.md` (riga 11, «peak phys
time -l: oracle 393,7MB / phpr 1.186,9MB»), ribaltano tre letture su quattro:

| asse | gamba **phpr** (Δ vs WP-64) | gamba **oracle** (Δ) | lettura di S-94.0 | lettura vera |
|---|---|---|---|---|
| media peak | 1186,9→1170,8 MB = **−1,4%** | 393,7→346,3 MB = **−12,0%** | «REGRESSO» | **il numeratore NON è cresciuto**: il rapporto sale perché scende il denominatore |
| full CPU | 788-800u → **796,78u** (dentro il range) | ~376-386u → 421,51u = **+9,3/+12,0%** | «il più basso mai registrato» | **phpr è PIATTO**: il record è dell'oracle che rallenta |
| full peak | 2,035-2,049 → **1,993 GB = −2,4%** | — | «MEGLIO» | **dentro** lo spread inter-coppia documentato (70 MB = 3,4%) ⇒ rumore |
| media CPU | 53,85→55,50u = **+3,1%** | 20,92→21,03u = +0,5% | «poco peggio» | **l'unico movimento vero di phpr**, al bordo dello spread A-A′ (1,26-3,16%) ⇒ SCREEN |

Ho falsificato due mie ipotesi prima di scriverlo: (a) «u+s vs user» — il
rapporto user-only è 1,890 vs 1,873, artefatto ESCLUSO; (b) «workload
cambiato» — il fail-set 2F+86W = 88 è byte-identico a run33, ESCLUSO.

**Le quattro cifre sono osservate, non attribuite.** Peggio: sono attribuite
*male* — a phpr — quando l'aritmetica le attribuisce al banco di prova.
GAP_TREND porta già l'avviso, in chiaro, alla riga WP-30 («il rapporto sale
per rumore dell'oracle, non per una regressione phpr»). Il costo
dell'apparato non è solo il probe slittato: sono i **dieci minuti di
aritmetica su numeri già in repo** che avrebbero reso corretta l'unica
misura prodotta.

Chiudere così **non è accettabile**, ma il difetto non è «regresso senza
canale»: è **regresso inesistente, pubblicato in GAP_TREND come tale**.

## Emendamenti

- **A-BG-76** — GAP_TREND registra per ogni asse le **quattro cifre assolute**
  (num/den × due epoche) accanto al rapporto. Riga con soli rapporti =
  respinta alla rotazione.
- **A-BG-77** — **Δ sulla gamba phpr o non è un claim su phpr.** Le parole
  MEGLIO/REGRESSO sono ammesse solo dopo la decomposizione numeratore/
  denominatore. Correggere `REPORT_GAP_94.md`, `MEASURE94_RESULTS.md`, riga
  WP-94 di GAP_TREND e §FONDAMENTALI **prima** di S-95.0.
- **A-BG-78** — un rapporto tonight-vs-banda-storica è **SCREEN**, mai
  VERDICT: il grado VERDICT copre la coppia, non il confronto cross-epoca
  (nessun controllo stessa-sera del punto storico). Coerenza con LEVER-2.
- **A-BG-79** — **l'oracle è strumento, non costante**: R≥3 sulla sola gamba
  oracle del media group (≈21s CPU × 3 ≈ 1 min) a ogni rotazione, come
  taratura del banco. Il −12% di memoria dell'oracle a CPU invariata (+0,5%)
  è un canale APERTO e nominato.
- **A-BG-80** — il giudice della leva S-95.0 è il **contatore per-unità con
  controllo positivo** (Σ T_i ≈ 25795552 B) più la **gamba phpr assoluta**;
  la coppia conferma, non giudica. Effetto atteso sul media peak ≈ 25-39 MB
  su 1170,8 MB = **2-3%**, cioè al bordo dello spread: pretendere R≥3.

## Kill-switch

- **KS-BG-96-1** — leva giudicata da un RAPPORTO ⇒ verdetto **VOID**.
- **KS-BG-96-2** — «regresso/miglioramento» pubblicato senza le due gambe
  assolute ⇒ riga **ADVISORY**, mai VERDICT.
- **KS-BG-96-3** — S-95.0 apre la leva senza aver corretto la riga WP-94 ⇒
  **STOP**: si costruirebbe la predizione WP-48 su un «prima» mal letto.

## Ordine per S-95.0

**La leva resta prima** — ha un canale proprio, indipendente dalla coppia:
è la scelta giusta per l'oggetto. Ma prima di essa, **30 minuti**: correggere
la riga WP-94 (A-BG-77) e tarare la gamba oracle (A-BG-79). Il canale di
attribuzione del «regresso» **non serve**: non c'è regresso da attribuire.

## Che cosa sappiamo oggi che ieri non sapevamo

1. **phpr è FERMO.** Su tre assi su quattro il suo assoluto non si è mosso
   oltre lo spread in trenta sessioni: media peak −1,4%, full CPU +0,8%,
   full peak −2,4%. Nove sessioni di roadmap footprint: **nessun movimento
   misurabile sull'oggetto.** È la notizia della sessione, ed è dura.
2. **L'unico movimento reale di phpr è in PEGGIO**: media CPU +3,1%.
3. **Il banco di prova è derivato del 12%** (oracle: memoria −12% a CPU
   piatta sul media; CPU +9-12% sul full). Ogni claim cross-sessione
   costruito sui rapporti dal WP-64 in poi va riletto.
4. **La batteria WordPress nativa esiste e morde** (5 BYTE-ID + NORM-ID,
   rc=0): questo sì è avanzamento durevole dell'oggetto.
5. **`stream_get_wrappers`** è l'unica divergenza phpr su 30472 test:
   confermata, non nuova.
