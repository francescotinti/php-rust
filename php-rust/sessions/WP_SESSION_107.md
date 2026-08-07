# WP_SESSION_107 — S-107: il lotto di superistruzioni dal census (tutte e sei le categorie in discesa) + la cura del variadic by-ref

**In una frase**: abbiamo contato quali coppie di istruzioni interne il
motore esegue più spesso nei sei collaudi di velocità, fuso a lotto le
sette più calde in istruzioni singole (con criterio di promozione scritto
PRIMA): quattro collaudi su sei sono migliorati sopra la soglia del
criterio (aritmetica, proprietà, chiamate, stringhe — quest'ultima a
margine stretto) e gli altri due mostrano la direzione giusta dentro il
rumore; in coda abbiamo corretto un difetto di fedeltà (gli argomenti
«per riferimento» multipli ora si comportano come nel PHP vero: 2 test
del corpus in più passano).

**SCOREBOARD** (micro R=5 sul pin finale 62a4df65, N emessi):

| giudice | S-106 | S-107 | trend |
|---|---|---|---|
| **aritmetica** | 11,6 | **9,7** | ↓ −1,9 (BinarySCSC+IncDecSlotJmp) |
| **proprietà** | 10,6 | **8,5** | ↓ −2,1 (PropGetSlot+PropSetPop+BinaryTC+IncDecSlotJmp) |
| **chiamate** | 6,3 | **5,3** | ↓ −1,0 (riscrittura Dup;StoreSlot;Pop + IncDecSlotJmp) |
| **stringhe** | 6,6 | **6,2** | ↓ −0,4 (A/B sopra soglia con margine 2,5 su spread 15: fragile) |
| **array** | 4,2 | **3,9** | ~↓ direzione firmata (A/B 4/5, Δ dentro rumore: magnitudine NON stabilita) |
| **regex** | 3,5 | **3,4** | ~↓ direzione firmata (Δ 10 < soglia 15: magnitudine NON stabilita) |

WordPress: riferimento full **1,894×** (WP-102, NON rimisurato in S-107) ·
media 2,64× · **coppia WP DOVUTA in S-108**. **Leve perf spedite: 2**
(lotto superistruzioni; cura §3.15 come fedeltà). run_loop 274.192 B
(+16,4 KB dichiarato). Contatore sessioni-senza-Δ-rapporti: 0.

**Data**: 2026-08-07 (09:33–11:0x). **Modello**: Fable 5. **Commit**:
622e757→3223150 pushati. **Ordine eseguito**: §S-107 punti 1-6 INTERI.

## Esiti secchi
1·census sui sei giudici (build 58a93c94, bigrammi per categoria) → 2·lotto derivato coi conti statici (arith 9→5 … re 13→9, s107-census-derive.out) → 3·criterio PRE-committato (622e757) → implementazione (7 op nuove, handler a metodi CONDIVISI col braccio storico; emissione ESATTA all'attesa) → A/B R=5: PROMOSSO — **4/6 sopra soglia** (arith +16,4 5/5 · prop +26,0 · calls +19,0 · str +17,5 a margine 2,5), arr/re direzione firmata sotto soglia, nessuna regressione → 4·**PIN-108 SALDO b4b1a87d** (batteria 1739/0 rc=0 · corpus 1417×2 per NOME · fixture ×5 · micro; NOTA revisore: A/B misurato su a0543213, build gemella pre-relink del pin) → 5·**cura §3.15** (maschera by-ref estesa oltre vslot in push_call_args): flip NOMINATI by_ref/by_ref_error, corpus **1415×2**, ORM 3E/13F=baseline per NOME, hk 0E/0F, golden fx21 allineato NELLO STESSO commit → **PIN FINALE 62a4df65** (stash phpr-s107b via script). Revisione (lente misura) in `wp107-harness/revisione.md`: 5 azioni, recepite in §S-108.

## ⭐ Lezioni (max 3)
- ⭐⭐ **Il lotto batte la leva singola quando le finestre sono derivate dai DATI**: il census (fermo da WP-33) ha nominato 7 fusioni in un colpo; l'attesa statica per giudice ha predetto il segno di TUTTI gli A/B.
- ⭐⭐ **Due regole hanno morso di nuovo chi le conosceva**: `tee` prima del `mkdir` (log perso) e rc di batteria incatenato a una pipe (run annullata e rifatta) — vanno applicate alla FORMA del comando, sempre.
- ⭐ **Un conteggio che cala di 1 si spiega con l'INVENTARIO, non si liquida**: batteria 1740→1739 riconciliata con `git grep '#[test]'` identico (1767=1767) fra i due commit; artefatto di conteggio run, dichiarato a verbale.
