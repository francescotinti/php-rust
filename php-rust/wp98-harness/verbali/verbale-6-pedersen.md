# Verbale sedia 6 — Pedersen (WP-98)
Perimetro: confine per-richiesta/per-test, lifecycle, provenienza degli artefatti.
Oggetto: S-96.0 (apparato, riconteggio, verdetto passo 2) e §WP-97.

## VERDETTO
**RESPINTO IN PARTE.** Ho ri-sommato il raw in modo indipendente (16 righe):
`.sum` e `.out` combaciano, le sei derivate tornano alla seconda cifra. I
conteggi sono macchina-fedeli. Ma **fedeltà non è provenienza**: il
riconteggio vale come DIFFERENZA (i delta vs F2 condividono lo stesso difetto
e si cancellano), è **nullo come ASSOLUTO** — nessuna sua cifra è citabile in
futuro come autorità indipendente. Scriverlo nel raw invece di nasconderlo è
un progresso vero: fissa il GRADO, non lo eleva.

**I miei emendamenti WP-97 sono applicati a metà.** A-PP-97-1 chiedeva tre
cose: porcelain con **hash del diff** (dato invece un CONTEGGIO di righe — una
directory untracked vale 1 riga per N file: il «6» sottostima senza limite);
**re-hash del binario census a fine run** (fatto invece sul **pin di parità**,
che in questo run non è stato eseguito: controllo giusto, oggetto sbagliato);
trascrizione dall'identity (unica applicata). A-PP-97-2 (pid= + asserzione di
N): NON applicata — nessun `pid=`, il summer stampa `processes=16` e non
fallisce mai. A-PP-97-3 (suite per NOME): NON applicata — `762/1912/52
IDENTICO` è di nuovo parità di conteggi.

**Cronologia a macchina**: build 11:12:43 · identity 11:20:35 (**otto minuti
dopo la build**: `tree_dirty` descrive un albero che non è quello da cui il
binario è nato) · run →11:21:50 · amend 11:25:13 · `dc560d2` (liveness.rs)
11:32:42. Venti minuti di editing scoperti. La verifica che chiuderebbe tutto
— ricostruire da `dc560d2` e ritrovare `3e0e861c5fdbcb9b` — manca, ed è la
stessa che in WP-94 FALLÌ sul pin php-server: precedente che la rende
necessaria, non superflua.

**L'orfano**: `git branch --contains 7847cc0` è vuoto — vive nel solo reflog e
sparisce al primo `gc`. Il successore `83661e4` differisce per
`gate-measure-cifre.sh` (+13/−2), cioè **per il giudice**. La provenienza non
è solo indimostrata: è destinata a diventare **irrisolvibile**.

**Corpus senza identità**: l'oggetto misurato è binario × workload, e
`recount.identity` non porta impronta alcuna di `wpdev` (src, vendor, versione
WP, schema DB). La comparabilità F1/F2/recount poggia su un'ipotesi tacita.

**Concorrenza col selftest**: legittima in principio — la traccia di opcode di
un workload deterministico è invariante allo scheduling, i contatori non sono
tempi. NON legittima come eseguita: (a) i test media dipendono da filesystem,
DB e tempo, e l'unico controllo che escluderebbe uno scivolamento è l'identità
di suite, che è per conteggio (uno skip che scambia con un pass lascia 52
intatto); (b) i due processi condividono risorse MUTABILI non recintate — il
worktree che il run campiona come identità, il DB `wptests` che DROPPA, la
uploads che azzera. Non è rischio di tempi: è isolamento per-test.

## Emendamenti
- **A-PP-98-1** — identity completa: porcelain integrale + **sha del diff**,
  campionata dallo script di BUILD e ri-campionata a fine run; re-hash del
  binario **che ha girato**, non del pin.
- **A-PP-98-2** — nessun `.out` è autorità se non riproducibile: o la
  ricostruzione dal commit ridà la sha, o il file porta in testa
  `provenienza=NON-RIPRODUCIBILE, grado=interno`. Retroattivo a recount/f1/f2.
- **A-PP-98-3** — vietato registrare un HEAD non raggiungibile: `head=` si
  scrive dopo l'ultimo amend, o il commit è ancorato (`refs/measure/<run>`)
  prima del run. Amend che orfanizza un commit citato in un raw = FAIL.
- **A-PP-98-4** — identità del CORPUS nel raw (hash albero wpdev, versione WP,
  schema DB): senza, due run non sono confrontabili nemmeno come differenza.
- **A-PP-98-5** — lock esclusivo su worktree, `wptests`, uploads per l'intera
  finestra di misura. Il selftest del giudice non è carico neutro.
- **A-PP-98-6** — A-PP-97-2 e A-PP-97-3 **RI-EMESSE INVARIATE**: un
  emendamento non applicato non decade, si ripresenta finché non morde.
- **A-PP-98-7** — controllo dei riferimenti come DENTE, non come lezione: ogni
  `file §sezione` a manifest deve risolversi a HEAD, altrimenti FAIL; una
  regola di spareggio dichiara gli artefatti che arbitra con le loro sha ed è
  **DORMIENTE** se una cambia.
- **A-PP-98-8** — `sites_*` sommati su 16 processi non sono una popolazione di
  siti (sono siti × processi): rinominare `_per_proc_sum`.

## Kill-switch
- **KS-PP-98-1** — ricostruzione che non riproduce la sha ⇒ decadono tutte le
  conclusioni ASSOLUTE del run; restano le sole differenze intra-run.
- **KS-PP-98-2** — altro processo dentro il recinto ⇒ run INVALIDO, si ripete.
- **KS-PP-98-3** — «suite IDENTICA» per soli conteggi ⇒ la dichiarazione si
  cancella dal raw.
- **KS-PP-98-4** — leva chiusa su una regola DORMIENTE si RIAPRE d'ufficio.

## Refutazioni capitali
**SÌ, due.**
1. **La provenienza è indimostrata e ora irrisolvibile**: `head` campionato
   otto minuti dopo la build, `tree_dirty` che è un conteggio, commit
   orfanizzato da un amend sul giudice, nessuna ricostruzione.
2. **Il verdetto del passo 2 è NULLO, non contrario.** Uno spareggio con
   entrambe le premesse refutate non si inverte: **decade**. Dedurne «la
   strada lunga non vince» e chiudere `TakeSlot` usa come autorità la regola
   appena dichiarata falsa — e il §WP-97 lo ammette («se così fosse, il
   verdetto del passo 2 cambierebbe»). F3 va riclassificata **SOSPESA**, non
   archiviata: chiudere un fronte sul singolo passo di una regola morta viola
   la regola utente di non-chiusura.
