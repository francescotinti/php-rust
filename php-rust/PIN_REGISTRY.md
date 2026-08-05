# PIN_REGISTRY — registro dei binari pinnati (A-PE-100-3, Concilio WP-100)

Regole: ogni pin porta `collaudato: sì/no/parziale` con l'evidenza per NOME.
Vietata la rotazione non collaudata oltre la prima; vietato il pin nato come
effetto collaterale di una build (il pin si COSTRUISCE con la sua ricetta).
Nessuna cifra è attribuibile a un pin `collaudato: no` (KS-PE-100-3).

## php-server

| pin (sha256/16) | sessione | ricetta | collaudato | evidenza |
|---|---|---|---|---|
| 48a46c690fb85005 | S-99.0 | `cargo build --release -p php-server --features axum-server` @ HEAD 8d1de3b (stesso sorgente phpr S-98) | **in corso** | sentinella output-capture G-APERTURA-2 axum PASS (byte-id); option 413 IDENTICO per NOME; restapi in corso (`wp99-harness/parity-out/`) |
| 365f4d4069513de3 | S-98.0 | effetto collaterale build S-98 (vietato d'ora in poi) | **NO — REFUTATO** | manca la feature `axum-server`: `--axum` rifiuta all'avvio (recidiva WP-77.6.5.2.3); il collaudo era impossibile per costruzione |
| 832568a72b925dd1 | S-97 | effetto collaterale | NO | mai collaudato (KS-PE-99-1) |
| f8f4295a1dcdb627 | WP-94 | stash `php-server-wp94` | parziale | battery61 riproducibile + coppia full WP-94; mai restapi/option per NOME sotto env -i |
| d45b57843eeb1375 | storico | NON riproducibile a HEAD (WP-94 §Il pin che non torna) | NO | voce aperta |

## phpr (parità release, flag-off)

| pin (sha256/16) | sessione | collaudato | evidenza |
|---|---|---|---|
| 4e268c3f61e6573d | S-98.0 | sì (CLI) | corpus Zend 1418 per NOME flag-OFF e flag-ON sullo stesso albero (`wp98-harness/evidence/`); batteria 1723/0. ⚠️ l'hash churna col relink del test-funnel: fa fede HEAD; stash `phpr-s98-hb2` |
