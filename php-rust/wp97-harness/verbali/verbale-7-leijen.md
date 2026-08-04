# Verbale sedia 7 — Leijen (allocatore mimalloc v3, footprint fisico) — WP-97

Oggetto: S-95.0 (A-ZV2 F1+F2, sola misura) e §WP-96 (F3 TakeSlot, F4 coppia).

## VERDETTO

**NON REFUTATO nel merito; prosecuzione CONDIZIONATA agli emendamenti.**
Il censimento F1/F2 è pulito dal mio perimetro: conteggi esatti, deterministici,
binario di parità invariato, strumentazione confinata dietro feature. Ma la
sessione tratta TakeSlot come leva *puramente* CPU, e questo è falso dal lato
allocatore: la mossa NON elimina allocazioni, però **sposta il momento
dell'ultimo drop** (free anticipati al termine dell'operazione invece che alla
riscrittura dello slot) e **consegna valori con rc=1** a valle. Entrambi i
canali toccano footprint e pattern di riuso mimalloc, in entrambe le direzioni.
Con il regresso media footprint 3,381× APERTO e NON attribuito (WP-94), spedire
F3 senza una predizione footprint contaminerebbe l'attribuzione futura: il solo
cronometro NON basta.

## Emendamenti

- **A-DL-97-1 (predizione footprint per F4, obbligatoria).** F4 registra anche
  il peak fisico (`/usr/bin/time -l`) della stessa coppia media, con predizione
  ex-ante FIRMATA prima del run: Δfootprint atteso debolmente ≤0 (free
  anticipati + separazioni CoW evitate); una predizione nulla è comunque una
  predizione. Qualunque AUMENTO oltre lo spread A/A è falsificatore nominato e
  va ATTRIBUITO (TakeSlot vs regresso aperto), mai sommato al trend.
- **A-DL-97-2 (canale CoW non contato, da nominare ex-ante).** Un valore mosso
  arriva con rc=1: può evitare separazioni copy-on-write di array e abilitare
  il riuso in-place di PhpStr growable (WP-57) — canale che il censimento NON
  conta. Se il guadagno F4 supera la banda P3 (falsificatore "doppio della
  banda"), questa è la prima candidata: va scritta PRIMA, o il sovra-guadagno
  resta un effetto non capito. Nota a favore del nucleo `_str` nella decisione
  di perimetro F3: rischio distruttori zero E canale di riuso stringhe massimo.
- **A-DL-97-3 (modello di costo per-evento).** La banda 4,5–6,5% assume costo
  medio uniforme per lettura rc, ma `Zval::drop` (7,20%) include i drop
  DEALLOCANTI, che TakeSlot non elimina: li anticipa soltanto. Se il controllo
  positivo (`slot_reads_avoided`) centra la predizione ma il Δt manca la banda
  per difetto, il verdetto è «modello di costo sbagliato», non «leva fallita» —
  la reazione va decisa prima di misurare.
- **A-DL-97-4 (churn di purge).** Con `MIMALLOC_PURGE_DELAY=0` i free
  anticipati possono indurre purge/ricommit ravvicinati (madvise churn, page
  fault). F4 usa lo stesso env del riferimento; se il Δt PEGGIORA, si guarda il
  canale purge prima di incolpare l'opcode.

## Kill-switch

- **KS-DL-97-1**: MAI leggere un footprint da una build `zval-census`:
  l'analisi ritiene `movable`/`movable_safe` per funzione e alloca nel punto
  fisso (clone di bitset per arco per iterazione). Footprint SOLO dal binario
  di parità.
- **KS-DL-97-2**: la lettura footprint di F4 (R=1, gruppo media) è grado
  **SCREEN**, non verdict: serve da tripwire e da ancora di attribuzione, NON
  chiude il regresso 3,381×. Resta integro il vincolo WP-96: qualunque claim di
  picco e la leva arene per-file esigono α RI-DERIVATO su mimalloc v3 con
  PURGE_DELAY=0.

## Refutazioni capitali

**NESSUNA.** La coppia F4 proposta rivendica TEMPO, non picco: non viola il
vincolo R=1-è-SCREEN — lo violerebbe solo un claim footprint senza
ri-derivazione, che KS-DL-97-2 preclude. F1/F2 restano valide come misura del
meccanismo; gli emendamenti aggiungono i canali allocatore che il censimento,
per costruzione, non vede.
