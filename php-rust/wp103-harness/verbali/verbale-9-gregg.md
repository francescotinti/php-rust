# Verbale sedia 9 — Gregg (misura e attribuzione) — Concilio WP-103, mandato inverso

**VERDETTO: SESSIONE AMMESSA. Una refutazione capitale su una LEZIONE, non
sulle leve.** S-101 è una sessione d'oggetto piena: le promozioni reggono i
miei criteri (pre-registrazione, A/B interleaved contro pin, census-control
esatto). Ma una lezione va riscritta e due rumori mai misurati minacciano il
punto 1 di S-102.

## FONDAMENTALI (per la testa della sintesi)

- **Contatore sessioni-senza-misura: 0.** Misure NUOVE per NOME: prop
  12,4→11,5 pre→post stessa sera (spread ≤0,03); Δ H-C1a=7,3 ns/iter in
  banda [4,13]; Δ H-C1b=6,0 sotto banda [7,20], refutazione REGISTRATA;
  census che ARBITRA (P2 aggravata 6 coppie Rc/iter, P3 raffinata 4
  gc_note→`reg_store_slot`); `recv_clone_prop` 90M→0; il 50% di run_loop
  APERTO (21,2% dispatch + ~26,6% pila operandi); coppia WP bimodale in
  banda tra-sere. Evidenze delle promozioni: complete e coerenti
  (prop netto 4,83 == braccio A 4,81).
- **Rischio d'oggetto più trascurato**: (a) il **21,2% «corpo proprio» di
  run_loop resta SENZA NOME** — seconda quota singola del profilo, nessuna
  ipotesi iscritta; (b) il **rumore full-peak della gamba PHPR non è mai
  stato misurato** (solo l'oracle, ~10%): il +95 MiB aperto è ~5% della
  gamba phpr — può essere interamente rumore, e il punto 1 di S-102
  rischia di bisecare un fantasma.

## (2) A/B interleaved R=7 sotto rumore remoto

Regge come pattern (spread 0,04 dove la finestra sequenziale era VOID), con
un difetto: ridurre i bracci a mediane butta l'informazione di
ACCOPPIAMENTO che l'interleave compra. → A-GR-103-1.

La banda [7,20] refutata da 6,0: la lezione «i simboli inlined
SOVRACONTANO» **non è generalizzabile in forma forte**: n=1 e il segno non
è garantito — skid, righe di confine e code motion possono sovra- O
sotto-attribuire. Forma debole ammessa: costo/evento da profilo a campioni
= banda LARGA a SEGNO IGNOTO; fa fede solo l'A/B sul canale contato. Un
terzo strumento (perf counter, xctrace CPU Counters) è utile ma NON dovuto
finché nessun go/no-go dipende dalla tariffa: il criterio resta il
pavimento. → R-GR-103-1, A-GR-103-2.

## (3) Il 26,6% «meccanica pila»: errori sistematici e validazione

Innermost su atos -i ha quattro errori sistematici: **skid del PC** (un
load lento nel dispatch viene attribuito alle istruzioni successive —
proprio gli accessor Vec adiacenti: gonfia la pila a spese del 21,2%);
**righe di confine** della line-table post-ottimizzazione; **scheduling/CSE**
che fonde istruzioni attraverso i confini inline; i path `expect` contati a
parte. Il 26,6% è una banda, forse ±10 punti. Validazione dal census
S-102.2: conteggi push/pop per categoria, poi UNA leva d'emissione
(dump-diff primo giudice) e Δ A/B ÷ round-trip rimossi = ns/evento
MISURATO; se conteggio×ns implica molto meno del 26,6%×T, la quota si
CORREGGE nel file di profilo, non si tramanda. → A-GR-103-3, KS-GR-103-1.

## (4) Bozza S-102: regola di ammissione

**Composta d'OGGETTO**: punti 1-4 e 6 sono misure/leve sul motore; il punto
5 è apparato ma condizionato e legato ai gate — ammissibile. Riserva sul
punto 1: ~10 run full per una metrica il cui rumore è misurato su UNA sola
gamba; esige banda phpr e regola di chiusura pre-registrata (→ A-GR-103-4,
KS-GR-103-2) e timebox.

## Emendamenti

- **A-GR-103-1**: negli A/B interleaved pubblicare i **delta per coppia
  adiacente** accanto alle mediane; ≥3/7 coppie a segno opposto ⇒ finestra
  sospetta, si ripete.
- **A-GR-103-2**: riscrivere la lezione S-101: «costo/evento da campioni =
  banda larga a segno IGNOTO», non «sovraconta».
- **A-GR-103-3**: il census pila S-102.2 chiude il cerchio: conteggi per
  NOME + ns/evento dall'A/B, confrontati col 26,6% e quota corretta.
- **A-GR-103-4**: PRIMA del punto 1: rumore full-peak PHPR intra-sera
  (R≥5) e regola scritta: |Δ| entro banda ⇒ voce chiusa come RUMORE, senza
  bisect.

## Kill-switch

- **KS-GR-103-1**: nessuna leva sulla pila operandi passa a scrittura se il
  criterio cita il 26,6% come atteso: il criterio nasce SOLO dal
  controfattuale contato.
- **KS-GR-103-2**: punto 1 eseguito senza banda phpr misurata stessa-sera
  = VOID.
- **KS-GR-103-3**: verdetti micro in finestra con burst remoto senza ABAB
  = VOID (promozione a KS della regola S-101).

## Refutazioni capitali

**Sì — R-GR-103-1**: la generalizzazione «i simboli inlined sovracontano»
da un solo caso e a segno non garantito è refutata come legge; è ammessa
solo la forma a segno ignoto (A-GR-103-2).
