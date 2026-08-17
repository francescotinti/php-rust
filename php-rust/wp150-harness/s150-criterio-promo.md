# Criterio S-150 p.1 — PROMOZIONE BT1 (catena piena collaudo-nell'atto) — scritto PRIMA del run

1. Candidato = **cbbe71735effb165** = build ricetta canonica
   (SOURCE_DATE_EPOCH=0, CARGO_INCREMENTAL=0, lto=fat cgu=1); la catena DEVE
   riprodurlo, pena STOP. **EMENDA (eseguita PRIMA del run, atto in
   `s150-identita-candidato.md`)**: il braccio B giudicato `ac26375a` fu
   costruito fuori ricetta (senza SOURCE_DATE_EPOCH, target separato);
   identità provata AL BYTE modulo {timestamp `__DATE__`, LC_UUID, firma}:
   contenuto codice+dati IDENTICO. Sorgente invariato da 6a7adc8 (diff vuoto).
2. Catena = `s150-promozione.sh`, copia DICHIARATA di s145-promozione.sh
   (manifest `s150-promozione-copia.diff`, collaudo copia-gate).
3. Batteria: inventario = baseline s125 + il SOLO `rczval_pattern_resta_nel_funnel`
   (estrazione dal diff 4a968b7..HEAD: ZERO `#[test]` nuovi; census tutto dietro
   `#[cfg(feature="mem-census")]`); `debug_backtrace_array_fields` resta verde.
4. Corpus 1414×2: flip ATTESI ⊆ famiglia backtrace (`s150-flip-famiglia.txt`).
   **EMENDA DICHIARATA (primo passaggio del gate)**: la famiglia era generata
   per NOME file (20) e mancava `backtrace/bug64239_2.phpt` (directory
   backtrace/, nome senza la parola): il handler fail-closed ha STOPPATO —
   generazione corretta a MATCH DI PATH (`-ipath "*backtrace*"`, 29 nomi),
   intento «famiglia backtrace» INVARIATO; l'esito del primo passaggio
   (flip: bug64239_2 + debug_backtrace_limit ×2 modi; contenuto mutato:
   debug_backtrace_options; off↔on ZERO) è agli atti in
   promo-out/corpus/corpus-gate.out. Bersagli DIRETTI citati dalla cura (REGOLE §9):
   `backtrace/debug_backtrace_limit.phpt`, `backtrace/debug_backtrace_options.phpt`.
   NESSUN nome extra (regressione) ammesso. Gestione = `s150-flip-handler.sh`
   fail-closed (aggiorna congelato+golden SOLO per i nomi flippati/mutati della
   famiglia, commit dichiarato) + `corpus-gate.sh --replay` che DEVE dare rc=0.
5. Fixture: catena s109 EMENDATA 9→10 gate (`backtrace` =
   s150-fx-backtrace-gate.sh, byte-parity piena nei 2 modi — la cura chiude la
   divergenza, nessuna riga a catalogo).
6. Guardie (riparazione incidente 17, az.rev.1): **R=5** ABAB su OTTO giudici
   (arith prop calls str arr re + m-dimread + m-dimrmw), SOLO-REGRESSIONE su
   mediana user; ROSSA se med_B > med_A + max(rumore drop-1 di A, 1 tick 0,01 s).
   Conferma leva: m-backtrace R=5, ns/iter netto pavimento PER-binario
   (N=150000), D=A−B atteso ≥ soglia max(4 ns/iter, drop-1); bilaterale con
   pavimento oracle MISURATO (az.rev.4, mai più RAW). Disasm bl-count di
   run_loop (metodo S-109) su A e B nel `.out` di RECORD: atteso INVARIATO
   (leva fuori dal dispatcher), difformità DICHIARATA a verbale.
7. Gate ORM fail-set == baseline 16 nomi · gate http-kernel 0E/0F ·
   pin-server.sh s150 (grado minimo). Esiti pre-registrati: qualunque gate
   rosso ⇒ promozione ABORTITA, pin resta s145, incidente dichiarato.
8. Scommessa ORM (s149-decisione-bt1.md p.4): NON si giudica qui — arbitro =
   coppia ORM al pin nuovo (p.4 dell'ordine di sessione), attesa ↓0,8–3,1 s.
