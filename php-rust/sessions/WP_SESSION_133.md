# WP_SESSION_133 — LEVA ctor resolve-once SPEDITA (pin s133) + gate teardown + coppia WP

**In una frase**: la terza cura del modello — risolvere il nome della proprietà
UNA volta invece di due quando un costruttore con tipi la scrive — è promossa su
tutta la catena (allocazione oggetti ~4% più veloce); teardown coperto da 7 prove.

**SCOREBOARD** (pin NUOVO **s133 c87439a9**789bcef4 + server s133 d447f828; micro gate promozione):
**arith 5,5 = · prop 5,5 = · calls 4,8 ↘ · str 4,2 ↘ · arr 3,2 ↗(jitter) · re 2,5 =**
· **WP full ON-ONLY = 1,754 (N=1) @ pin s133** (off 1,781–1,808 N=2; media 2,428–2,487)
· **leve spedite: 1 (ctor resolve-once)** · incidenti: **1 NUOVO** (sed copia
dichiarata → stash sovrascritti; riparato al byte). 2026-08-13.

## Esiti secchi
1·**Fixture destructor-window (az.rev. #1)**: 7 vettori (dtor self/peer,
  gc annidata, resurrezione, dtor-walk, WeakReference, doppio gc) TUTTI byte-id
  oracle==s131==s132 ⇒ nessuna divergenza; gate `teardown` fail-closed in
  catena s109 (8 gate, emenda dichiarata). #2/#3/#5 a costo zero (c1f8f54).
2·**Sonda conteggi ctor**: split **2+2 CONFERMATO** (magic_applies + fallback
  PropSet); scostamenti dichiarati (entrate 2, datains TOT 6 vs k=9 di s130).
3·**LEVA SPEDITA** (criterio dbff54a → codice 59f1cb0): resolve hoistata
  post-hook in prop_set_entry, condivisa da magic_applies_resolved + key/slot/IC.
  R=5: **objalloc D=+46,7** (soglia 16,7; DICHIARATO fuori UB 35,4, +11,3 non
  ripartita) · **objdatains D=+30,0** (10,0) · riconciliazioni in banda ·
  guardie 8/8 · promozione rc=0 (1746/0/2, corpus 1414, fixture 8, ORM 16
  nomi, hk 0E/0F) → **pin s133**. Submicro: objalloc **7,5** (946,7, −36,6) ·
  objdatains 7,2 (1183,3) · churn 8,2 · dropdef 8,9 · allocni 9,4 · objmap 17,3.
4·**Coppia WP t1**: 3/4 gambe pulite (leg2-on esclusa, firma phpr ictx 226%);
  on-only 1,754 dentro il rif. s131 — la leva NON muove WP (coerente: morde il
  fallback non-plain, profilo ORM) · peak +80 bordo alto → REPORT_GAP_133.
5·**INCIDENTE riparato al byte**: sed della copia cieco sulle righe QUOTATE →
  promo con `pin-*.sh s132` → stash s132 sovrascritti; server da backup .SAFE,
  phpr-s132=6af6e497 ricostruito da ceb1ec2 nel tree PRINCIPALE (worktree NON
  riproduce: il path entra nel binario); PIN_REGISTRY emendato; CI sospesa in
  finestra. Revisione PROCESSO: registro pin marcio, riparazione a mano fuori
  dagli script — az.rev. vincolanti in §S-134.
## ⭐ Lezioni (max 3)
- ⭐⭐ La copia dichiarata si collauda ANCHE sulle righe che il pattern non tocca.
- ⭐⭐ Il determinismo della ricetta ripara (6af6e497 al byte dal commit giusto)
  ma SOLO nel tree originale: la riproducibilità è path-dipendente.
- ⭐ Per una leva resolve-bound basta una sonda a SOLI CONTEGGI: costo una build.
