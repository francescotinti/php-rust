# Verbale Sedia 6 — Pedersen (confine per-richiesta, lifecycle, server) — Concilio WP-103

## VERDETTO

**S-101: AMMESSA CON RISERVA. Bozza S-102, punto 5 (clausola server): REFUTATA.**

S-101 ha rispettato la lettera del registro (A-PE-100-3): pin 2c4242b6
costruito con la ricetta obbligatoria, dichiarato **NON collaudato** senza
travestimenti, nessuna cifra server attribuita (KS-PE-100-3 non violato).
Questo è il comportamento giusto quando il collaudo non entra nella finestra.
La riserva è sulla SOSTANZA: non è una deroga neutra, perché il runtime
imbarcato è cambiato ESATTAMENTE nella meccanica che il confine per-richiesta
osserva.

## Perché il buco è reale (non teorico)

1. **H-C1b è un MOVE dell'handle ricevitore**: elimina una coppia clone+drop
   Rc per accesso (recv_clone_prop 90M→0). Sposta `strong_count` e ACCORCIA
   la vita di un riferimento forte temporaneo. Il corpus e le 13 fixture
   provano che l'ordine dei distruttori intra-script non diverge — ma il
   punto di osservazione del server è un ALTRO: RetainSet per-richiesta,
   distruttori a request_end, **output capture PRIMA di request_end**
   (binding rule Pedersen/Stogov). Un distruttore che scatta un tick prima
   perché il temporaneo non tiene più vivo l'oggetto emette dentro o fuori
   la finestra di capture: la CLI non può giudicarlo per costruzione.
2. **H-C1a splitta gc_note** (guardia scalari): cambia cosa entra nella coda
   GC — il collettore di cicli tra richieste su processo persistente è un
   percorso che batteria/corpus non esercitano.
3. Quindi la frase della rotazione è corretta e va presa sul serio: la
   batteria copre il runtime, **non il capture-boundary**. Il registro regge
   solo finché il debito ha una scadenza.

## Refutazione capitale (una)

**RC-PE-103-1 — la clausola «se si tocca il server» del punto 5 bozza S-102.**
Confonde toccare il CODICE del server con toccare ciò che il server ESEGUE.
Il runtime del server è GIÀ stato toccato in S-101: H-C1a/b vivono in ogni
richiesta. La clausola istituzionalizza una finestra non-collaudata a durata
illimitata su un runtime a lifecycle cambiato, e mette in rotta di collisione
col divieto del registro («vietata la rotazione non collaudata oltre la
prima»): alla prossima build server il divieto scatterebbe senza che nessuno
l'abbia deciso.

## Emendamenti

- **A-PE-103-1**: il collaudo di 2c4242b6 è **debito NON condizionato** di
  S-102 — primo atto della sessione, o comunque PRIMA di qualunque cifra o
  uso del server; la condizione «se si tocca il server» è cancellata.
- **A-PE-103-2 (collaudo MINIMO che grada)**: (a) sentinella estesa bimodale
  già di precedente (16 interleaved su 3 endpoint + 4 concorrenti,
  workers=2, due modi) **con mode-probe A-PE-102-1** (modo effettivo provato
  dal log del server); (b) **dente capture-boundary NUOVO**: fixture servita
  che emette output da `__destruct`/shutdown alla request_end, richiesta
  **≥2 volte consecutive sullo STESSO worker**, byte-id tra 1ª e 2ª
  richiesta e tra i due modi — è l'unico braccio che vede il MOVE al
  confine. Senza (b) il collaudo ripete ciò che il corpus già prova.
  GRADUATO pieno (cifre server attribuibili) vuole in più option 413 +
  restapi 3508 per NOME sotto `env -i`.
- **A-PE-103-3**: finché non gradato: nessuna cifra server su 2c4242b6 e
  **nessuna nuova build php-server** — la più recente collaudata resta
  f2ab0636.

## Kill-switch

- **KS-PE-103-1**: se S-102 produce un nuovo binario php-server senza aver
  prima gradato 2c4242b6 (o collaudato il nuovo nello stesso atto), la
  rotazione è VIETATA e il registro segna la violazione.
- **KS-PE-103-2**: se sentinella o dente capture-boundary falliscono anche
  in UN solo modo, il pin si marca REFUTATO nel registro e la diagnosi parte
  dal confine (RetainSet / ordine capture), non dal runtime già coperto.
- **KS-PE-103-3**: un collaudo senza mode-probe nel log del server NON grada
  (recidiva A-PE-102-1): il modo si prova, non si presume dall'env.
