# PIN_REGISTRY — registro dei binari pinnati (A-PE-100-3, Concilio WP-100)

Regole: ogni pin porta `collaudato: sì/no/parziale` con l'evidenza per NOME.
Vietata la rotazione non collaudata oltre la prima; vietato il pin nato come
effetto collaterale di una build (il pin si COSTRUISCE con la sua ricetta).
Nessuna cifra è attribuibile a un pin `collaudato: no` (KS-PE-100-3).

## php-server

| pin (sha256/16) | sessione | ricetta | collaudato | evidenza |
|---|---|---|---|---|
| f2ab06369239f389 | S-100 (FLIP: register-lowering ON di default) | `cargo build --release -p php-server --features axum-server` @ HEAD fb861e4 | **sì — GRADUATO (A-PE-101-1): emissione-CLI ✓ (due modi) · server-sentinella-estesa ✓ (16 interleaved su 3 endpoint + 4 concorrenti, workers=2, due modi) · server-HTTP option+restapi via phpr CLI per NOME ✓ (due modi)** | run env -i 2026-08-05: modo off 22:41-22:48 fails=0, modo on 22:49-22:56 fails=0 (`wp100-harness/parity-out-{off,on}/`, launcher bimodale `s100-parity-server.sh`); braccio OFF collaudato alla rotazione (KS-HE-101-3); stash `php-server-s100-flip` |
| 9db61be0c11cedea | S-100 (candidato pre-flip, estensione BinaryAdd) | stessa ricetta @ HEAD ~6c05775 | sì (superseded da f2ab0636) | parità bimodale fails=0 nei due modi (prima esecuzione ASSOLUTA del server flag-on), sentinella estesa PASS |
| a838866e134b6a20 | S-99.0 (sigillo eager) | `cargo build --release -p php-server --features axum-server` @ HEAD 4c34f61 | **sì** | run unico rc=0 sotto env -i (2026-08-05 14:17): sentinella output-capture PASS + option 413 IDENTICO per NOME + restapi 3508 IDENTICO per NOME (`wp99-harness/parity-out/`); stash `php-server-s99-sigillo` |
| 48a46c690fb85005 | S-99.0 | `cargo build --release -p php-server --features axum-server` @ HEAD 8d1de3b (stesso sorgente phpr S-98) | **sì** (superseded da a838866e) | run unico rc=0 sotto env -i (2026-08-05 13:03): sentinella output-capture G-APERTURA-2 axum PASS (byte-id) + option 413 IDENTICO per NOME + restapi 3508 IDENTICO per NOME (`wp99-harness/parity-out-run2-pin48a4/`, script `s99-parity-server.sh`); stash `php-server-s99` |
| 365f4d4069513de3 | S-98.0 | effetto collaterale build S-98 (vietato d'ora in poi) | **NO — REFUTATO** | manca la feature `axum-server`: `--axum` rifiuta all'avvio (recidiva WP-77.6.5.2.3); il collaudo era impossibile per costruzione |
| 832568a72b925dd1 | S-97 | effetto collaterale | NO | mai collaudato (KS-PE-99-1) |
| f8f4295a1dcdb627 | WP-94 | stash `php-server-wp94` | parziale | battery61 riproducibile + coppia full WP-94; mai restapi/option per NOME sotto env -i |
| d45b57843eeb1375 | storico | NON riproducibile a HEAD (WP-94 §Il pin che non torna) | NO | voce aperta |

## phpr (parità release; da S-100 il default è flag-ON, opt-out `PHPR_REG_LOWER=0`)

| pin (sha256/16) | sessione | collaudato | evidenza |
|---|---|---|---|
| 725a2ffad763bbc4 | S-100 (FLIP + contratto value-parsed + estensione BinaryAdd) | **sì — GRADUATO: CLI due MODI** | @ HEAD fb861e4: corpus Zend 1418 per NOME IDENTICO a wp82 con `=0` E `=1` ESPLICITI sul pin (`wp100-harness/corpus-gate/`); diff per-test off↔on ZERO fuori carve-out nominata (albero candidato stessa emissione, `evidence/corpus-diff-verdetto.txt`); batteria post-flip 1735/0 rc=0; smoke default: assente⇒registri, `=0`⇒pila, output identico; coppia WP stessa-sera con bande (G1-G4, `pair100-bande.out`); stash `phpr-s100-flip`. ⚠️ l'hash churna col relink del test-funnel: fa fede HEAD |
| 52330330873f0132 | S-99.0 (sigillo eager A-PE-100-2) | **sì** (CLI) | corpus Zend 1418 per NOME IDENTICO alla lista wp82, flag-OFF E flag-ON sullo stesso albero @ HEAD 4c34f61 (`wp99-harness/evidence/corpus-s99-{off,on}.fails`); batteria 1726/0 rc=0 (con i 2 denti anti-putenv + funnel); stash `phpr-s99-sigillo`. ⚠️ l'hash churna col relink del test-funnel: fa fede HEAD |
| 4e268c3f61e6573d | S-98.0 | sì (CLI, superseded) | corpus Zend 1418 per NOME flag-OFF e flag-ON sullo stesso albero (`wp98-harness/evidence/`); batteria 1723/0. Stash `phpr-s98-hb2` |
