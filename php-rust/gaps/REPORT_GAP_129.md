# REPORT_GAP_129 — SOLO S-129 (2026-08-11). Az.rev. S-128 #1+#2: UNA gamba off nuova (pair109 invariata, rc=0) + gate contesa ictx/s PER GAMBA applicato a tutte (4 archiviate S-128 + leg3-off). Pin s127b ccb63dca INVARIATO.

## Cifre (raw: wp129-harness/pair-out/leg3-off/ + wp128-harness/pair-out/leg*-*/; verdetto: wp129-harness/s129-pair-legoff-verdetto.out)
| gamba | ictx/s oracle/phpr | full cpu ratio proprio | media user-only | esito gate |
|---|---|---|---|---|
| leg1-off (S-128) | 2624 / 545 | 1,909 | 2,539 | **SEGNALATA** (>1,5× med 908) |
| leg1-on (S-128) | 1279 / 210 | 1,767 | 2,463 | pulita |
| leg2-off (S-128) | 1285 / 189 | 1,805 | 2,447 | pulita |
| leg2-on (S-128) | 1271 / 220 | 1,765 | 2,455 | pulita |
| leg3-off (NUOVA) | 2273 / 223 | 1,947 | 2,506 | **SEGNALATA** |

## Riferimento RIPUBBLICATO (gambe pulite, N=3; matrice 9 celle nel verdetto)
**full = 1,758–1,805** (coppie proprie 1,765–1,805) · **media CANONICA user-only = 2,447–2,463**
(companion user+sys 2,408–2,419) · peak phpr pulite 1828–1880 MiB. Sostituisce il
bordo alto 1,909/2,539 di REPORT_GAP_128: quello misurava la contesa, non il motore.

## Reperto nuovo
Le DUE gambe segnalate sono entrambe **prime-della-sequenza** (off1 in S-128,
leg3-off unica di S-129) con ictx/s oracle 2–3× e oracle cpu BASSO (409–424 vs
441–445): indizio di warm-up sistematico della prima gamba, non contesa casuale
⇒ apertura: warm-up leg (o permutazione dell'ordine) nella ricetta pair.
