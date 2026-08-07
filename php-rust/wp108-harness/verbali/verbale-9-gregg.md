# Verbale sedia 9 — GREGG (metodologia di misura, attribuzione) — WP-108

**VERDETTO: S-106 NON REFUTATA nel nucleo d'oggetto** — è la sessione di
misura più pulita del ciclo — **con tre rilievi, due emendamenti e due
KS**. Nessuna refutazione capitale.

## Bilancio d'oggetto (mandato inverso)

Due rapporti mossi con causa NOMINATA, un server finalmente attribuibile,
un pin fantasma smascherato in 6 minuti di tentativo: S-106 ha prodotto
conoscenza di phpr, non apparato. Il grado del server è apparato che
COMPRA oggetto: prima di dde2a64d ogni cifra server era voce senza
mittente.

## Rilievi

- **R-GR-108-1 (qualità A/B H-A1)**: il protocollo regge TUTTE le
  KS-GR-107: criterio committato PRIMA (60448b3), ABAB, pavimenti
  per-binario, co-primario strutturale verificato prima del timing,
  admission con rimescolo d'inliner DICHIARATO e nessuna componente
  prezzata. La banda orientativa stavolta era giusta — ma la domanda è
  mal posta: il metodo è migliorato PERCHÉ il verdetto non dipende più
  dalla banda (decide max(pavimento 4; rumore 3,5; layout 0,67) + segno
  5/5, tutte quantità misurate o storicizzate). L'essere-giusta della
  banda non porta più peso di verdetto: questo è progresso strutturale,
  non fortuna.
- **R-GR-108-2 (prop −0,9)**: il dump prova che BinarySTDst è PRESENTE
  nel corpo di prop.php — firma il meccanismo e la direzione (−0,9 esce
  dalla banda tra-sere ±0,4). Ma prop NON ha avuto A/B sul proprio
  giudice: il suo Δ per-iter implicito (~12–13 ns) è MAGGIORE dei 7,0
  di arith, dove la fusione era l'intera tesi. Plausibile (op diverse),
  non ripartito: dentro i −0,9 può vivere churn di layout.
- **R-GR-108-3 (coppia WP)**: la media fuori banda (+2/+4% in ENTRAMBI
  i modi) è vecchia di due sessioni e senza data di rerun. Le sei
  micro-categorie sono un faro, non una copertura: se una leva regredisse
  fuori dal fascio, oggi nessuno lo vedrebbe.

## Emendamenti

- **A-GR-108-1**: l'attribuzione di prop assume la forma canonica
  «direzione + meccanismo firmati (dump), magnitudine NON ripartita»;
  vietato citare «H-A1 vale −0,9 su prop» come componente. La cifra
  ufficiale resta il micro sul pin.
- **A-GR-108-2**: la coppia WP in S-107 è DOVUTA, non condizionale — il
  «se» del punto 4 provvisorio si cancella: la condizione (leva spedita
  in S-106 E voce fuori banda ≥2 sessioni) è GIÀ vera.

## KS

- **KS-GR-108-1**: una voce fuori banda che sopravvive DUE sessioni
  senza rerun programmato = debito di misura in FONDAMENTALI con data;
  alla terza sessione blocca la promozione della leva successiva.
- **KS-GR-108-2**: un Δ di categoria non-bersaglio oltre banda si
  registra SOLO nella forma A-GR-108-1; magnitudine ripartita senza A/B
  sul giudice proprio = VOID.

## Rischio d'oggetto più trascurato per S-107 (nomina secca)

**La media WP fuori banda senza data di rerun** (2,697/2,734 vs
[2,57;2,64]): è l'unico strumento che vede fuori dal fascio dei sei
giudici, e sono state spedite due leve dall'ultima lettura.

## Ordine S-107: giudizio

Denti-mutanti (OBS-8, fx20-leak, dente direct-bind) in finestra corta:
giusto. **Ma il contatore hit/miss D-5 va DOPO la cura §3.15**: il fix
D-13 riscrive push_call_args, il sito stesso che il census strumenta —
eseguirlo prima significa censire un binder che muore col fix (o
pagare il rerun). La fedeltà §3.15 muove il corpus (1417→1415): è
conoscenza dell'oggetto, non manutenzione; sta bene in testa dopo i
mutanti. Sequenza emendata: mutanti → §3.15 → census D-5 sul binario
post-fix → leva → coppia WP (dovuta).

## Cosa sappiamo oggi che ieri non sapevamo (≤5 righe)

1. Il compound assign su slot costava 3 dispatch: fonderli vale +7,0
   ns/iter misurati 5/5 — e «LoadSlot mai foldato» era dogma refutabile.
2. Il meccanismo RMW-su-slot è trasversale: firmato nel dump di prop.
3. run_loop può CALARE aggiungendo un braccio (−128 B).
4. Il server di S-105 non era mai stato collaudato: le sue cifre non
   erano attribuibili; da stanotte (dde2a64d) lo sono. 1,894 è citabile.
