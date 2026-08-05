# Verbale Sedia 6 — Pedersen (confine per-richiesta, lifecycle, ambiente) — Concilio WP-101

## VERDETTO

**Esecuzione di R1: esemplare.** Il pin 365f4d40 è stato refutato PRIMA di ogni uso
(la refutazione "senza eseguire nulla" — manca `axum-server` — è la conferma più
forte possibile del sospetto WP-100), il pin nuovo è nato CON ricetta, la ri-parità
post-sigillo è sulla ROTAZIONE GIUSTA (a838866e @ HEAD 4c34f61, script fail-closed su
`PIN_SRV_ATTESO`, phpr ruotato a 52330330 con corpus off+on). Il sigillo è davvero il
primo atto UTILE dei due main (`logging::init()` lo precede ma non esegue codice PHP:
fuori dal modello di minaccia putenv). La nota di metodo nel `.out` è onesta.
**Ma due dichiarazioni sono più larghe dell'evidenza** — refutazioni sotto.

## Refutazioni capitali

**R1 (capitale) — «php-server collaudato: sì» poggia, per la gamba SERVER, su DUE
richieste sequenziali identiche.** Option 413 + restapi 3508 girano via phpr CLI
(stesso albero, binario DIVERSO): collaudano l'emissione, non il lifecycle del
server. La sentinella G-APERTURA-2 verifica il reset (GLOBALS/static/object_id) tra
richiesta 1 e 2 — niente payload diversi interleaved, niente richiesta 3+ (i leak
monotoni tipo WP-78 mordono dopo), niente workers>1, niente restapi via HTTP. Il
registro deve GRADUARE: `collaudato: sì (emissione CLI + smoke server 2-req)`, non
un «sì» secco. La classe «dichiarazione più larga dell'evidenza» è la stessa del pin
effetto-collaterale, spostata dal binario al verbo.

**R2 (capitale, sulla bozza S-100) — il flip del default INVERTE il significato di
«flag ASSENTE» e nessun gate della bozza lo nomina.** `reg_lower_antiputenv.rs`
braccio 1 asserisce «assente ⇒ zero forme registro»: dopo il flip è FALSO per
costruzione. `s99-parity-server.sh` dichiara «PHPR_REG_LOWER ASSENTE per
costruzione» come garanzia di flag-OFF: dopo il flip lo stesso script collauda
silenziosamente il modo NUOVO. E l'opt-out (`PHPR_REG_LOWER=0`? unset?) oggi non ha
una grafia definita né un dente che lo sigilli. Punto 2 della bozza incompleto.

**R3 (non capitale) — il registro pin chiude la classe solo per la feature axum.**
Il gate meccanico vero è la sentinella (esercita `--axum`); l'hash sha256/16
certifica «è il binario atteso», NON «è costruito con la ricetta» (altre feature —
mem-census, allocatore — non lasciano traccia; e il phpr-hash «churna col relink»
è già l'ammissione che l'hash è identità debole). Nel run di parità l'hash phpr è
«registrato, non gate»: la gamba B non è vincolata al binario che l'ha eseguita.

## Emendamenti

- **A-PE-101-1**: `PIN_REGISTRY.md` — campo `collaudato:` GRADUATO per gamba
  (emissione-CLI / server-smoke / server-N-req / server-HTTP-suite); vietato il «sì»
  secco quando le gambe differiscono.
- **A-PE-101-2**: census di TUTTI i `std::env::var("PHPR_*")` (runtime+compile) con
  classificazione eager/lazy; ogni flag lazy che tocca emissione o semantica del
  motore riceve il sigillo o la motivazione scritta. (PHPR_DUMP_OPS incluso.)
- **A-PE-101-3**: sentinella estesa: N≥16 richieste, payload interleaved diversi,
  workers>1, almeno una richiesta restapi-shaped via HTTP — prerequisito del «sì»
  server pieno, obbligatoria in modo nuovo dopo il flip.
- **A-PE-101-4**: PRIMA del flip: grafia dell'opt-out definita e documentata;
  riscrittura dei due bracci anti-putenv nel modo nuovo (assente⇒on, =0⇒off, putenv
  impotente in TUTTE le direzioni); aggiornare il commento-garanzia dello script di
  parità.
- **A-PE-101-5**: `--build-info` nei due binari (feature abilitate + HEAD): il gate
  pin verifica ricetta, non solo hash; gamba B gate ANCHE sull'hash phpr.

## Kill-switch

- **KS-PE-101-1**: il flip del default NON si dichiara eseguito finché i bracci
  anti-putenv riscritti (A-PE-101-4) e la parità server in ENTRAMBI i modi non
  passano sulla STESSA rotazione.
- **KS-PE-101-2**: nessuna cifra WordPress attribuita a un pin server la cui riga
  registro non espliciti la gamba di collaudo che la copre.
