# REPORT_GAP_101 — S-101 (2026-08-06, SOLO questa sessione)

Binario phpr **48a5d4384970d8ff** @ HEAD f808017 (H-C1a+b, default flag-on);
oracle brew 8.5.7. Strumenti: run-micro.sh (R=5, mediane, netto pavimento);
pair101.sh (`/usr/bin/time -l`, guardia uploads, reset DB, oracle prima).

## Giudice (sei micro-categorie, modo DEFAULT) — pre e post H-C1 nella stessa sera

| categoria | pre-H-C1 (725d5a1) | post-H-C1 (cumulativo) | Δ |
|---|---|---|---|
| proprietà | 12,4 | **11,5** | −0,9 (H-C1a −0,5 + H-C1b −0,4) |
| aritmetica | 12,7 | **12,2** | −0,5 (guardia scalari) |
| chiamate | 7,9 | **7,3** | −0,6 (5 gc_note/iter scalari censite) |
| stringhe | 7,1 | 7,0 | −0,1 |
| array | 4,6 | 4,3 | −0,3 |
| regex | 3,6 | 3,5 | −0,1 |

Spread ≤0,03 s su tutte le gambe della finestra post (macchina quieta).
ns/op prop: oracle 1,56 · phpr 9,67→8,94 (4,83/(30e6·18)·1e9).

## Coppia WordPress stessa-sera (pair101, due modi, binario cumulativo)

| metrica | off | on | riferimento S-100 |
|---|---|---|---|
| full CPU phpr/oracle | 794,84/419,76 = **1,894** | 792,22/419,23 = **1,890** | 1,873 (in banda tra-sere) |
| media CPU | 54,45/21,03 = **2,589** | 54,79/21,04 = **2,604** | 2,639 |
| full peak phpr | 1979,5 MiB | **1863,8 MiB** | off 1998,5 / on 1929,0 |
| full peak oracle | 720,9 MiB | 795,5 MiB | rumore intra-sera ~10% RICONFERMATO |

Peak = SOLO riferimento (bande VOID sul rumore misurato dello strumento;
regola KS-GR-102-2). Parità: media 0 fail identici; full = solo il delta
pre-esistente S-100 (`test_wp_is_stream ftp://`), invariante di modo.

## Lettura

La gamba dominante di H-C resta il costo per opcode: dopo H-C1a+b il
recuperato è ~13 ns/iter su 174 e prop siede a 11,5× — coerente col TETTO
dichiarato dal Concilio WP-102 (H-C1 non chiude H-C). Le due gambe nominate
che restano, con quota misurata: **meccanica della pila operandi ~26,6%**
(dentro run_loop) e **ciclo di vita Zval residuo ~28%** (drop-glue sui
temporanei che nessun prestito elimina + 3 clone LoadVar/iter del canale
emissione).
