# Verbale sedia 6 — PEDERSEN (confine per-richiesta, lifecycle server) — Concilio WP-106 su S-104

## VERDETTO: CON EMENDAMENTI (nessuna refutazione capitale)

## R-PE-106-n (refutazioni)

**R-PE-106-1 — Il pin server fermo a 37312e8 è disciplina sana SOLO con la
clausola negativa esplicita.** «Si rigrada col pin che porterà cifre» regge
perché in S-104 NESSUNA cifra server è stata prodotta. Ma il binario
31aa7c2e non contiene assert nested-Ref, predicato `is_trivial_drop`, denti
né census: qualunque osservazione server fatta oggi («il server va») NON
prova nulla sul runtime di HEAD 090e2eb. La deriva comincia nell'istante in
cui un'osservazione su 31aa7c2e viene citata come se parlasse di HEAD.
Finché la clausola è rispettata: sana; senza scadenza nominata (vedi
A-PE-106-2): deriva.

**R-PE-106-2 — Il lifecycle del run A/B R=7 è pulito negli atti, non
nell'ambiente.** Atti corretti: daemonizzato (16:23→20:41, mai dentro un
task), regola di lettura pre-registrata ALTROVE, zero build in finestra,
uploads-guard con restore verificato count=3, backup conservato, nessun
processo residuo. MA il regime-shift infra-run (pavimento −90 MiB su
ENTRAMBI i bracci, coppie 5-6) prova che l'ambiente non era controllato per
4h18: accettabile UNICAMENTE perché la metrica full-peak è chiusa per
sempre. E il backup uploads è oggi un asset eterno senza criterio di purge.

**R-PE-106-3 — «Runtime parity-null ⇒ server parity-null» è un'inferenza,
non un teorema.** Batteria/corpus/micro sono processo-per-test: non
esercitano request_end, RetainSet persistente, capture-boundary
(binding-output-capture), sweep distruttori, interleave worker. I commit
parity-null (OBS, debug_assert compilati via, census env-gated sotto
env -i) hanno superficie server-side bassa ma non nulla. Il rischio vero
del debito coppia WP è l'ATTRIBUZIONE: ogni sessione di rinvio allarga la
finestra 37312e8→HEAD, e la prima regressione server-side trovata si
spalmerà su N sessioni di commit — il costo del bisect cresce linearmente
col rinvio. Il riferimento 1,89×/2,64× invecchia (WP-102) mentre la
crescita peak B>A è FIRMATA (7/7, p=0,0078) e non attribuita.

## A-PE-106-n (emendamenti)

**A-PE-106-1**: cifre server/WP citabili SOLO da un server pin costruito
allo STESSO HEAD del pin phpr firmatario E gradato PIENO (option 413 +
restapi 3508 per NOME, env -i, 2 modi, mode-probe). Coppia WP su pin
difforme o su grado minimo = VOID.

**A-PE-106-2 — scadenza del debito**: coppia WP + rebuild server + grado
PIENO scattano alla PRIMA leva spedita O alla chiusura di S-106 (due
sessioni ulteriori), quale viene prima. Se scatta il ramo temporale, il
PIENO si spende comunque sul rebuild @ HEAD di chiusura: il debito smette
di essere «della prossima leva» e diventa «di calendario».

**A-PE-106-3**: retention per NOME del backup in `uploads-backups/`: voce
a registro con data e criterio di purge (es. si elimina al PASS del
restore successivo). Un backup eterno non nominato è deriva d'asset.

**A-PE-106-4**: nel design per-fase (A-LE-105-5) il controllo d'ambiente
entra NEL launcher: pavimento/regime registrati per braccio, coppia
scartata se il pavimento salta oltre banda pre-registrata. Il regime-shift
di R=7 non deve poter contaminare la metrica nuova.

## KS-PE-106-n (kill-switch)

**KS-PE-106-1**: cifra server/WP prodotta su pin non-same-HEAD o non
gradato PIENO = VOID senza appello (estende KS-PE-100-3 al confine HTTP).

**KS-PE-106-2**: se alla chiusura di S-106 la coppia WP non è rieseguita,
il riferimento WP-102 decade da «baseline» a «storico»: nessun report può
più citarlo come stato corrente.

## Priorità S-105 (perimetro Pedersen)

1. Se una leva viene spedita: rebuild server @ HEAD di chiusura → grado
   PIENO (A-PE-105-1/3/4) → coppia WP sul pin stesso (salda il debito).
2. Altrimenti: dichiarare a verbale il conto alla rovescia (S-106 =
   scadenza, A-PE-106-2).
3. Retention backup uploads a registro (A-PE-106-3).
4. Nessuna osservazione server su 31aa7c2e spacciata per HEAD (R-PE-106-1).
