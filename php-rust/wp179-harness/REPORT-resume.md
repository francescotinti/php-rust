# S-179 ripresa — costo e ammissibilità delle scritture private
La serie densa completa supera i criteri diagnostici aggregati: il cammino privato non-readonly osservato totalizza circa 0,42–0,43s inclusivi per suite da circa 37s. Il 68,09% dei tentativi supera le guardie censite, ma non è misurato il costo del solo sottoinsieme compatibile né un guadagno ottenibile. Nessuna ottimizzazione implementata.

**Scoreboard storico, non rimisurato:** arith 2,4 = · prop 2,6 = · calls 4,6 = · str 4,1 = · arr 3,0 = · re 2,5 =. WP t20 1,765×; ORM [6,936;7,014]×. **Leve spedite: 0.**

## Stato della ripresa
L'utente ha autorizzato la terminazione dei quattro processi yes. PID 62666/62668/62670/62672 verificati come yes, SIGTERM e scomparsa verificata. E2 nuovo prima delle build: 85,1 / 86,6 / 69,8 / 74,3% CPU totale, quattro campioni a 30s. Data circa 19,7GiB all'avvio, sopra 10GiB. Nessun altro processo terminato. Pre-flight precedente fallito preservato separatamente in phpr-s179-cost-results.

Sorgente diagnostico a1eb40e8 estratto in deposito separato, toolchain Rust stable 1.98.1 verificata; build release fat LTO/1CGU, SOURCE_DATE_EPOCH=0/CARGO_INCREMENTAL=0. Due build sequenziali riuscite: controllo puro e sonda, target nuova /Users/francescotinti/Claude/phpr-s179-cost-target. Il controllo è copiato prima della build sonda, nessuna build durante le suite. Nessuna modifica ai crate del checkout, nessun pin/promozione. HEAD resta non verde; gc-idle/L-CM1 non promossi.

Serena disponibile, attivato php-rust e letti simbolicamente prop_set_entry, resolve_prop_access e write_property_at sul repository canonico. Vexp disponibile ma indice limitato a 2000 nodi, oltre limite e incompleto: non usato come prova di copertura.

## Validazione e igiene del workload
Fixture nuova: output `12:4:9:11` e stderr vuoto byte-identici fra oracle, pin S-175, controllo puro, sonda off/count/time(rate=1), tutti rc=0. Diciassette miss con controlli positivi/negativi: privato mutable, typed, readonly, scope padre su figlio con private omonima, Closure::bind, __set, hook backing, unset e typed reference. Count con ns=0; timer rate=1 osserva tutti i 17 miss e tempi positivi. Nessuna Weak o retention del ricevente nella nuova sonda.

Primo census scartato: estrazione Python del tarball includeva 6.957 AppleDouble `._*`, interpretati da PHPUnit come test. Rimosse soltanto le copie locali, elenco conservato. Secondo census passa riepilogo/fail-set ma ha +10 miss private-readonly: differenza isolata alla classe PHPUnit Framework TestStatus Risky, dovuta ai dieci stati rimasti nella cache dal primo run scartato. Conservati raw dei due tentativi. Cache pulita dalla suite S-178 copiata con provenienza/hash e ripristinata prima di ogni corsa successiva; nessun cambiamento al tarball o ai dati originali.

**Census definitivo count-frozen:** 3484 test, 11989 assertion, 3E/13F/59S/2I, rc PHPUnit=2; baseline16 per nome identico. 4.979.840 miss, 1.795.090 private non-readonly e 2.438.339 private readonly, esattamente S-178. È parità di riepilogo/fail-set, non full-output parity della suite.

## Quota compatibile con le guardie censite
Sette bit indipendenti: scope==classe del ricevente, slot disponibile, controlli hook eseguiti e negativi, assenza strutturale di __set, assenza lazy/enum, slot presente, riferimento compatibile con typed_refs. Richiesti tutti i bit e completamento riuscito del cammino lento. Asymmetric visibility già superata al punto di osservazione; readonly esclusa. Bit aggiuntivo distingue typed: coercizione deve rimanere nel futuro hit.

**1.222.283 tentativi** superano queste guardie: **68,090% delle private non-RO**, **24,545% dei miss**. Di questi 568.775 untyped e 653.508 typed. Sono opportunità per il perimetro conservativo proposto, non hit dimostrati, non scritture evitabili e non una prova completa di correttezza del futuro fill.

| Ricevente | Private non-RO | Compatibili | Esclusione principale |
|---|---:|---:|---|
| PDOStatement | 400.472 | 400.472 | nessuna nei tentativi osservati |
| Pdo\Sqlite | 144.258 | 0 | scope diverso e slot non disponibile |
| Doctrine\ORM\UnitOfWork | 77.659 | 77.659 | nessuna nei tentativi osservati; typed |
| Doctrine\ORM\PersistentCollection | 70.513 | 70.513 | nessuna nei tentativi osservati; typed |

571.661 tentativi falliscono insieme scope/slot/presenza; altri 1.146 falliscono la guardia sui riferimenti tipizzati (1.144 OutputFormatterStyleStack, 2 ArrayCollection). Le cause si sovrappongono: non sommare i margini dei bit. Nessuna esclusione aggiuntiva hook/magic/lazy nel sottoinsieme private non-RO classificato di questa suite; le fixture negative verificano hook e magic, oltre a scope/slot/ref. Il bit instance (lazy/enum) è letto dal codice ma non ha una fixture negativa dedicata: limite di copertura dichiarato.

Lettura statica: resolve_prop_access assegna slot=None per privato nello scope del padre su oggetto figlio, per evitare indici di layout stantii. write_property_at ripiega sul nome ricevuto se l'indice fallisce; l'hit corrente passa il nome semplice. Il futuro fill deve conservare la key mangled nel fallback, scope, lifecycle e coercizioni. Il censimento non modifica questo comportamento.

## Metodo temporale e limiti
Criterio preventivo in criterio-resume.md. Timer Instant monotono campiona i miss con pseudo-casualità e seed dichiarato, densità iniziali 1/256 e 1/1024; nuova serie 1/16 e 1/64 secondo criterio-dense.md, scritto prima dei relativi run. Regione: dopo IC miss, prima del rilascio finale del ricevente; include percorso sincrono resolve/coerce/write/gc_note, esclude forwarding lazy precedente e PHP differito di hook/magic. Classificazione private/RO dentro regione; stringhe metadati e aggregazione fuori. Le guardie aggiuntive sono calcolate solo nella modalità count, che non produce tempi.

La fixture rate=1 verifica il funzionamento del timer, non la sua accuratezza rispetto a una durata nota. Floor per processo: 10.001 letture vuote, mediana/p95 registrate, sottrazione per campione con clamp a zero. Stima sum(ns-floor)×rate dai campioni della suite stessa: nessuna moltiplicazione di conteggi per microcosti. Si tratta di tempo monotono inclusivo, non CPU esclusiva; denominatore è real della stessa corsa. Coercizioni come __toString possono rientrare nella VM: campioni annidati possono sovrapporsi, quindi il rapporto della somma al wall NON è una quota esclusiva del tempo né un tetto rigoroso allo speedup. User CPU separata misurata da /usr/bin/time -lp, non dal rusage del supervisore che include le sentinelle.

La lettura dei metadati può riscaldare il ricevente campionato; perturbazione globale e variazione di densità non eliminano ogni bias locale. Nessuna stima temporale dei soli ammissibili è ottenuta: il timer classifica privato/non-RO ma non ricalcola le guardie. L'eventuale costo è un budget diagnostico dell'intero cammino osservato, mai uno speedup dimostrato o integralmente recuperabile.

## Serie rada respinta prima della terza replica
Undici corse valide conservate: tre controlli puri, quattro off, due time256 e due time1024. Tutte passano riepilogo e fail-set ORM. Le stime inclusive private non-RO sono 0,447366656 e 0,442305280s a 1/256; 0,490959872 e 0,475827200s a 1/1024. Una mediana di tre valori rimane fra i primi due già osservati: anche la terza replica più favorevole lascerebbe uno scarto minimo del **5,9813%**, oltre il 5% prescritto. Serie respinta; terzo off cancellato durante il pre-flight, prima del workload, senza rilassare soglie. Le cifre rade non costituiscono un costo accettato.

Il criterio denso è stato fissato successivamente e prima delle nuove corse: R=3 off/time16/off/time64, semi 20011/30011/40009, medesimi binari e suite. I tre controlli puri precedenti restano il riferimento di compilazione; non sono nuovamente intercalati nella serie densa. Il confronto off nuovo/vecchio e la deriva dei controlli riducono ma non eliminano questo limite temporale.

## Serie densa conclusa — criteri aggregati superati
Dodici corse valide (tre off/time16/off/time64), tutte con parità del riepilogo e dei sedici nomi baseline. Mediana CPU utente off 35,67s; time16 35,97s (**+0,841%**), time64 35,65s (**−0,056%**). Off nuovo/puro precedente **−0,419%**, off nuovo/off precedente **−0,266%**, deriva primo/ultimo off **0,810%**: tutti entro 5%. Il valore negativo time64/off è variabilità fra corse, non un'accelerazione causata dalla sonda.

| Densità | Campioni private non-RO nelle tre corse | Mediana somma inclusiva | Min–max osservati |
|---|---|---:|---:|
| 1/16 | 111.808 / 111.738 / 112.662 | 0,417630s | 0,412223–0,419829s |
| 1/64 | 28.043 / 28.107 / 28.317 | 0,431558s | 0,422388–0,433454s |

Scarto delle mediane **3,228%**, entro 5%; `dense-analysis.json: valid_cost=true` indica il superamento del protocollo diagnostico aggregato, non esattezza assoluta o efficacia di una futura leva. Floor mediano/p95 **41/42ns** in tutte le sei corse campionate. I min–max sono dispersione osservata, non intervalli di confidenza. Restano i limiti di bias locale e sovrapposizione descritti sopra.

### Classi richieste: diagnostica, stabilità non uniforme
Tempi dell'intero cammino private non-RO della classe, inclusivi. Numeri in millisecondi per suite; parentesi = min–max delle tre repliche. Il criterio preventivo di stabilità riguarda il totale, non certifica separatamente ogni riga.

| Ricevente | Mediana 1/16 (min–max), ms | Mediana 1/64 (min–max), ms | Campioni aggregati 1/16;1/64 | Scarto mediane |
|---|---:|---:|---:|---:|
| PDOStatement | 55,618 (55,227–56,688) | 59,362 (58,940–61,226) | 74.703;18.867 | 6,307% |
| Pdo\Sqlite | 28,099 (27,969–28,834) | 29,798 (28,961–30,742) | 26.964;6.815 | 5,699% |
| Doctrine\ORM\UnitOfWork | 27,871 (26,245–28,126) | 25,396 (24,418–29,608) | 14.678;3.713 | 9,746% |
| Doctrine\ORM\PersistentCollection | 17,164 (17,046–17,745) | 17,863 (17,665–17,997) | 13.021;3.292 | 3,911% |

Nessuna di queste quattro classi è sottocampionata secondo la soglia preventiva di 200 campioni. Tuttavia **PDOStatement, Pdo\Sqlite e UnitOfWork superano il 5% fra densità**: conservarne l'ordine di grandezza diagnostico, non presentarne le cifre come costi per-classe stabili al 5%. `class-summary.json` riporta tutte le classi e marca quelle con meno di 200 campioni per densità.

Ripartizione descrittiva per provenienza, mediane 1/16 e 1/64 in ms: prelude 118,723/125,122; ORM 108,718/107,685; PHPUnit 101,662/103,104; dipendenze Doctrine 70,527/71,933; altro 17,155/17,756. Le mediane dei gruppi non vanno sommate come se fossero una singola corsa; PHPUnit è parte materiale del giudice ORM.

Otto tentativi densi esclusi: due pre-flight falliti e sei interruzioni durante la suite per background CPU. Tra i carichi osservati renderer Codex, top e servizi macOS (Foto/Spotlight). Nessuno di questi processi è stato terminato: la guardia arresta esclusivamente il gruppo della propria misura. Corse scartate non entrano nelle mediane. La serie rada conserva separatamente i propri scarti. Driver completato, lock proprio rilasciato.

## Raccomandazione e beneficio plausibile
**Priorità bassa al fill privato come leva per questo workload; nessuna implementazione o promozione in S-179.** Il volume di miss compatibili è ampio, ma l'intero cammino private non-RO osservato vale circa 0,42–0,43s inclusivi su corse di circa 37s. PDOStatement è dell'ordine di 55–61ms, con stabilità per-classe insufficiente al 5%; UnitOfWork e PersistentCollection sono dell'ordine delle decine di millisecondi. Questi confronti motivano la priorità diagnostica, non un tetto rigoroso al beneficio.

Un fill potrebbe evitare risoluzioni ripetute su parte dei tentativi compatibili, ma deve conservare scrittura, coercizioni typed, lifecycle e guardie. Non sono misurati hit futuri né la sola risoluzione evitabile: **nessuna percentuale di speedup è dimostrata o promessa**. Pdo\Sqlite resta fuori dal perimetro conservativo. Il censimento chiude la quota compatibile; la serie densa chiude la diagnosi aggregata nei limiti dichiarati. Per una decisione di produzione servirebbe un A/B della leva concreta, con fixture bilaterali/mutanti e i giudici di regressione previsti; non è autorizzata automaticamente da questi dati.

## Artefatti e riproduzione
Deposito persistente: /Users/francescotinti/Claude/phpr-s179-cost-resume-results. identity.json, sorgente esatto src/, binari phpr-control/phpr-probe, build log/done/sentinelle, validation.json, count.tsv/count-analysis.json, parità per nome, cache-provenance.json, removed-appledouble.json, timings.json/timing-analysis.json (serie rada respinta), dense-timings.json/dense-analysis.json/dense.done.json (serie densa completa), class-summary.json e log per ogni tentativo. final-identity.json verifica hash finali di controllo, sonda, pin s175 e oracle PHP 8.5.7; pin invariato. Revisione finale: revisione-s179-resume.md. Script sorgenti in wp179-harness; copie nel deposito. Lock esclusivo s179-cost-resume-cb7f, supervisor con watchdog 1800s e sentinelle disk/swap/CPU. Dopo ogni guardia E2 fallita fino a 10 tentativi distinti, senza abbassare soglia o usare run scartati.
