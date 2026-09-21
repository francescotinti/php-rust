# S-178 — attribuzione delle scritture nella suite Doctrine ORM

Questa indagine distingue il lavoro della libreria da quello del framework di test; non misura uno speedup.

**Scoreboard storico, non rimisurato:** arith 2,4 = · prop 2,6 = · calls 4,6 = · str 4,1 = · arr 3,0 = · re 2,5 =. WP t20 1,765×; ORM [6,936;7,014]× (S-175/S-176). **Leve spedite: 0.**

## Identità e perimetro

- Sorgente archiviato: `a1eb40e8ba8764e0ebb000c60a733e595ff1f31a`; il tree include gc-idle e L-CM1, entrambi non promossi. L-CM1 non ha verdetto prestazionale valido.
- Pin S-175: SHA-256 `5de14d6856d760a83a333361f2c293e6272d9ded4d4928483efe3aa9cc68de76`, preservato. Server non usato/modificato.
- Nuove build: `rustc +stable 1.98.1 (48a229cea 2026-09-01)`, LLVM 22.1.8, aarch64-apple-darwin, release fat LTO / 1 CGU, `SOURCE_DATE_EPOCH=0 CARGO_INCREMENTAL=0`. Nessun confronto di prestazioni tra toolchain.
- Target: `/Users/francescotinti/Claude/phpr-s178-investigation-target`. Sorgente strumentato e raw: `/private/tmp/phpr-s178-investigation`. Nessuna modifica ai crate del checkout o ai copioni canonici.
- Tarball canonico `orm-work.tgz`: SHA-256 `206cf384f4102b9522c73e378310fad1685586dd1dc124e55fdfd2597220d1bb`. ORM `3.6.x-dev`, reference `2bae808d73d9559dc2720ddfe3be19d3e4a2a0c6`; DBAL `4.4.3`, reference `61e730f1658814821a85f2402c945f3883407dec`.
- PHPUnit **11.5.56**, reference `5f83edffa6967c3db468d48a695ec7bcb02e9256`: confermato da Composer, `Runner/Version.php` e log S-177. La menzione di PHPUnit 13 nella revisione S-177 è errata. Il sorgente estratto contiene 522 dichiarazioni `readonly class` in PHPUnit, 0 in ORM; ORM contiene però 118 dichiarazioni `private readonly`: assenza di readonly class non significa assenza di proprietà readonly.
- Oracle `/opt/homebrew/opt/php/bin/php`, SHA-256 `07b0df8d6324769540987f2213070a6ba90eba667aa56b34c64e6c770a300a36`.

## Metodo e limiti

Criterio scritto prima della build in [criterio.md](criterio.md). Sonda iniettata soltanto nell'archivio: nessuna nuova feature di produzione. Non usa `op-census`, così non aggiunge contatori a ogni opcode. Registra le chiamate di `prop_set_entry` **dopo il lazy forwarding riuscito**, incluse inizializzazioni, hit e miss. Non copre ogni forma di scrittura del runtime (es. Reflection o altri handler).

Classe ricevente e file della funzione PHP corrente sono due attribuzioni distinte: non è un profiler dello stack o del tempo. L'incrocio private/readonly usa la key effettiva e `ro_decl` sul cammino lento; `-1` significa non classificato, mai falso. Le categorie sono esclusive, diversamente dalle percentuali sovrapposte di S-177.

`object_repeat` conta tentativi precedenti sullo stesso oggetto e sulla stessa chiave storage osservati dalla sonda; può includere inizializzazioni e tentativi falliti, non prova scritture riuscite. La Weak distingue il riuso dell'ID senza tenere vivo il contenuto dell'oggetto. Due private omonime padre/figlio sono distinte. `site_repeat` è condizionato a sito IC×classe×file×percorso×categoria: non è un conteggio globale di siti e non dimostra da solo che un fill sia semanticamente ammissibile. Hook/magic non risolti non alimentano la storia dello slot.

Weak, mappe e allocazioni alterano il costo e la memoria del processo: **nessun tempo del census è utilizzabile**. Il sorgente della sonda non contiene il timer esplorativo rimosso prima della corsa registrata; la colonna finale TSV vale sempre zero, non è un costo.

## Risultati verificati

Sonda `0cef791ff4c19d9446d771afa742d8510ad5986e0342c9845de851c525c71324`. Build finale rc=0. Fixture: oracle=pin=sonda byte-identici; 32 chiamate, categorie e ripetizioni esatte (incluse private omonime padre/figlio e riuso degli ID su nuove istanze readonly).

**ORM con sonda attiva e spenta:** entrambi rc=2, 3484 test, 11989 assert, 3 errori, 13 failure, 59 skipped, 2 incomplete; i 16 nomi coincidono esattamente con il baseline. È un gate di invarianza riuscito, non una suite senza fallimenti.

Totale handler: **8.998.101**, di cui 9.219 inizializzazioni. Escluse queste: **8.988.882**, con 3.277.200 hit plain, 731.842 hit typed e **4.979.840 miss**. Questi quattro totali e i margini private/readonly coincidono esattamente con il raw S-177, pur senza attivare op-census.

| Categoria esclusiva nei miss | Tentativi | % dei miss | Ripetizioni stesso oggetto/slot |
|---|---:|---:|---:|
| Private ∩ readonly | 2,438,339 | 48.964% | 0 |
| Private ∩ non-readonly | 1,795,090 | 36.047% | 535,617 |
| Non-private ∩ readonly | 512,099 | 10.283% | 0 |
| Non-private ∩ non-readonly | 234,304 | 4.705% | 45,099 |
| Non classificati | 8 | 0.000% | 0 |

Le private non-readonly sono **36,05% dei miss e 19,97% delle scritture non-init osservate**, non un perimetro vicino a zero. Di queste 1.795.090, 535.617 (29,84%) ripetono lo slot dello stesso oggetto; 1.777.221 ripetono la stessa combinazione sito/classe/file/percorso/categoria. Le private readonly hanno zero riscritture osservate sul medesimo oggetto/slot, ma **2.437.117 ripetizioni di sito** su 2.438.339 tentativi: write-once per oggetto non significa sito freddo.

### Attribuzione per classe ricevente

| Famiglia | Tutte le entrate (incl. init) | Miss | Private non-readonly nei miss | Readonly nei miss |
|---|---:|---:|---:|---:|
| PHPUnit | 1,803,390 | 1,803,237 | 211,900 | 1,591,274 |
| Doctrine ORM | 1,385,051 | 1,082,253 | 403,230 | 645,764 |
| Doctrine dependencies | 1,548,700 | 1,133,397 | 280,568 | 695,811 |
| Doctrine tests | 195,978 | 134,982 | 129,513 | 1,295 |
| Other | 4,064,982 | 825,971 | 769,879 | 16,294 |

PHPUnit è la famiglia singola più grande nei miss (36,21%), ma non la maggioranza. Dei 2.950.438 tentativi readonly, 53,93% riguardano classi PHPUnit: **46,07% sono altrove**. La precedente deduzione «readonly = eventi PHPUnit» era troppo ampia. Le dipendenze Doctrine comprendono soprattutto DBAL e non sono ORM core.

### Attribuzione per file PHP che esegue la scrittura

| Origine | Entrate (incl. init) | Miss |
|---|---:|---:|
| Runtime prelude | 3,956,010 | 769,755 |
| PHPUnit | 1,901,890 | 1,901,822 |
| Doctrine dependencies | 1,653,595 | 1,186,121 |
| Doctrine ORM | 1,287,428 | 1,034,880 |
| Doctrine tests | 69,613 | 17,226 |
| Sebastian dependencies | 40,596 | 40,577 |
| Other | 88,969 | 29,459 |

Il `prelude` è codice PHP interno con cui phpr implementa parte delle classi standard: non è sorgente di Doctrine né PHPUnit. Esegue 3.956.010 entrate (43,96%), prevalentemente hit; non è però una quota di tempo e non identifica il chiamante applicativo a monte. ReflectionMethod/ReflectionClass sono i maggiori riceventi per numero di entrate (1.346.484 / 1.309.810), un ulteriore motivo per non identificare frequenza con costo.

### Top 10 classi per miss

| Classe | Miss | Private non-readonly | Readonly |
|---|---:|---:|---:|
| `PDOStatement` | 400,473 | 400,472 | 0 |
| `PHPUnit\Event\Telemetry\GarbageCollectorStatus` | 363,636 | 0 | 363,636 |
| `Doctrine\ORM\Mapping\Column` | 328,594 | 0 | 328,594 |
| `PHPUnit\Metadata\MetadataCollection` | 204,620 | 0 | 204,620 |
| `Doctrine\DBAL\Schema\Name\Identifier` | 185,704 | 0 | 185,704 |
| `Doctrine\DBAL\Schema\Identifier` | 174,333 | 114,996 | 0 |
| `PHPUnit\Event\Telemetry\Info` | 151,510 | 0 | 151,510 |
| `Pdo\Sqlite` | 144,258 | 144,258 | 0 |
| `PHPUnit\Event\Telemetry\Duration` | 128,174 | 0 | 128,174 |
| `PHPUnit\Event\Telemetry\Snapshot` | 121,212 | 0 | 121,212 |

I 400.472 miss private non-readonly di PDOStatement e i 144.258 di Pdo\Sqlite mostrano un bersaglio distinto dalla telemetria PHPUnit. Non sono automaticamente tutti ammissibili al fill proposto: Pdo\Sqlite eredita da PDO, e il vincolo `obj_class == scope` va rispettato. Il census non misura ancora quel sottoinsieme.

## Raccomandazione e beneficio plausibile

**Prossimo bersaglio da misurare: il cammino lento di risoluzione/scrittura delle proprietà private non-readonly, con PDOStatement come caso dominante e UnitOfWork/PersistentCollection come controlli ORM.** È una candidatura, non una dimostrazione del miglior rapporto costo/beneficio. Il perimetro dinamico è concreto, ampio anche fuori da PHPUnit, e non richiede di allargare subito l’intervento alla semantica readonly. Un fill nello scope dichiarante potrebbe evitare risoluzioni ripetute su siti già caldi, anche quando ogni oggetto è scritto una volta.

Il beneficio plausibile è ridurre il lavoro per una **parte** degli 1,795 milioni di miss; non è un 20% di accelerazione della suite. Restano da misurare costo per percorso e quota effettivamente ammissibile (scope, slot, hook, magic, typed/ref e stato lazy). Non esiste qui una percentuale di speedup difendibile o un tetto temporale numerico. Se il costo di quel sottoinsieme risultasse marginale, la candidata va scartata anche se frequente.

Passo concreto quando la CPU sarà libera: (1) misurare costo inclusivo del cammino private non-readonly sul carico completo, separando prelude/ORM/DBAL/PHPUnit e verificando perturbazione della sonda; (2) contare la guardia di ammissibilità `obj_class == scope` con slot risolto; (3) solo allora progettare il fill privato e provarlo in A/B reale, stessa toolchain e controllo nullo. Le readonly restano seconda candidata separata: 2,95M miss, ma obbligatori i controlli write-once/clone; il fatto che la maggioranza PHPUnit sia write-once non le rende siti freddi.

**Nessuna ottimizzazione di produzione implementata.** Il censimento chiude gli incroci e le attribuzioni; la classifica per costo e il beneficio quantitativo restano aperti per scelta esplicita dell’utente sulle condizioni di misura.

## Costi effettivi: limite concordato

Pre-flight: oltre 500% CPU totale, quattro `yes` (PID 62666/62668/62670/62672) prossimi a 100% ciascuno, attivi da oltre tre giorni. E2 richiede <150% per quattro campioni. L'utente ha scelto esplicitamente **lasciare attivi i processi e consegnare censimento e limite sui tempi**. Non sono stati terminati processi estranei; nessuna misura temporale valida è stata eseguita.

Non è quindi possibile affermare quale quota del *tempo* appartenga a Doctrine o PHPUnit, né quantificare un beneficio percentuale sulla suite. I ~7× restano un dato storico; non vengono reinterpretati usando i nuovi conteggi. Per il prossimo costo servono prima una finestra E2 valida e profilazione temporale rappresentativa, poi un A/B proprio della suite sul candidato, con controllo nullo e stessa toolchain. Moltiplicare questi conteggi per un costo micro non sarebbe una prova.

## Riproduzione e fonti

Comandi e controlli: [README.md](README.md), [inject.py](inject.py), [probe.rs](probe.rs), [run_guarded.py](run_guarded.py), [validate.py](validate.py), [check_suite.py](check_suite.py), [analyze.py](analyze.py). Il supervisore conserva rc reale, comando, cwd, sentinelle Data/swap/CPU e watchdog; output fuori repo. Lock acquisito atomicamente con token `s178-orm-a98b`.

Letture storiche: `REGOLE.md`, `migration/RULEBOOK.md`, `NEXT_SESSION_WORDPRESS.md`, sessioni 175–177, revisioni e criteri 176/177, `PIN_REGISTRY.md`; AGENTS.md letto dal checkout canonico. Serena/Vexp non esposti: letture mirate come fallback. Il raw S-177 `census-miss-out/census-op-orm.txt` e il log ORM sono presenti; altri raw citati non sono stati presunti disponibili (inventario esterno `historical-availability.txt`).

HEAD non è dichiarato verde: il blocco noto `loc_dente` e l'assenza del corpus nella CI restano invariati. Le verifiche qui riguardano la sonda e questo workload, non la promozione del runtime.

## Consegna e revisione

Raw, hash e sorgente esatto persistenti: `/Users/francescotinti/Claude/phpr-s178-investigation-results/` (`MANIFEST.json`, `identity.json`, `census.tsv`, `analysis.json`, `validation.json`, `parity.json`, log e sentinelle). I percorsi `/private/tmp` descrivono il run originale; la copia persistente conserva i medesimi file. Due build sequenziali: preliminare e finale dopo correzioni della sonda; solo la seconda ha alimentato il census. Nessuna build durante le corse ORM.

[Revisione adversariale finale](revisione-s178.md): numeri ricalcolati dal TSV; nessun rilievo bloccante sul census, nessun verdetto di prestazioni. [Sessione](../sessions/WP_SESSION_178.md) e handoff aggiornati. Modifiche lasciate nel worktree senza commit/push: HEAD ha gate noto rosso e questo lavoro non ha eseguito né aggirato i gate di promozione.
