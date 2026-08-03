# GIUDIZIO DI SENSO — WordPress su Axum (S-93.0 p.C, primo atto scritto, zero codice)

Sede deliberata dalla direttiva utente 2026-08-03 (§WP-93 p.C): il fronte
C si apre SOLO dopo la chiusura di B, e il primo atto è questo giudizio.
B è chiuso in questa sessione: B1 siti nominati, B2 natura decisa, B3
LEVER-2 refutata con misura e leva vera nominata col suo contatore
(evidenza: `wp93-harness/huge-sites.out`, ADVISORY).

## Le tre domande della direttiva, con risposta

### 1. Il worker pool persistente serve a WordPress o solo alla misura?

**Solo alla misura, oggi.** Il pool con cache per-thread (unit cache TL,
preludio per-thread) è la macchineria del canale di attribuzione
per-worker (slope b, canale iter-3/m91). WordPress è già servito dal
**modo nativo** (`php_cli::server::serve`), SAPI maturo con `WebRequest`
completo — ed è lì che l'accettazione WP-61 è avvenuta (5 probe su 6
BYTE-ID + admin NORM-ID, comando nel README). Nessun bisogno funzionale
di WordPress passa dal canale axum.

### 2. Modo nativo e modo axum: convergere o due prodotti?

**Due mandati distinti, dichiarati.** Il nativo è il prodotto funzionale
(WordPress, parity); l'axum è il canale di misura (per-worker, censimenti,
testimone). La convergenza (portare il SAPI `WebRequest` nell'handler
axum: `set_web_request` è thread-local e va installato SUL worker, header
di risposta di ritorno, multipart, file statici, query string) costa una
sessione piena e non sblocca oggi nessun obiettivo del programma
footprint. La scoperta B di questa sessione lo conferma nel merito: la
leva vera trovata (arena del preludio, spike per-thread) vale per
ENTRAMBI i modi e per il CLI — è a monte del canale, non nel canale.

### 3. Il costo del portage si giustifica col guadagno?

**No, oggi — con condizione di riapertura nominata.** Il programma
footprint ha davanti, nell'ordine: la leva per-file del preludio (canale
quantificato in huge-sites.out, gate di parità completi obbligatori), il
canale m91 (A-MS-53 → A-DL-57/58/60, PT-1) e l'attribuzione dello slope
per-worker. Nessuno di questi richiede WordPress sull'handler axum. La
voce si RIAPRE per nome quando il programma per-worker avrà consumato le
leve hello-grade e servirà un braccio a carico REALE (WordPress) sul
canale per-worker: a quel punto il portage del `WebRequest` sarà
l'oggetto di una sessione dedicata, con questo giudizio come premessa.

## Il debito che resta dovuto comunque (criterio 5 della chiusura fronte)

`battery61.sh` non è mai stato committato: l'accettazione WordPress è un
verdetto storico senza harness riproducibile, mai rilanciato in 31
sessioni. **Debito indipendente dalla decisione C**: ricommittare la
batteria WP-61 come gate riproducibile sul MODO NATIVO (mezza sessione),
con le probe WP-61 e il confronto BYTE-ID/NORM-ID. Va in cima al §WP-94
della rotazione.

## Stato dei criteri di chiusura del fronte Axum/php-server (delibera con l'utente)

1. Attribuzione per NOME: i sei huge nominati (arena preludio, transiente);
   lo slope ~18.8 MB/worker resta da attribuire per NOME al canale m91 —
   PARZIALE.
2. Leva applicata e misurata: LEVER-2 refutata con misura (delta nullo);
   leva per-file nominata col contatore — SODDISFATTO (esito negativo
   misurato, come previsto dalla direttiva).
3. Parità: binari pinnati INVARIATI (phpr d5ce86e3342f3926, php-server
   d45b57843eeb1375); le modifiche sorgente sono env-gated dormienti; la
   certificazione battery/corpus avviene alla prossima build in campagna
   (battery-91pre) — DA CERTIFICARE alla campagna.
4. Apparato congelato: in S-93.0 solo la quota A deliberata (A-SK-82 +
   A-AH68 + T23) — RISPETTATO.
5. Batteria WordPress riproducibile: NON fatta — debito nominato sopra.

Il fronte NON si dichiara chiuso (legge feedback-no-front-closure): si
chiude coi criteri 1 (attribuzione slope al canale m91), 3 (battery) e 5
(batteria WordPress nativa).
