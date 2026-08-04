# WP_SESSION_96 — S-96.0: l'ottimizzazione che si è chiusa da sola, e il guardiano che ora si costruisce l'aria

**In una frase**: abbiamo chiuso la falla per cui chi lanciava il nostro
controllo qualità poteva truccarne l'ambiente e farlo mentire, abbiamo corretto
un errore di ragionamento nello strumento che decide quali copie di dati si
possono risparmiare — scoprendo che su questo programma l'errore non capitava
mai — e poi, facendo i conti onestamente, abbiamo concluso che la strada
scelta per rendere il motore più veloce costerebbe quanto rende, e quindi non
va percorsa così.

**Data**: 2026-08-04 (sera; stessa giornata di S-95.0, che aveva eseguito F1+F2).

## Oggetto ed esito

L'ordine era quello del Concilio WP-97, in quattro passi sequenziali. Tre
eseguiti, il quarto e il quinto chiusi da un verdetto, non saltati.

### Passo 0 — apparato A-SK-93..97 (in timebox, era precondizione)

Il Concilio WP-96 aveva riprodotto A MACCHINA tre canali che attraversavano
intatta la cura di S-94.0. La cura era una **lista di negazione** (`env -u …`),
e una lista di negazione è vacua per costruzione: può nominare solo ciò a cui
qualcuno ha già pensato. Erano passati `GIT_CONFIG_*` (un'intera directory
fuori dal perimetro con un `PASS --all` FIRMATO sopra una cifra fabbricata), il
`filter.<f>.clean` (`hash-object` restituiva lo sha PRISTINO per byte PATCHATI,
così il self-tether firmava codice che non girava) e `PERL5OPT`/`PERL5LIB` (un
`BEGIN` ostile dentro il processo che COSTRUISCE il corpus).

La cura è che **l'ambiente si costruisce, non si sottrae**: il re-exec passa da
`env -i` e consegna esattamente una lista CHIUSA; l'indurimento di git viaggia
IN quella lista, così lo eredita anche il `git` che perl raggiunge via `qx()` →
`/bin/sh`, dove una funzione di shell non lo avrebbe seguito. `hash-object`
ovunque con `--no-filters` (si chiude la delega, non solo l'iniezione). Il lato
untracked del perimetro si elenca SENZA `--exclude-standard`: una regola di
ignore è un'autorità del giudice e obbedisce alla legge A-SK-67 come tutte le
altre — conta solo se è committata a HEAD e identica nell'albero.

`perl -T` è stato VALUTATO E RESPINTO (`@ARGV` taintato: ogni `qx(git …)`
morirebbe, e untaintare a mano re-introduce la fiducia che il taint doveva
togliere); la lista chiusa dà la stessa garanzia in forma CONTROLLABILE.

Denti permanenti **T27-T30, ciascuno col proprio morso sul giudice pre-cura**.
T30 è il solo che non invecchia: un nome fuori dalla lista chiusa sopravvissuto
al re-exec = REFUSE, e il braccio positivo prova che la lista COSTRUISCE invece
di rifiutare ogni chiamante. **SELFTEST PASS rc=0** su tutti i denti, T0→T30.

### Passo 1 — fix di soundness e riconteggio

Applicati: A-TH-97-1 (la def non è più sottratta sul contributo dell'arco
eccezionale), A-TH-97-2/A-SK-97-2 (match ESAUSTIVI, 119 e 150 varianti per
nome: una variante nuova di `Op` che tocca uno slot ora NON COMPILA finché
qualcuno non la classifica), A-SK-97-1 (`NewAnonDeferred` + `DeclareDeferred`
alla rinuncia intera), A-DS-97-5/A-MS-97-5 (`debug_zval_refcount`,
`debug_zval_dump`), A-MS-97-1 (contatore `would_take_safe_ref`).

Riconteggio sul media group in `wp96-harness/zvalcensus-recount.out` (VERDICT
sui contatori). **P2 resta soddisfatta ⇒ KS-TH-97-3 non scatta.**

Il risultato che conta è però un altro: **il fix non cambia NESSUN conteggio F1
su questo corpus** (i delta sono esattamente zero). Il difetto è reale — la
fixture `wp96-harness/fixtures/t4-first-op-def.php` lo fa mordere a macchina
fra binario pre-fix e post-fix, con l'output del programma identico — ma la
FORMA che lo espone non ricorre nel media group.

### Passo 2 — il confronto col piano B (`wp96-harness/design96-confronto-piano-b.md`)

Il perimetro fedele non è F2 intero (refutato da Stogov) ma il nucleo stringhe,
che sta in banda MEDIA ⇒ la regola a tre bande OBBLIGA il confronto.

**Il piano B del confronto non esiste nella forma assunta.** Il riferimento a
«`design95-leva-zval.md` §Correzione» è PENDENTE: quella sezione non c'è, e
«superistruzione» non compare in nessun altro documento del repo. Il piano B
reale è A-ZV1, che non è una superistruzione ma un fast path DENTRO il braccio
`Binary` che già esiste. Quindi entrambe le premesse dello spareggio di §P1
sono refutate, in direzioni opposte, e **lo spareggio punta dalla parte opposta
a quella in cui è scritto**.

**VERDETTO: la strada lunga non vince sul perimetro fedele.** Il passo 3
(`TakeSlot`) non si apre, e il passo 4 (F4) non è applicabile: era il giudice di
una leva che non esiste, e cronometrare oggi misurerebbe il binario di ieri.

### Trovata di lato — divergenza §3.10

Costruendo le fixture serviva un builtin che LANCIASSE prima di scrivere il suo
out-param, e non lanciava. Verificato sul binario di PARITÀ: dove PHP 8 solleva
`TypeError` per un argomento non-stringa, phpr coercizza con un Warning e
prosegue (`preg_match`, `preg_split`, `explode`, `substr`, `strtoupper`;
`strlen` è già corretto). Cambia il FLUSSO, non solo il testo. Catalogata in
`PHPR_DIVERGENCES_FROM_PHP.md` §3.10, perimetro dichiarato NON misurato.

## ⭐ Lezioni

- ⭐⭐ **Una cura enumerabile contro un attacco non enumerabile è vacua per
  costruzione.** Tre canali diversi, un difetto solo: il giudice si fidava di
  programmi il cui comportamento è definito dall'ambiente del chiamante. Non si
  chiude togliendo i nomi che si conoscono, si chiude decidendo quali nomi
  esistono.
- ⭐⭐ **Un controesempio vero può essere non riproducibile nella sua forma
  letterale.** Quello di Hoare è corretto come ragionamento, ma l'analisi dà un
  arco eccezionale a OGNI op della regione: se un op senza def precede quello
  con la def, ri-inietta il live-set dell'handler e maschera tutto. È servita
  una forma costruita apposta (l'op che definisce come PRIMO della regione:
  `unset`, `=&`, `global`). Chi si ferma alla prima fixture che non morde
  conclude che il difetto non c'è.
- ⭐⭐ **«Corretto per fortuna del corpus» non è «corretto».** Le bande di
  S-95.0 sopravvivono al fix perché WordPress non scrive quella forma, non
  perché l'analisi fosse giusta. La distinzione va tenuta scritta: un corpus
  diverso non deve la stessa fortuna a nessuno.
- ⭐⭐ **Una forgia che fallisce in silenzio si traveste da cura.** Il morso di
  T28 non riproduceva perché il path del repo contiene uno spazio e il comando
  del filtro non era quotato: git ignorava il filtro rotto e usava i byte
  originali. Sarebbe passato per «canale chiuso». L'ha preso solo perché il
  dente pretendeva lo sha pristino ESATTO, non «diverso da».
- ⭐⭐ **Una regola di spareggio invecchia peggio dei numeri che arbitra.** Il
  §P1 diceva «la strada lunga non aggiunge opcode, il piano B sì»: al momento di
  usarla, entrambe le metà erano false e il riferimento al piano B era pendente.
  Le regole di decisione vanno ri-verificate contro gli artefatti quando si
  applicano, non quando si scrivono.
- ⭐⭐ **Il perimetro di un giudice non deve fidarsi della forma che un
  programma sceglie di STAMPARE.** git quota i path non-ASCII, e la virgoletta
  iniziale è bastata a far uscire un documento dal perimetro. La cura non è una
  regex migliore: è chiedere i BYTE (`-z`). La stessa classe si è ripresentata
  due volte nello stesso pomeriggio — in `check-ignore` e in `ls-tree` — e
  ciascuna è stata trovata solo seguendo la precedente fino in fondo.
- ⭐ **Un dente nuovo va collaudato in isolamento prima di pagare il selftest
  intero.** Tre run di selftest da ~70 minuti persi (uno ucciso dalla chiusura
  della chiamata, uno su una forgia rotta, uno su testo che avevo modificato
  sotto i piedi al run in corso). I quattro denti verificati singolarmente in
  pochi secondi ciascuno hanno reso il quarto run l'unico necessario.
- ⭐ **Modificare il giudice mentre il suo selftest gira invalida il run**: le
  autorità si leggono da HEAD e il self-tether hasha il testo su disco.

## Coda della sessione — il Concilio WP-98 ha morso il lavoro appena spedito

Il concilio (`wp98-harness/`) ha prodotto OTTO refutazioni capitali. Tre sono
state APPLICATE in sessione, e due di esse riguardano l'apparato spedito poche
ore prima:

- **Klabnik ha fatto ATTERRARE una forgia**: un doc-cifra con un accento nel
  NOME è uscito dal perimetro mentre il gemello ASCII veniva nominato — git
  QUOTA i path non-ASCII e la virgoletta iniziale rompe l'ancoraggio
  `^php-rust/`. **Chiuso**: `-z` + `core.quotePath=false` su `ls-files`; la
  STESSA correzione su `check-ignore -v` (che quotava a sua volta, e faceva
  sembrare non-ignorati i sidecar AppleDouble accentati — trovato seguendo il
  primo fix); e su `ls-tree`, perché il lato COMMITTATO del perimetro aveva lo
  stesso punto cieco (team-catena). Dente permanente **T31** col suo morso.
  La CLASSE resta aperta: ~20 siti `git status --porcelain`, che quotano E
  collassano le directory untracked.
- **Hoare**: il raw contiene un delta che il changeset non può produrre
  (`slot_reads_rc` si conta al sito di lettura). Ne segue che le attribuzioni
  −21/−18/−6 sono state scritte **senza controllare il pavimento di rumore**.
  Accolta e annotata nel raw. Resta in piedi ciò che non ne dipende:
  `would_take`, `would_take_rc` e `sites_movable` sono identici AL BYTE fra F1,
  F2 e riconteggio.
- **Bak**: il tetto sui corpi caldi è stato usato come una **tariffa**, e non lo
  è — in WP-44 passare da 2 a 9 corpi costò MENO che passare da 2 a 4. La
  chiusura del passo 2 è **declassata a SOSPENSIONE**.

E il dente T27 ha morso la propria costruzione: aggiungendo `-z` alla riga di
elenco, la sostituzione che ricostruiva il giudice pre-cura ha smesso di
trovare il testo che cercava, e il morso è diventato muto pur continuando a
stampare il proprio nome. Ora la riga porta un'ancora e si sostituisce INTERA.

## NON fatti (dichiarati)

- **`TakeSlot` non è stato scritto** — per verdetto del passo 2, non per tempo.
- **F4 non eseguita** — era il giudice di una leva non costruita.
- **La taglia `nm -S` predetta non è stata calcolata**: si predice la taglia di
  un braccio che si ha intenzione di scrivere.
- **Il perimetro della divergenza §3.10 non è misurato** (sonda a mano).
- Probe slope v2 e attribuzione dello slope: invariati da WP-94.

## Stato binari e processi

- phpr parità: **d5ce86e3342f3926 INVARIATO** (tutto dietro la feature
  `zval-census`; nessun ri-stash necessario). php-server: f8f4295a1dcdb627,
  non toccato.
- Build di strumentazione: post-fix `3e0e861c5fdbcb9b` in
  `phpr-census-target/`, pre-fix `e318fbfc248a8e35` in `phpr-pre-target/` (la
  ricetta per ricostruirlo sta nella testata di
  `wp96-harness/check-liveness-fixtures.sh`; il worktree usato è stato rimosso
  perché puntava a un commit reso orfano da un amend).
- Nessun processo orfano. Uploads ripristinati dalla guardia.
- `~/Claude/php-rust-output/debug/` si RIGENERA (rust-analyzer): rimossa due
  volte in sessione, il volume locale resta al limite dei 15G.
