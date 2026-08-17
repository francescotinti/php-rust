# WP_SESSION_151 — concilio ratifica Zval-first; dente A4 armato; census tranche-5 VALIDO (i canali Zval hanno i loro primi numeri)
**In una frase**: nove revisori indipendenti hanno approvato con correzioni il
piano di ristrutturazione della memoria del motore, è entrato in vigore un
freno automatico alla crescita dei file, e il censimento ha misurato per la
prima volta quanto «traffico» di oggetti fa davvero la suite Doctrine.

**SCOREBOARD** (pin s150 INVARIATO cbbe71735effb165 + 18c2740774336c82):
arith 5,5 → · prop 5,5 → · calls 4,8 → · str 4,3 → · arr 3,3 → · re 2,5 →
(non rimisurate: nessun pin nuovo) · WP t4 mediana 1,781 (rif.) · ORM
7,104–7,149 · dbal 7,283–7,491 · corpus 1412×2 · **leve perf spedite: 0 —
ANOMALIA DICHIARATA** (coperta da rotta utente 2026-08-17 + concilio) ·
incidenti 18→**19** (probe s149 «conservato» SENZA path = non reperibile).

## Esiti secchi
1·**Concilio a 9 (fascicolo Gemini)**: 9× CONCORDO CON EMENDAMENTI. Ratifica
  emendata: A2 **chirurgia-first** ~3 sessioni (touch-map da A1; run_loop
  ultimo/mai) · A3 FUORI AGENDA senza numeri; forma: store bucket+free-list
  alla Zend campo della Vm, handle LINEARE non-Copy, weak=(id,gen), coda
  decrementi+shadow-mode; A3.0 = sweep-preserving (dissenso agli atti) ·
  soglie GO/NO-GO cumulative PRE-registrate · cifre Gemini mai nei criteri ·
  veto WP-44 confermato. 2 quesiti utente. `wp151-harness/concilio/`.
2·**Dente A4 ARMATO in batteria**: `loc_dente.rs` (nuovi ≤2.000; allowlist
  21 cap ESATTI; anti-slack 200); collaudo negativo rc=101 con messaggio
  esatto + positivo verde. Inventario batteria: s125+rczval+loc_dente.
3·**Probe tranche-5**: identità ricetta = pin ESATTO cbbe7173; smoke 25/25
  su attesi PRE-dichiarati; probe ab02faec0abfab67 CONSERVATO in
  `wp151-harness/census-prep/phpr-census-s151` (path+hash a verbale).
4·**CENSUS VALIDO rc=0** (×2 repliche IDENTICHE; identità e conservazione
  esatte su 2.393 classi): C2 borrow **340,9M** · C1 handle clone/drop
  **254,0M** · C5 valori PropGet/Set **191,2M** · C4 gc_note **43,2M** ·
  C3 alloc **6,4M** — **domina il teardown/sweep**, non il prop-access.
  N2: p50=1/p90=6/p99=18, ≤8=92% ⇒ inline-8 fondato. live_end 5,5%.
  Testa hostcall 82,2M: **residuo non-backtrace INVARIATO vs s149 (+0,16%)**
  — staleness post-BT1 provata; §5-bis fuori bande ⇒ scarto +3,2% a diff
  sorgente (S-152). debug_backtrace ANCORA #1 (21,3M). Lettura:
  `s151-census-lettura.md`.

## ⭐ Lezioni (max 3)
- ⭐⭐ Una conservazione dichiarata SENZA path non conserva nulla: ogni asset
  «conservato» entra a verbale con path+hash (fatto: probe s151).
- ⭐⭐ Le cifre census invecchiano con le leve promosse: testa 335,8M→82,2M
  post-BT1 col residuo non-bt a +0,16% — rifondare PRIMA di citare.
- ⭐ Un conteggio frame-granulare attribuito a un builtin traveste un outlier
  algoritmico da fatto di memory-model: le teste per NOME si leggono con la
  granularità dichiarata.
