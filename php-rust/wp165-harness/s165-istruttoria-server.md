# S-165 p.3 (az.rev. S-164 #4) — istruttoria non-riproducibilità php-server

## Ipotesi NOMINATA (da sorgente, costo zero)
La ricetta CANONICA del server (scripts/pin-server.sh, righe 14-16) è:
`SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0 cargo build --release -p php-server
--features axum-server` — con la FEATURE `axum-server`. Una ricostruzione via
build di WORKSPACE (`cargo build --release`, che compila php-server SENZA
quella feature) produce un binario NECESSARIAMENTE diverso: il 661b490c
osservato in S-164 è compatibile con questo meccanismo, non con un difetto
di determinismo della pipeline A′ (che per phpr è provato build ×2).

## Prova PRE-registrata (da eseguire a tree == pin s163, FUORI finestra misure)
1. `shasum` del canonico PRIMA (atteso: 8d76d6f129bfd4af dallo stash).
2. Build ricetta VERA ×2 (`-p php-server --features axum-server`, env A′):
   attesa = hash IDENTICO tra le due build E == 8d76d6f129bfd4af.
3. Esiti: (a) 8d76d6f1 riprodotto ⇒ istruttoria CHIUSA, rilievo #5 S-164
   declassato a «ricetta non citata nel tentativo»; il verbale S-164 si emenda
   DICHIARANDO (la falsa pista «pin solo come artefatto di stash» cade).
   (b) hash ×2 uguali tra loro ma ≠ 8d76d6f1 ⇒ pin server davvero orfano
   della ricetta: istruttoria drift toolchain/lockfile (cfr. d45b578).
   (c) hash ×2 DIVERSI tra loro ⇒ non-determinismo nella build server:
   istruttoria dedicata (LTO/feature unification), pin congelato dallo stash.
Esito in s165-server-ricetta-verdetto.out (build log + 2 hash + rc).
