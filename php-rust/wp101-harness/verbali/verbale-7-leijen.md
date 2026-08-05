# Verbale sedia 7 — Daan Leijen (allocatore, footprint fisico, layout) — Concilio WP-101

## VERDETTO

**APPROVO CON RISERVA.** Il mio A-LE-100-1 è RISPETTATO: `pair99.sh` applica
`/usr/bin/time -l` + `MIMALLOC_PURGE_DELAY=0` su tutte e quattro le gambe,
ricetta identica a pair94, identity pinnata (phpr 4e268c3f, oracle 8.5.7
Jun-2, rustc 1.96.0). I rapporti sono derivazione meccanica dai `.time` —
verificati a mano sui raw: tornano tutti. Ma due letture del report vanno
corrette e la bozza S-100 è CIECA sul footprint del flip.

## Refutazioni capitali — SÌ (una)

**R1 (capitale) — la lettura (b) del media peak è INCOMPLETA nei report.**
"Il rapporto media si muove per la gamba ORACLE" è vera solo a metà: dai
raw, oracle 346.325.904→445.809.528 B (+28,7%) ma **phpr
1.170.785.648→1.202.701.752 B (+2,7% = +31,9 MB)** — la gamba phpr media
NON è piatta e nessun report lo dice (REPORT_GAP_99 scrive solo "phpr
1202,7 MB"; collaudo99 dichiara piatta solo la full, correttamente, ma
attribuisce il movimento dei rapporti "media/full peak" alla sola gamba
oracle). +31,9 MB senza banda di spread né attribuzione non è "piatto":
è NON MISURATO.

**R2 — il denominatore peak è uno strumento non calibrato.** L'oracle
(stesso binario brew, stessa suite, 24h di distanza) si muove +12,1% full
e +28,7% media. Nessuno ha mai misurato lo spread del peak oracle
stessa-sera: finché non esiste un controllo positivo del metro (due run
oracle adiacenti), ogni rapporto peak è uno strumento con ±30% di
incertezza sul denominatore. La lettura (a) sul FULL è invece **provata
dai raw** (pair94-ratios + pair99-ratios, entrambi output macchina):
phpr −0,45%, oracle +12,1% — la frase è supportata, la causa no.

**R3 — la coppia fotografa un binario già superseded.** Il peak 1892,56
MiB è del pin 4e268c3f (S-98); la sessione ha poi ruotato a 52330330 (col
sigillo eager). Corpus per NOME identico ≠ footprint identico. Tollerabile
solo perché S-100 ri-fotografa in modo flag-on (regola n.2).

**R4 — (c) A-LE-100-3 resta APERTO e la bozza S-100 non lo nomina.**
Oggi 186/256 col BinaryAdd. Il flip flag-on (punto 2) non aggiunge shape
(le forme registro esistono già nell'enum), ma il punto 4 (Sub/cmp
int-int stack-path) SÌ: ogni specializzazione aggiunge varianti. Peggio:
una variante col payload sbagliato allarga `size_of::<Op>()` — cioè lo
STRIDE di ogni istruzione di ogni unità compilata: unit cache, arena del
preludio (i sei huge di WP-93), icache. La bozza non ha né il budget
shape né il gate di taglia.

**R5 — il flip cambia lo stream di istruzioni e nessuno misura la taglia
unità.** H-A1: flag-on = 19→11 op/iter — meno op per corpo, stessa
taglia/op ⇒ le unità compilate DOVREBBERO calare, ma non esiste una
misura bytes-unità off vs on. La roadmap footprint (d) è ferma proprio
perché manca l'asse compile-side fresco: la coppia peak da sola non lo dà
(un calo unit cache può essere mascherato da crescita runtime nel peak).

## Emendamenti

- **A-LE-101-1**: nel collaudo flag-on di S-100, il gate footprint della
  promozione è **phpr-off vs phpr-on stessa-sera** (stesso metro, stesso
  malloc), MAI il rapporto vs oracle (R2). In più: un doppio run oracle
  media adiacente, una volta, per bandare lo spread del metro.
- **A-LE-101-2**: al collaudo flag-on aggiungere una sonda taglia-unità
  (bytes totali delle unità compilate sui sei micro o sul corpus) off vs
  on — è l'asse che rianima la roadmap footprint (d).
- **A-LE-101-3**: ogni nuova variante Op del punto 4 porta nel commit:
  N_OPS aggiornato (gate ≤255, const-assert) + `size_of::<Op>()`
  invariato dichiarato.
- **A-LE-101-4**: sanare nei report la riga media-peak con le DUE gambe
  (R1).

## Kill-switch

- **KS-LE-101-1**: se al flip il peak phpr-on supera phpr-off stessa-sera
  oltre lo spread bandato, la promozione si FERMA finché la crescita non
  è attribuita per owner.
- **KS-LE-101-2**: se `size_of::<Op>()` cresce per una specializzazione
  del punto 4, la variante è RESPINTA (ripiegare il payload), qualunque
  sia il suo D misurato.
