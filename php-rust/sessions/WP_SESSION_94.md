# WP_SESSION_94.md — S-94.0 "IL FALSIFICATORE CHE HA MORSO TRE VOLTE CHI LO SCRIVEVA"

**In una frase**: abbiamo rimesso in funzione il metro che mancava da otto
sessioni — misurando di nuovo, nella stessa sera, quanto il nostro PHP
costa rispetto a quello vero su tutta la suite di WordPress, e scoprendo
che sul lavoro completo è molto migliorato mentre su un gruppo di test più
piccolo è peggiorato — e abbiamo riparato il controllo di qualità che
poteva essere aggirato, con una prova che ha bocciato tre volte le nostre
stesse riparazioni prima di accettarle.

**Data**: 2026-08-04
**Scope**: ordine del Concilio WP-95, FONDAMENTALI-first. Mezza sessione
d'apparato (A1-A4, ammessa solo perché A-SK-82 era AGGIRATA) e poi
l'OGGETTO: coppia full stessa-sera, battery61 nativa.
**Modello verificato all'apertura**: Fable 5.
**Commit**: 3448c82 → 5900ac9 → A1-fix → F-K7-fix → 93721a5 → … → 3082115,
tutti su main, pushati.
**Binari**: phpr **d5ce86e3342f3926 INVARIATO** (mai ricompilato: la coppia
full e battery61 girano sul pin baseline) · php-server **f8f4295a1dcdb627**
(vedi §Il pin che non torna; stash additivo `php-server-wp94`).

## Ordine eseguito

| # | Esito |
|---|---|
| **P0** | `--all` PASS a HEAD fc12992 col budget in vigore, prima di toccare qualsiasi cosa |
| **A1** | **I tre canali di Klabnik RIPRODOTTI A MACCHINA** (`wp94-harness/A1_FORGE_REPRO.out`): ognuno produceva un `PASS --all` rc=0 firmato col judge_sha PRISTINO — F-K3/K4 (symlink) riprodotto end-to-end per la PRIMA volta. Cura di classe: re-exec sanificante su self FISICO (`cd -P`/`pwd -P`) via `bash -p`, marker anti-loop = **lo STATO sanificato stesso** (mai una env che il chiamante può pre-impostare), contatore di profondità fail-CLOSED. Falsificatore A-SK-91 = denti T24/T25/T26, ognuno col MORSO sul giudice pre-WP-95. **SELFTEST PASS rc=0** |
| **A2+A3+A4** | `writer=script:<h16>` autenticato contro lo sha del battery a HEAD (forma → origine); `.done` letto per-RIGA (i 4 campi dalla riga che porta rev, mai un first-wins); `var_os` e `thread::current()` fuori dal `GlobalAlloc` (deadlock latente + panic-path che è UB da contratto nel canale che genera le MISURE). Predicati puri + dente `--selftest-stamp` |
| **OGGETTO 1** | **COPPIA FULL STESSA-SERA** — la prima misura full/media dopo otto sessioni. Vedi §Le cifre |
| **OGGETTO 2** | **battery61 RIPRODUCIBILE sul modo nativo** — criterio 5 del fronte SODDISFATTO: 5 BYTE-ID + dashboard NORM-ID, rc=0 |
| **OGGETTO 3-4** | **NON FATTI, dichiarati**: probe slope v2 fuso e attribuzione dello slope per NOME slittano a S-95.0. L'apparato ha sforato il timebox (vedi §Il costo onesto) |

## Le cifre (raw: `wp94-harness/pair94.out`, rapporti macchina in `pair94-ratios.out`)

| metrica | rapporto phpr/oracle | vs riferimento WP-85 |
|---|---|---|
| media group, user CPU | 2,639× | 2,58× → poco peggio |
| media group, peak footprint | 3,381× | ~3,0-3,1 → **REGRESSO** |
| full suite, master CPU | 1,873× | 2,06-2,11× → **MEGLIO, il più basso mai registrato** |
| full suite, peak footprint | 2,673× = 1901,11 MiB | ~1,98-2,03 GB → **MEGLIO** |

Conteggi **identici** sui due lati (media 762/1912/52; full 30472 test e
4558029 assertions). Unica divergenza phpr per NOME:
`Tests_Functions::test_wp_is_stream` — `stream_get_wrappers()` non elenca
`ftp`; già a catalogo, qui **confermata**, non scoperta. L'altro failure
(`test_search_hierarchical_pages_first_page`) fallisce **anche
sull'oracle**: non è nostro.

**Il regresso del media footprint è nominato e NON attribuito**: attribuirlo
richiede un canale di misura, e questa sessione non l'ha eseguito.

## 🔵 Scoperte

1. **`bash -p` sanifica la SHELL, non i FIGLI.** Impedisce alla shell di
   importare una funzione esportata, ma **non rimuove la variabile
   d'ambiente che la porta** (`BASH_FUNC_git%%`), che viene ereditata da
   ogni sottoprocesso — e questo giudice raggiunge `git`/`perl` anche via
   `qx()` di perl, che esegue `/bin/sh`, non privileged. La funzione ostile
   sopravviveva al re-exec in una run in cui la shell era pulita.
2. **Il pin `php-server` d45b578 NON è riproducibile a HEAD**: il rebuild
   pulito dà `f8f4295a1dcdb627`, e la patch A4 **non ne è la causa**
   (verificato per differenza: stesso sha con e senza, la regione è
   cfg-gated). Il sospetto di Pedersen era fondato; la causa no.
3. **`php -S` è mono-processo e WordPress fa richieste HTTP a sé stesso**:
   la prima battery61 è morta in «Maximum execution time exceeded».
   `PHP_CLI_SERVER_WORKERS=4` è la sola ragione per cui la batteria può
   esistere sul modo nativo.
4. **I «fallimenti silenziosi» del selftest erano istanze CONCORRENTI**: i
   denti piantano gli stessi file forge nel repo e due istanze si
   cancellano a vicenda. Errore di conduzione mio, non del gate.

## ⭐ Lezioni

1. ⭐⭐ **Un privilegio che vale per il processo non vale per la sua
   discendenza.** Se il giudice delega a sottoprocessi, l'unità da
   sanificare è l'**ambiente consegnato**, non la shell. (T26)
2. ⭐⭐ **Un range che si apre su un pattern presente ANCHE nella riga che
   lo usa cancella in silenzio**: lo strip della regione guard troncava lo
   script, e il dente T23 non falliva — diventava **vacuo**. L'ha rivelato
   solo l'rc ESATTO preteso da A-SK-79. *Un dente che smette di mordere non
   lo annuncia.*
3. ⭐⭐ **Un predicato non deve dipendere da ciò che esso stesso
   introduce**: la prima stesura calcolava la lista con una funzione shell
   mentre il predicato pretende `declare -F` vuoto — si falsificava da sola
   e fail-chiudeva ogni run.
4. ⭐⭐ **Un confronto identico non è un confronto valido se entrambi i lati
   stanno fallendo**: la prima battery61 dava rc=0 con i due probe
   autenticati identici… su un percorso di login FALLITO. Il probe deve
   provare che l'operazione RIESCE.
5. ⭐ **Il rc del runner non è il giudice di una coppia**: entrambe le full
   escono rc=1 perché la suite WordPress fallisce anche sull'oracle. I
   giudici sono i conteggi identici e i nomi dei failure.

## Il pin che non torna (voce APERTA per S-95.0)

`php-server` dichiarato d45b57843eeb1375; rebuild pulito a HEAD →
`f8f4295a1dcdb627`. **A4 è esclusa per differenza misurata.** Restano da
falsificare: (a) il pin fu prodotto da un albero diverso da quello che la
rotazione dichiara; (b) la build non è riproducibile byte-a-byte fra
invocazioni. Finché non è deciso, **ogni claim che poggia su quel pin resta
ADVISORY** — ed è esattamente ciò che Pedersen chiedeva, ottenuto però da
una misura e non dalla ricevuta che si sperava.

## Il costo onesto (per il Concilio WP-96)

L'apparato ha **sforato la mezza sessione**. Il tempo è andato in tre morsi
del falsificatore, ognuno dei quali ha corretto un difetto reale del lavoro
di questa sessione: senza di essi avrei chiuso A1 dichiarando sanato un
canale ancora aperto. Il prezzo pagato è nominato: **il probe slope v2 e
l'attribuzione dello slope non sono stati eseguiti**, e il criterio 1 del
fronte resta PARZIALE come lo era.
