# COUNCIL_WP96_REVIEWS — verbali integrali del Concilio a 9 sedie su S-94.0

Fonte VINCOLANTE per il design di S-95.0. Sintesi di convergenza in `SYNTHESIS_WP96.md`.

## ⚖️ SINTESI DI CONVERGENZA — Concilio WP-96 su S-94.0 (dalle ricevute + estrazioni mirate; i verbali sono la fonte VINCOLANTE)

**Verdetto complessivo: nessuna sedia ha benedetto. Tre refutazioni
capitali riprodotte a macchina, una convergenza indipendente a tre sulla
lettura delle cifre, e un aggiramento NUOVO del gate.**

### §FONDAMENTALI (in testa, per direttiva utente 2026-08-03)

**(a) Avanzamento dell'OGGETTO.** La sessione ha rimesso in funzione il
METRO (coppia full stessa-sera dopo otto sessioni) e ha reso RIPRODUCIBILE
la batteria di accettazione WordPress sul modo nativo. Ma sul prodotto:
**Gregg, col mandato inverso, incrociando `pair94.out` con i raw storici,
trova la gamba phpr PIATTA su ogni asse** (media peak −1,4%, full CPU
+0,8%, full peak −2,4%, tutti dentro lo spread). Il «record» e il
«regresso» della prima lettura erano **la gamba ORACLE**. Verdetto
d'oggetto: **nove sessioni di roadmap footprint senza movimento misurabile
su phpr**. Il guadagno vero è strumentale: un metro che funziona e una
batteria che si può rilanciare.

**(b) Contatore sessioni-senza-misura**: azzerato come conteggio (la coppia
c'è), **ma non come conoscenza**: la coppia non ha mosso nulla di phpr e
non ha attribuito nulla. Il probe slope v2 e l'attribuzione dello slope
restano non fatti (criterio 1 PARZIALE, invariato da WP-93).

**(c) Rischio d'oggetto più trascurato ORA**: **giudicare la leva di S-95.0
da una FRAZIONE**. Con un denominatore che si muove (l'oracle), qualunque
guadagno di phpr può essere mascherato o simulato. KS-BG-96-3 lo rende
bloccante: nessuna leva prima che il trend pubblichi le assolute per gamba.

### Refutazioni capitali (tutte riprodotte a macchina)

1. **🔴 Le letture comparative erano artefatti del denominatore** (Bak,
   Hoare, Gregg — convergenza INDIPENDENTE). La ricetta storica di
   GAP_TREND divide per un oracle **congelato a 5:39 = 339 s**: stesso
   numeratore, 838,59/339 = **2,474**, non 1,873. Sul media, il rapporto
   peggiora perché l'oracle è **sceso**. → SANATORIA APPLICATA in chiusura
   S-94.0 su MEASURE94, REPORT_GAP_94, GAP_TREND, WP_SESSION_94,
   NEXT_SESSION. A-TH-76, A-BB-67..72, A-BG-76..80.
2. **🔴 Il gate cifre è di NUOVO AGGIRATO, da un canale che i tre denti non
   coprono** (Klabnik, riprodotto): le env di **git** —
   `GIT_CONFIG_COUNT`/`GIT_CONFIG_KEY_0=core.excludesFile` produce `PASS
   --all` rc=0 firmato col judge_sha pristino con un doc di cifre inventate
   nel perimetro; un clean filter iniettato per env sconfigge anche
   A-SK-78. **Classe**: la cura di S-94.0 SOTTRAE variabili note, ma
   l'insieme non è enumerabile — si sanifica COSTRUENDO (`env -i` +
   allowlist chiusa). A-SK-93..97, denti T27-T30.
3. **🔴 A-AH-71 è ancora FORMA, non origine** (Hejlsberg, misurato): `git
   show` di un path assente stampa nulla e sha256 del vuoto è
   `e3b0c44298fc1c14…`, quindi la guardia «non committato» era **codice
   morto**; e un PASS con `writer=operator` salta l'autenticazione. →
   A-AH-76/77 **APPLICATI in chiusura S-94.0**; A-AH-78/79 (ancoraggio al
   commit, BREV fail-closed) a backlog.

### Altre refutazioni sostanziali

- **Matsakis**: A4 non ha tolto i percorsi panicanti — `eprintln!` panica se
  stderr è in errore, due righe sotto il `thread::current()` rimosso; manca
  un drop-guard sul flag di rientranza (A-MS-65/66).
- **Pedersen**: il pin phpr è uno sha di **contenuto senza provenienza** —
  la malattia di d45b578 non è esclusa nemmeno per phpr; battery61 **non
  resetta lo stato** fra le due gambe (A-PP-79..83).
- **Stogov**: la divergenza dei wrapper non è «correct-or-absent onesto» ma
  **incoerenza fra tabelle**: `is_builtin_scheme` rivendica già tutti e 12 i
  nomi mentre `stream_get_wrappers` ne dichiara 5 (A-DS-96-1/2/3).
- **Leijen**: CONTRARIO al grado VERDICT sul footprint — il picco a R=1 è
  uno SCREEN, e la giustificazione di α poggia su «mimalloc non
  decommitta», **falso sotto PURGE_DELAY=0** nell'albero costruito (che è
  mimalloc v3.0.2, non v2): la predizione della leva va ri-derivata
  (A-DL-67..73).
- **Hoare/Stogov**: battery61 passa anche **con login fallito su entrambi i
  lati** — serve un predicato POSITIVO, o il criterio 5 torna PARZIALE
  (KS-DS-96-3). *(Il difetto era stato osservato e corretto in sessione, ma
  il gate non lo impedisce strutturalmente.)*

### Ordine vincolante per S-95.0

**0. Denominatore omogeneo** (KS-BG-96-3, bloccante): GAP_TREND pubblica le
quattro **assolute per gamba** e il Δ sulla gamba phpr. Nessuna leva prima.
**1. Apparato minimo che blocca l'oggetto**: `env -i` + allowlist chiusa
(A-SK-93..97) — senza, ogni cifra di S-95.0 nasce di nuovo senza autorità.
**2. LEVA arene per-file del preludio** coi 16 obblighi del team-leva.
**3. Probe slope v2 fuso** e **attribuzione dello slope** (criterio 1).
**4. Il pin che non torna** (php-server e, per estensione, phpr).

**BACKLOG PER NOME**: A-MS-65..70 · A-AH-78..84 · A-PP-79..83 ·
A-DS-96-1..9 · A-TH-77..82 · A-BB-68..72 · A-BG-76..80.

---
# VERBALI INTEGRALI (fase 1, bozze indipendenti)

---
## verbale-1-hoare.md

# Verbale 1 — Tony Hoare (Concilio WP-96, giudizio su S-94.0)

**VERDETTO: CON EMENDAMENTI**, con **opposizione nominata a due voci**: la
riga WP-94 di `GAP_TREND.md` e il «criterio 5 SODDISFATTO».

## Refutazioni (sostanziali)

**R1 — I tre giudizi della tabella sono artefatti del DENOMINATORE.** Il pin
phpr è INVARIATO, quindi nessuna delle tre letture può parlare di phpr.
Le cifre committate lo dicono da sole:
- *media footprint «REGRESSO»*: il numeratore phpr 1170785648 B (1170,8 MB)
  è **piatto** sullo storico (WP-63 1170,6 · WP-65 1150,6 · WP-64 1186,9);
  è l'**oracle** a essere sceso a 346,3 MB da 382,2-393,7 (−9,6…−12%).
  Col denominatore storico il rapporto è 2,97-3,06 = **dentro la banda**.
  GAP_TREND contiene già due precedenti identici (⚠️ WP-30; G3 su WP-62).
- *full CPU «il più basso mai registrato»*: la banda 2,06-2,11× divide per
  un oracle **CITATO** (5:39 = 339 s); S-94.0 divide per un oracle
  **misurato** (447,84 s, +32%). Rapporti con denominatori diversi non si
  ordinano. Col metodo storico stanotte darebbe **838,59/339 = 2,47×**, cioè
  un REGRESSO. Il numeratore è pure cambiato di definizione (master-CPU
  user dal tail `.rss` → tree user+sys dell'albero).
- *full peak «MEGLIO»*: 1993459800 B = **1,993 GB decimali**, cioè **dentro**
  la banda 1,98-2,03 GB; sembra migliore solo perché convertito in MiB
  (1901,11) mentre la banda non dichiara la sua unità.

**R2 — La coppia di S-94.0 NON è il giudice della leva.** NEXT_SESSION §WP-95
la eleva a «prima» della leva per-file. La regola vincolante del progetto è
l'opposta (WP-65: *la coppia build-ADIACENTE è l'unico giudice del costo*;
deriva inter-giornata osservata fino a 2,6%, WP-55). Una baseline di
un'altra sera è un riferimento di trend, mai il «prima» di una leva.

**R3 — battery61 non falsifica.** La lezione 4 della sessione («il probe deve
provare che l'operazione RIESCE») è rimasta in prosa: nel giudice non c'è
**nessun predicato positivo**. Login fallito su entrambi i lati ⇒ due pagine
di login identiche ⇒ `rc=0`. Inoltre `norm()` è un normalizzatore
**generico** (`[0-9a-f]{10}\b`) malgrado il commento: cancella qualunque
token da 10 hex e, per via del `\b`, **mutila la coda degli md5** (due md5
che differiscono solo negli ultimi 10 caratteri diventano identici). Infine
le due gambe girano **senza reset DB fra loro** (a differenza di pair94):
il protocollo è asimmetrico per costruzione.

**R4 — Un hash di binario non è un'identità.** Il «pin che non torna» è posto
come dilemma a due; manca la terza ipotesi (stesso albero, toolchain /
lockfile / feature / RUSTFLAGS / target-dir diversi), e il rebuild ripetuto
proposto non la separa.

## Emendamenti

- **A-TH-76** «Nessun rapporto senza denominatore omogeneo»: la riga WP-94 di
  GAP_TREND si riscrive senza MEGLIO/REGRESSO/«mai registrato»; le tre
  letture si declassano a NON-COMPARABILI e si nominano numeratore e
  denominatore di ogni banda storica prima di qualsiasi ordinamento.
- **A-TH-77** «Un identificatore di metrica denota UNA definizione»:
  `full_master_cpu` è tree user+sys, non master-user; media è user-only.
  Rinominare entrambe e dichiarare l'unità (GB vs GiB) in ogni banda.
- **A-TH-78** «Il "prima" della leva è la sua coppia adiacente»: S-95.0
  misura old **e** new la stessa sera, stesso protocollo; la coppia S-94.0
  resta baseline di trend.
- **A-TH-79** «Un gate d'accettazione porta un predicato positivo»:
  battery61 pretende `Location: …/wp-admin/`, cookie `wordpress_logged_in_`
  e un marcatore admin-only nel body, su **entrambi** i lati.
- **A-TH-80** «Normalizzare per NOME»: nonce ancorato all'attributo, conteggio
  delle sostituzioni uguale sui due lati; niente classi hex libere.
- **A-TH-81** «PATH è ambiente scelto dal chiamante»: la cura A1 chiude
  BASH_SOURCE/symlink/BASH_ENV ma lascia `PATH=…:/opt/homebrew/bin:$PATH`
  (dir scrivibile dallo stesso utente) **fuori** da `GATE_SANE`. PATH fisso
  senza coda del chiamante e dentro il predicato. (IFS: congetturato canale,
  **refutato a macchina** — bash lo reimposta all'avvio.)
- **A-TH-82** «Un canale di misura muto si dichiara»: in `huge_note` il
  `try_with` in Err salta in silenzio; contatore atomico degli scarti nel
  banner. Per il probe v2 il ledger indicizza per **puntatore** (nota
  spostata dopo l'alloc interna), mai per taglia.

## Kill-switch

- **KS-TH-96-1**: se una lettura di trend confronta rapporti con denominatori
  di regime diverso ⇒ la riga è VOID.
- **KS-TH-96-2**: se la leva è giudicata contro una coppia di un'altra sera
  ⇒ verdetto VOID, ripetere in coppia adiacente.
- **KS-TH-96-3**: se battery61 in ordine invertito (phpr-first, DB resettato)
  non replica il verdetto ⇒ criterio 5 torna APERTO.
- **KS-TH-96-4**: se un forge che sostituisce un tool risolto fuori da
  `/usr/bin:/bin:/usr/sbin` ottiene `PASS --all` rc=0 ⇒ A-SK-91 riaperta.

---
## verbale-2-matsakis.md

# Verbale sedia 2 — Niko Matsakis (Concilio WP-96)

Perimetro: ownership, aliasing, borrow checking, soundness. Mandato: refutare.

## VERDETTO

**CON EMENDAMENTI — l'obiettivo dichiarato di A4 NON è raggiunto.**

Ciò che A4 ha fatto è corretto: `IN_TRACE`/`THR_ID` sono `Cell` con
`const {}` init, quindi il thread_local **non ha distruttore e non alloca
in init pigra** — `try_with` non può fallire e, soprattutto, non c'è il
percorso di allocazione ricorsiva che una TLS lazy avrebbe introdotto.
`galloc_note`/`gfree_note` sono `fetch_add` puri: nessun lock, nessuna
alloc, nessun panic. `HUGE_TRACE` fail-closed a 0. Fin qui, sound.

**Ma la funzione riscritta per non panicare contiene ancora un `eprintln!`**
(main.rs:121), documentato dalla std come *«panics if writing to
io::stderr fails»*. Stderr chiuso, EPIPE, o disco pieno ⇒ panic che
**srotola fuori da `GlobalAlloc::alloc`**: esattamente l'UB-da-contratto
che A-TH-73/74 volevano chiudere. `thread::current()` è stato tolto e il
gemello è rimasto in piedi due righe sotto. Aggravante di secondo ordine:
il panic srotolante salta `f.set(false)`, e il flag di rientranza resta
`true` per sempre su quel thread — il canale si spegne **in silenzio**
(lezione WP-94: «un dente che smette di mordere non lo annuncia»).

## Emendamenti

- **A-MS-65** — `huge_note`: output panic-free (`io::stderr().write_all`
  con `let _ =`, o `libc::write`), MAI `eprintln!`. Nessuna macro che
  possa unwindare dentro un `GlobalAlloc`.
- **A-MS-66** — il flag di rientranza si rilascia con un **drop-guard**
  (`struct Reset<'a>(&'a Cell<bool>)`), non con un `set(false)` finale:
  correttezza per costruzione, non per raggiungibilità.
- **A-MS-67** — `huge_note` su `realloc` **asimmetrico**: si nota solo `n`.
  Un blocco huge che si contrae (40 MB → 1 KB) **non emette nulla**: la sua
  morte è invisibile. Questa è la stessa classe di errore che generò il
  «mai liberata» di WP-93. Notare `l.size()` come `dealloc` e `n` come
  `alloc`, sempre (chiude anche §WP-95 punto 2).
- **A-MS-68** — `thr=` è un seriale assegnato **all'ordine della prima
  huge**, non al worker: non è congiungibile per NOME. Registrazione
  esplicita dell'id all'entrata del worker (fuori dall'allocatore) o il
  campo si chiami `serial=`, mai `thr=`.
- **A-MS-69** (leva §WP-95) — nel commit della leva **vietata ogni nuova
  `unsafe`/`transmute`/`Box::leak`/`&'static` in `lower/mod.rs`**. Con arene
  per-file il borrow checker è il gate: se il codice compila, i nodi non
  sopravvivono all'arena. L'unico modo di sbagliare è **disattivare il
  gate** per tenere in vita l'AST — e quella è la leva WP-78 che divenne
  leak.
- **A-MS-70** (leva) — `low` è costruito su `&file` (il PRIMO unit) e poi
  riceve statement di `file_ns/bc/gmp/mysqli/gd/fi`: **gli span sono
  risolti contro la sorgente sbagliata**. Oggi è latente; spezzando le
  arene diventa attivo. Un `Lowerer` per unit, o prova esplicita che gli
  span non indicizzano mai `file`.

## Kill-switch

- **KS-MS-96-1** — se `grep` trova una macro panicante (`eprintln!`,
  `println!`, `unwrap`, `expect`, `panic!`, `assert*`) nel corpo
  raggiungibile da `MemCountingMi::{alloc,dealloc,alloc_zeroed,realloc}`:
  **build della campagna INVALIDA**, cifre VOID.
- **KS-MS-96-2** — se il digest di `(class_index, fn_index, static_count)`
  del preludio cambia dopo la leva: **revert della leva**. Le arene
  cambiano l'ownership, non l'ordine di hoist.

## Refutazioni

1. **CAPITALE**: «A4 ha tolto i percorsi che panicano dal `GlobalAlloc`» —
   **FALSO**, `eprintln!` panica su stderr in errore ed è nella stessa
   funzione (A-MS-65).
2. «Σ `allocated_bytes` per-arena è il numeratore della leva» — **falso**:
   N arene ripagano ciascuna il raddoppio iniziale, Σ può **crescere**
   mentre il **peak concorrente** cala. Il numeratore è il peak simultaneo;
   il controllo positivo `25795552` va riletto in quella metrica.
3. «`try_with` protegge da TLS distrutto» — vero ma **vacuo** qui: `Cell`
   senza Drop non ha distruttore. La vera protezione è il `const {}` init
   (nessuna alloc in init pigra): se un domani `THR_ID` diventa un
   `String`, la protezione sparisce senza che una riga la nomini.
4. Nota di verifica: `init_huge_trace()` è chiamata **dopo**
   `logging::init()`; «prima di ogni spawn» va provato, non asserito.

---
## verbale-3-klabnik.md

# Verbale sedia 3 — Steve Klabnik (Concilio WP-96)

## VERDETTO: **A-SK-88/89/90 AGGIRATE — due forge riprodotte a macchina, una terza primitiva verificata.**

La cura di S-94.0 ha sanificato l'ambiente per **lista di negazione**
(`env -u BASH_ENV -u ENV -u SHELLOPTS -u BASHOPTS -u CDPATH -u GLOBIGNORE
$UNSET_FN`). Una blacklist non può enumerare ciò che non conosce. Tutte le
autorità di questo giudice sono **sottoprocessi `git`** e il suo valutatore è
un **processo `perl`**: né `GIT_*` né `PERL5*` sono toccati dal re-exec. La
classe del difetto è di nuovo UNA: *il giudice si fida di programmi il cui
comportamento è definito dall'ambiente del chiamante.*

Controllo positivo (albero invariato, verbali WP-96 untracked):
`bash wp81-harness/gate-measure-cifre.sh --all` → **FAIL**, 7 righe
`UNCOMMITTED … perimeter class`.

## Refutazioni capitali

**F-K10 — perimetro cieco per `core.excludesFile` (nessun byte del giudice
toccato).** Doc-forge piantato: `php-rust/wp96-harness/zzk96-forge.md`
(«il picco era 123458 B e la media 987654 B»), cifre in NESSUN corpus.

```
printf 'wp96-harness/\n' > /tmp/ex
GIT_CONFIG_COUNT=1 GIT_CONFIG_KEY_0=core.excludesFile GIT_CONFIG_VALUE_0=/tmp/ex \
  bash wp81-harness/gate-measure-cifre.sh --all
```
→ **rc=0**, **zero FAIL**, `PASS gate-measure-cifre --all …
[judge_sha=27e5b75fb593d86e … head=93733821404d]`. Il giudice è il blob di
HEAD, verbatim, firmato: il perimetro A-SK-71/A-SK-80 poggia su
`ls-files --others --exclude-standard`, e l'ignore è **politica
dell'ambiente**, non autorità.

**F-K11 — self-tether A-SK-78 sconfitto da un clean filter iniettato.**
`git hash-object` applica i filtri per default; la config si inietta per
sola env:
```
perl -pe 's/\$all_rc = 1;//g if /UNCOMMITTED/' wp81-harness/gate-measure-cifre.sh \
  > wp81-harness/zzk96-judge.sh          # sha reale 08a573e4e0e8a687
GIT_CONFIG_COUNT=2 GIT_CONFIG_KEY_0=core.attributesFile GIT_CONFIG_VALUE_0=/tmp/attrs \
  GIT_CONFIG_KEY_1=filter.pristine.clean GIT_CONFIG_VALUE_1="cat …/gate-measure-cifre.sh" \
  bash wp81-harness/zzk96-judge.sh --all
```
(`/tmp/attrs`: `zzk96-judge.sh filter=pristine`) → **rc=0**, nessun REFUSE
A-SK-78, `PASS --all` con **judge_sha pristino 27e5b75fb593d86e** mentre gira
testo PATCHATO. Primitiva isolata: `hash-object` del file patchato restituisce
`27e5b75f…` invece di `a42354d9…`.

**F-K12 (primitiva verificata) — `PERL5OPT`/`PERL5LIB` dentro il valutatore.**
`PERL5LIB=… PERL5OPT=-MHack perl - a b <<'P'` esegue il `BEGIN` del modulo
ostile nel processo che COSTRUISCE il corpus, e sopravvive intatto a
`/usr/bin/env -u BASH_ENV … /bin/bash -p`. Da lì `CORE::GLOBAL::exit`, `qx`,
`%corpus` sono del chiamante.

**Canale CHIUSO (refutazione mia, riportata):** `BASH_FUNC_x%%` con corpo non
`() {` (subshell, newline) — il bash 3.2 Apple **rifiuta** l'import: la regex
di `FN_ENV_NAMES` non è aggirabile per questa via su questa macchina.

## Emendamenti

- **A-SK-93** — l'ambiente si **costruisce**, non si sottrae: re-exec via
  `env -i` con lista CHIUSA (PATH fisso, HOME, LANG, TMPDIR). Una blacklist è
  vacua per costruzione.
- **A-SK-94** — ogni `git` del giudice passa da un wrapper unico con
  `GIT_CONFIG_NOSYSTEM=1 GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null`.
- **A-SK-95** — `hash-object --no-filters` ovunque si tetheri (shell **e**
  `work_blob_sha`); meglio: sha calcolato sui byte letti, mai delegato.
- **A-SK-96** — gli untracked del perimetro si elencano **senza**
  `--exclude-standard`, sottraendo solo i `.gitignore` COMMITTATI a HEAD.
- **A-SK-97** — `perl` invocato con `-T` o con `PERL5OPT/PERL5LIB/PERLLIB/
  PERL5DB` provati assenti nel predicato di stato.

## Kill-switch

- **KS-SK-96-1 (T27)** — F-K10: rc **esatto 1**, mai `PASS --all`; morso sul
  giudice pre-cura (rc=0 + PASS firmato).
- **KS-SK-96-2 (T28)** — F-K11: REFUSE per nome rc=1; morso pre-cura rc=0 con
  judge_sha pristino.
- **KS-SK-96-3 (T29)** — F-K12: REFUSE per nome; morso pre-cura.
- **KS-SK-96-4 (T30, il solo che non invecchia)** — dopo il re-exec il giudice
  confronta il proprio `env` con la lista chiusa: **una variabile in più =
  REFUSE**. Senza T30, WP-97 mi ritrova qui con un prefisso nuovo.

*Residui rimossi (`zzk96-judge.sh`, `zzk96-forge.md`); nulla committato.*

---
## verbale-4-hejlsberg.md

# Verbale 4 — Anders Hejlsberg — Concilio WP-96

**VERDETTO: A-AH-71 è ancora una FORMA.** Non per stile: per tre misure fatte
a macchina in questa lettura. `drow_judge` (A-AH-69) è corretto ma
sottodimensionato. Il triangolo attempts↔stamp↔OUT ha un lato che si apre
con **una parola**.

## Refutazioni (capitali)

**R1 — l'autenticazione FALLISCE APERTA su una costante pubblica.**
`BSCRIPT_SHA=$(git show HEAD:… | shasum | cut -c1-16)`: se il path non è
committato, `git show` non scrive nulla, `shasum` digerisce lo stdin vuoto e
la variabile vale **`e3b0c44298fc1c14`** (verificato: `git show
HEAD:wp99-harness/battery-99pre.sh` → esattamente quel valore). Quindi
`[ -z "$BSCRIPT_SHA" ]` **non è mai vero**: il fail «script NON committato ⇒
VOID» (riga 447-449) è **codice morto**, e il tooth degrada in un confronto
contro una costante che qualunque falsario sa scrivere. Aggravante: l'idioma
giusto è **già nel file otto righe sopra** — per la matrix (A-SK46) c'è
`git cat-file -e` accanto allo sha. Applicato lì, dimenticato nel dente nuovo.

**R2 — il selftest morde il comparatore, il forge vive nella derivazione.**
`--selftest-stamp` dichiara «writer= authenticated against the HEAD battery
script» ma **non tocca git**: passa due sha a mano a `writer_foreign`. Prova
l'uguaglianza di stringhe; **non** prova la sola cosa che A-AH-71 aggiunge,
cioè che l'atteso venga dallo script a HEAD. La riga di verdetto asserisce
una proprietà che il dente non esercita.

**R3 — opt-out con una parola.** A-AH58 ammette `writer=(script:h16|operator)`;
solo gli ABORT esigono `operator`. `writer_foreign` filtra su
`writer=script:` ⇒ **un PASS con `writer=operator` salta l'intera
autenticazione**, e A-AH54 gli chiede solo `sha256=DSHA`, che il falsario
possiede (è lo sha del proprio OUT). Il triangolo si chiude dal lato del
falsario.

**R4 — comunque non sarebbe un'origine.** L'atteso è funzione pubblica di un
file committato: chiunque lo digita. È una forma con la risposta nota, non
una firma. E si confronta con **HEAD**, non con lo stato al momento della
scrittura: al primo edit del battery **tutta la famiglia** (`grep
"battery=$BNAME "`, ogni rev) diventa foreign e ogni consumazione voida
retroattivamente. La pressione che genera è **riscrivere il ledger** —
l'atto che A-AH54 esiste per impedire.

**R5 — il dente non ha mai girato sul percorso reale.** Censimento del
ledger committato: righe `battery=89pre|90pre` soltanto, **zero occorrenze
di `writer=`**. Il blocco grammar-v2 non si è mai eseguito fuori dal
selftest.

**R6 — BREV non è validato.** `git diff "$BREV..HEAD" 2>/dev/null` con rc
ingoiato: misurato, un BREV inesistente dà `DELTA=[]` ⇒ **denti (i) e (iv)
VACUI in silenzio**, e in modo equivalence non c'è nemmeno l'ancestor-check.
In più BREV entra non-quotato in ≥6 regex (`^rev=$2 `, `git=$BREV`,
`^battery_rev=$BREV `): un «rev» di metacaratteri è un jolly.

## Emendamenti

- **A-AH-76** — `git cat-file -e` prima dello sha; rifiuto esplicito se
  `BSCRIPT_SHA` == sha256("")[0:16]. Fail-closed.
- **A-AH-77** — `esito=PASS` **esige** `writer=script:`; operator non firma
  un PASS (simmetrico ad A-AH58 sugli ABORT).
- **A-AH-78** — `writer=script:<h16>@<commit>`, verificato contro
  `git show <commit>:<path>` (il commit che ha introdotto la riga), mai HEAD.
- **A-AH-79** — BREV fail-closed: `rev-parse --verify "$BREV^{commit}"`,
  `^[0-9a-f]{7,40}$`, ancestor di HEAD in **entrambi** i modi.
- **A-AH-80** — `.done` per-OUT; `drow_judge` seleziona su `battery=`+`rev=`
  con **una** riga matching (oggi la regola «una sola riga `rev=`» è
  per-directory e collide fra batterie: fail-closed che invita a cancellare).
- **A-AH-81** — dente sulla DERIVAZIONE in un repo git temporaneo (script
  assente/spostato ⇒ REFUSE); e il verdetto del selftest nomini solo ciò che
  morde.

### Leva S-95.0 (perimetro mio: dedup/incrementale)

- **A-AH-82** — predizione ex-ante sui **byte toccati** per-unità, mai sulla
  capacità d'arena: la coda 13738592 mai usata prova capacità ≠ touched; una
  leva misurata sulla capacità misura la politica di crescita del bump.
- **A-AH-83** — **ordine invertito**: A-AH-73 (HIR plain-data) **prima**. Le
  arene per-file cambiano la granularità di *lifetime*, non il contenuto:
  non deduplicano nulla fra i W worker, il canale ×W resta. Partizionare
  prima incide lifetime per-file in un'API che la leva condivisa dovrà
  disfare. *Non si condivide ciò che non si sa serializzare.*
- **A-AH-84** — il «prima» **non può essere pair94**: la leva ricompila, e la
  coppia **build-adiacente** è l'unico giudice del costo (WP-65; spread
  inter-build 38229 misurato in S-93.0). Serve il gemello stesso-albero
  cfg-off, stessa sera.
- **A-AH-85** — `Σ T_i ±10%` = ±2,6 MB, più largo di quasi ogni effetto
  per-file; e se contatore e censimento condividono l'hook GlobalAlloc
  l'accordo prova il **determinismo dell'hook**, non l'attribuzione. Secondo
  stimatore indipendente + residuo (disciplina repair90-estimators).
- **A-AH-86** — almeno una fixture oracle-morsa in cui l'**ordine** di parse
  del preludio è osservabile (redeclare / const-fold): il parse pigro è un
  cambio di semantica, e le fixture attuali provano solo il caso felice.

## Kill-switch

- **KS-AH-96-1** — `BSCRIPT_SHA` vuoto o == sha256("")[0:16] ⇒ consumazione VOID.
- **KS-AH-96-2** — una riga `esito=PASS` senza `writer=script:` ⇒ VOID.
- **KS-AH-96-3** — BREV non risolvibile a un commit ⇒ VOID (mai delta vuoto per errore).
- **KS-AH-96-4** — predizione della leva formulata sulla capacità d'arena ⇒ campagna VOID.
- **KS-AH-96-5** — leva misurata contro pair94 e non contro il gemello build-adiacente ⇒ cifra ADVISORY, mai verdict-grade.

---
## verbale-5-bak.md

# Verbale 5 — Lars Bak (alloc-rate, path caldi, disciplina statistica)
## Concilio WP-96 · oggetto: la coppia full di S-94.0

## VERDETTO — **RIGETTO le quattro letture, non la misura**

I raw di `pair94.out` sono buoni: coppia stessa-sera, conteggi identici,
guardia uploads, identità in banda. È tutto ciò che c'è di verdict-grade.
Le **quattro letture** — tre «MEGLIO» e un «REGRESSO» — sono **tutte
comparazioni con una citazione**, esattamente ciò che `pair94.out` §grade
dichiara di NON fare («le due gambe si confrontano fra loro, non con una
citazione»), e **tre di esse si ribaltano o si annullano con l'aritmetica
dei raw stessi**. Grado corretto della coppia: **VERDICT sui due rapporti
di stasera; SCREEN su ogni delta cross-sessione**.

## Refutazioni capitali

**R1 — il «REGRESSO» del media footprint è il DENOMINATORE, refutato al
byte.** phpr stasera = 1170785648 B = **1170,8 MB**. Le tre coppie che
FONDANO la banda: WP-63 1170,0 · WP-64 1186,9 · WP-65 1150,6 MB. phpr è
**dentro la banda, al centro**. È l'oracle a essersi mosso: 346,3 MB
contro 393,0 / 393,7 / 382,2 = **−11,9%**. E 3,381/2,979 = **1,135** =
esattamente 1/0,881: **l'intero «regresso» è il movimento dell'oracle**.
Con i denominatori storici: 1170,8/393,0 = **2,979**, in banda 2,9-3,2.
Peggio: GAP_TREND porta già il riquadro d'avvertimento di **riga WP-30**
(«il rapporto sale per rumore dell'oracle, non per una regressione phpr»)
e la **regola G3-Gregg** citata alla riga WP-62 per un oracle **−6,9%**:
«coppia singola, NON entra nel trend». S-94.0 ha commesso l'errore che la
sua stessa tabella documenta. Il backlog «Regresso del media footprint»
di §WP-95 **brucerebbe una sessione a inseguire un denominatore**.

**R2 — il «full CPU il più basso mai registrato» confronta due rapporti di
costruzione diversa.** GAP_TREND §Metodo 2: denominatore **congelato a
5:39 = 339 s**. Verifica: 971/339 = 2,86 ✓, 699/339 = 2,06 ✓. Stasera il
denominatore è **vivo, 447,84 s**. Applicando la ricetta della tabella al
numeratore di stasera: 838,59/339 = **2,474×**, cioè PEGGIO di 2,06-2,11.
Non affermo il segno: affermo che **1,873 è il primo punto di una serie
nuova, N=1, e non ha prior**. «Migliorato in modo netto» non è sostenuto.

**R3 — il peak full non è «SCESO»: è FLAT.** 1993459800 B = **1,993 GB**,
**dentro** la banda 1,98-2,03 GB. E quella banda ha provenienza ambigua
(le righe WP-60/62 danno peak 3,900 GB su runNN tree-user, WP-64 dà 2,035
GB): un riferimento senza workload, R e raw nominati non è un riferimento.

**R4 — VERDICT su R=1 di una MAX-statistic viola KS-BB-92-1.** Il peak è
un estremo, non una media: R=1 è un tiro da una coda, la statistica meno
mediabile che abbiamo. Fu la mia sedia a far declassare la min-statistic e
ripubblicare b_peak a mediana. WP-84 misurò 217,7/228,8/229,2 MiB (5%) sul
peak; WP-62 documenta spread serale phpr ±1,7-2,3% sulla CPU — e il media
CPU 2,639 vs 2,58 è **+2,3%**, cioè dentro quello spread: **flat**.

**R5 — il grado è una costante, non un calcolo.** `pair94.sh` fa
`echo "grade=VERDICT"`: stringa cablata, poi citata come output macchina.
Nessun R, nessuno spread, nessun drift entra in quella parola.

**Ordine oracle-prima**: due bias di segno opposto, **entrambi non
misurati** — phpr sempre secondo (page cache calda) e ultimo su 28 minuti
continui (01:20:29→01:48:45, deriva termica). L'ABBA esiste già in casa
(WP-86, purge). Non invalida stasera; invalida il *claim di precisione*.

**Concedo**: conteggi identici (762/1912/52; 30472/4558029) e i due
failure per NOME sono fedeltà genuina; rc non-giudice è corretto.

## Emendamenti

- **A-BB-67 DENOMINATORE VIVO**: la colonna full-suite di GAP_TREND è
  rifondata; il 5:39 congelato è **ritirato**, le righe storiche marcate
  «costruzione diversa». 1,873 apre serie nuova, N=1, nessun delta.
- **A-BB-68 SANDWICH-DRIFT**: ripetere la gamba più economica (media
  oracle, 36 s = 2% della campagna) **in coda**; pubblicare
  drift=|Δ|/media. Delta cross-sessione < drift ⇒ SCREEN.
- **A-BB-69 GRADO CALCOLATO**: lo script deriva il grado da R, spread e
  drift; VERDICT **vietato** a R=1 su peak. Mai un `echo` costante.
- **A-BB-70 NUMERATORE PUBBLICATO**: ogni riga di trend porta i **due
  assoluti**, il workload, R e il raw. Un rapporto senza denominatore non
  è falsificabile fra sessioni (R1 è la prova).
- **A-BB-71 G3 APPLICATO**: pair94 è coppia singola ⇒ **non aggiorna il
  trend**. Correggere MEASURE94, REPORT_GAP_94, §Le cifre e NEXT_SESSION:
  «riferimenti INVARIATI». Cancellare il backlog «Regresso media».
- **A-BB-72 ABBA sulla gamba media** (o,p,p,o): costo 108 s.

## Kill-switch

- **KS-BB-96-1**: claim cross-sessione il cui denominatore non è misurato
  nella stessa campagna **e con la stessa costruzione** del riferimento
  citato ⇒ **VOID**, rimosso dai documenti finché non ri-derivato.
- **KS-BB-96-2**: grado VERDICT su max-statistic con R=1 ⇒ **declassato a
  SCREEN d'ufficio**, dal gate cifre, senza discussione.
- **KS-BB-96-3**: se il drift di A-BB-68 supera il delta rivendicato, la
  campagna è **invalida per quella metrica** (non «indicativa»).

---
## verbale-6-pedersen.md

# Verbale 6 — Anders Pedersen (confine per-richiesta/per-run, lifecycle, ricevute in banda)
## Concilio WP-96 su S-94.0 · mandato: REFUTARE

## VERDETTO: **CONTRARIO CON EMENDAMENTI** — l'apparato è migliorato, le
**ricevute NON sostengono i claim**. Tre difetti di confine, uno capitale.

### 1. «ADVISORY» non è un confine, è un aggettivo
Il pin `php-server` non riproducibile è stato NOMINATO (bene) e poi
declassato con una parola. Non esiste un predicato che neghi
`grade=VERDICT` a un numero prodotto da quel binario: la degradazione vive
in prosa, in due file di rotazione, e muore alla prima citazione. Serve
etichetta IN BANDA nell'artefatto e un dente che morda. Inoltre la sessione
ha rinviato l'esperimento che DECIDE fra (a) e (b): due rebuild puliti allo
stesso HEAD sono due comandi. Rinviare un falsificatore da due comandi
mentre si spendono tre morsi sull'apparato è un errore di priorità.

### 2. CAPITALE — la malattia di `php-server` è UNFALSIFIED su `phpr`
`d5ce86e3342f3926` è **registrato in banda** (`pair-out/pair94.identity`,
`battery61-accettazione.out`, header di `pair94.out`: concordi) — quindi
l'identità del FILE è verificata, non solo asserita. Ma è uno sha di
CONTENUTO **senza ricevuta di provenienza**: nulla lo lega a un albero.
È esattamente la condizione in cui `d45b578` è marcito. Se un pin ha già
divorziato dal suo albero in questo repo, «INVARIATO» per `phpr` prova
solo che *lo stesso ignoto* è stato misurato due volte. Su questo ignoto
poggia l'intera baseline della leva.
Aggravanti misurate: lo sha è calcolato **una volta**, prima di quattro run
(~45 min di orologio: epoch pair94 1785799229 → battery61 1785801803), mai
riverificato dopo; **non è mai CONFRONTATO** con il pin dichiarato (se
avesse differito, lo script avrebbe prodotto numeri lo stesso: registra,
non giudica); l'oracle è identificato da `php -v`, **stringa, non hash**.

### 3. `pair94.identity` è **untracked** — l'autorità è una TRASCRIZIONE
`git ls-files` di `wp94-harness/pair-out/` restituisce **solo i quattro
`.time`**. Non sono in repo: `pair94.identity`, `full-*.txt`, `media-*.txt`,
`progress.txt`, `.done`. Quindi (i) l'unica identità che sopravvive a HEAD è
il blocco copiato **a mano** in `pair94.out`; (ii) `pair94-ratios.out` —
il file che lo script stesso elegge ad autorità («il documento CITA questo
file») — porta `grade=VERDICT` e **zero campi d'identità**: numeri senza
misurato; (iii) **i conteggi 762/1912/52 e 30472/4558029 e i due nomi di
failure non sono ricevutati**: i `.txt` da cui provengono non esistono a
HEAD e `pair94.sh` **non contiene alcun giudice di parità** — calcola
meccanicamente i rapporti e lascia la PARITÀ alla prosa. È la violazione
letterale di `gate-diff-fail-set-not-count`, commessa dal file che si
vanta di non fare aritmetica in prosa.

### 4. battery61 — il confine per-run NON esiste
`serve_and_capture` pulisce la **directory di output**, non lo **stato**.
Nessun reset DB, nessuna guardia uploads (che `pair94.sh` invece usa).
La gamba oracle esegue un **login POST + dashboard**: scrive session token
in usermeta, arma wp-cron, muove transient/option. La gamba phpr parte da
quel DB mutato, **sempre seconda**: asimmetria sistematica, non rumore. E
lo stato **esce** dal run: l'installazione viva resta modificata.
Inoltre: (a) il confronto è **body-only** — degli header si legge la sola
prima riga, `Set-Cookie`/`Location`/`Content-Type` non sono mai diffati, e
«BYTE-ID» si legge come identità di risposta; (b) probe 5 è `bytes=0`:
due corpi vuoti coincidono per costruzione — il dente è vacuo, la prova del
login la porta solo il 200 del probe 6; (c) `norm()` è `[0-9a-f]{10}\b`,
**generico di FORMA** mentre il commento sopra dichiara «per NOME, nessun
normalizzatore generico»: maschera qualunque token 10-hex in 142 KB.

## Emendamenti
- **A-PP-79** — *Identità giudicata, non registrata*: ogni harness di misura
  confronta lo sha calcolato col pin ATTESO passato in argomento e
  **fail-CLOSED** su divergenza; ricalcolo **dopo l'ultima gamba**, entrambi
  nel receipt; oracle per **sha**, non per `-v`.
- **A-PP-80** — *Nessuna autorità senza identità*: il file citato dal gate
  cifre incorpora il blocco identità; `grade=VERDICT` negato se assente.
- **A-PP-81** — *Ricevuta di provenienza del pin*: sha + HEAD + rustc +
  ricetta di build emessi **dallo script di build**, e probe di determinismo
  (2 rebuild puliti) per `php-server` **e per `phpr`** prima di usare
  S-94.0 come baseline della leva.
- **A-PP-82** — *Giudice di parità in banda*: `pair94.sh` estrae conteggi e
  **SET dei nomi** dai due `.txt` e scrive `PARITY=OK|DIFF` + set nel
  ratios; i `.txt` (o un estratto integrale macchina-prodotto) committati.
- **A-PP-83** — *Confine per-run di battery61*: reset DB + guardia uploads
  **prima di ogni gamba**, ordine **alternato** in un secondo giro,
  header diffati per NOME, `norm()` ancorato a `_wpnonce` con conteggio
  sostituzioni uguale sui due lati, probe 5 giudicato su `Location`.

## Kill-switch
- **KS-PP-96-1**: qualunque numero la cui catena d'identità risalga a un
  binario **senza ricevuta di provenienza riproducibile** è `grade=ADVISORY`
  **meccanicamente**; il gate cifre rifiuta `VERDICT`. Vale anche per `phpr`
  finché A-PP-81 non è eseguito.
- **KS-PP-96-2**: un `.done` che scrive `rc=0` **incondizionatamente**
  (`pair94.sh:67`) è una ricevuta che non sa dire NO: vietato: il `.done`
  porta l'rc reale di ogni gamba, o il run è NON CONCLUSO.
- **KS-PP-96-3**: nessun claim di parità/accettazione senza il RAW da cui è
  derivato presente a HEAD. Prosa + artefatto untracked = claim ritirato.

## Refutazioni
- **REFUTO** «la coppia gira sul pin invariato» come garanzia: è verificata
  l'identità del file, **non** la sua provenienza — il difetto che ha
  affondato `d45b578` non è stato escluso per `phpr`.
- **REFUTO** «ADVISORY è ciò che Pedersen chiedeva»: chiedevo una
  **ricevuta**, non un'etichetta in prosa senza dente.
- **REFUTO** l'autosufficienza di `pair94.out`: è trascrizione manuale di un
  artefatto untracked, e i suoi conteggi non hanno giudice.
- **REFUTO** «i volatili per NOME» di `battery61.sh`: `norm()` è per FORMA.
- **CONCEDO**: rapporti meccanici, `PHP_CLI_SERVER_WORKERS` nominato,
  failure elencati per NOME, regresso media non attribuito — corretti.

---
## verbale-7-leijen.md

# Verbale 7 — Daan Leijen (allocatore, footprint fisico, semantica dei contatori)
Concilio WP-96 · mandato: REFUTARE

## VERDETTO

**CONTRARIO al grado dichiarato per il footprint; FAVOREVOLE CON EMENDAMENTI
al resto.** Ho letto l'albero COSTRUITO, non la documentazione: il binario di
parità linka **mimalloc v3.0.2**, non v2 (`libmimalloc-sys 0.1.49`,
`build.rs:8-12` — `v3` è il DEFAULT, la feature `v2` non è accesa da
`mimalloc = "0.1"`). Nessun banner di questo progetto lo dice.

Due fatti sopravvivono al mio morso: (a) `MIMALLOC_PURGE_DELAY=0` **funziona
davvero** — `arena.c:2060-2067`: `delay==0` ⇒ `mi_arena_purge()` diretta;
`os.c:650-655` con `purge_decommits=1` ⇒ decommit; `prim/unix/prim.c:495-498`
⇒ su macOS `MADV_FREE_REUSABLE`, che fa contabilità immediata. Il controllo è
in grado. (b) Il picco è un **high-water**: nessun purge può abbassare un
picco già raggiunto. Da questi due fatti discende tutto il resto.

## Emendamenti

- **A-DL-67 — identità dell'allocatore IN BANDA.** Ogni identity block di
  misura registra `mi_version()` e il valore **letto dal processo**
  (`mi_option_get(mi_option_purge_delay)`), mai la riga env dello script.
  `pair94.identity` pinna phpr/oracle/rustc/head e **non** l'allocatore né
  l'opzione che dichiara di governare. *Un controllo dichiarato non è un
  controllo in vigore* — è la lezione del `#if` di WP-95 applicata a una env.
- **A-DL-68 — `max_rss` accanto a `peak_footprint`, sempre, col gap.** Dal
  raw già committato: media rss 2,555× contro pf 3,381×; full rss 1,986×
  contro pf 2,673×. E il segno: phpr ha pf **sopra** il proprio rss di
  142.820.720 B (media) e 369.592.408 B (full), l'oracle ha pf **sotto** il
  proprio rss di 55.966.832 / 72.153.936. Il «regresso» vive nel **gap**
  (compresso/charged-non-residente), non nel residente.
- **A-DL-69 — grado del picco = SCREEN a R=1.** Il picco è una statistica di
  MASSIMO; repair90 impose la MEDIANA per `b_peak` dopo due outlier.
  `pair94.out` firma VERDICT a R=1 sui due rapporti di footprint: incoerente
  con la disciplina di stima di questo stesso progetto.
- **A-DL-70 — ordine INCROCIATO per il footprint.** «oracle-prima» è un
  controllo per la CPU e un **confondente** per il footprint: la seconda gamba
  gira sempre su una macchina il cui stato di compressore è stato plasmato
  dalla prima. Alternare l'ordine fra le ripetizioni.
- **A-DL-71 — la leva si riceve SOLO sul picco CLI.** 21 MB su 44.630.520 =
  47%; sul picco media = 1,8%; sul full = 1,06%. Nessuna cifra media/full può
  ricevere né falsificare la leva.
- **A-DL-72 — α va RI-DERIVATO.** `team-leva.md:168-171` giustifica α≈1 con
  «mimalloc … NON decommitta, le pagine restano committed e riusabili». Sotto
  `PURGE_DELAY=0` è **falso** (v. sopra): decommitta subito. La banda può
  sopravvivere, l'argomento no. Corollario da firmare: 15 arene per-file =
  decommit→recommit→re-fault ripetuti; firmare anche una predizione di
  `page reclaims` dallo stesso `time -l`, o la leva paga in CPU ciò che
  incassa in footprint.
- **A-DL-73 — dente sul punto di DROP.** `N = T_tot − T_max` presuppone che
  l'arena del file *i* muoia prima del parse di *i+1*. Se l'AST del preludio è
  ritenuto attraverso i file, **N = 0** e resta solo la coda 13.738.592 B, che
  vale **zero** footprint: pagine mai faultate non sono mai addebitate. Il
  punto di drop dev'essere un dente, non un presupposto.

## Kill-switch

- **KS-DL-96-1**: ricevuta della leva che cita media/full come evidenza ⇒
  **NULLA** (l'effetto atteso è sotto ogni spread mai misurato su quei picchi).
- **KS-DL-96-2**: qualunque cifra di footprint da un albero il cui `Cargo.lock`
  o `build.rs` del `-sys` differisce dal lock di parità (MI_STAT compreso) ⇒
  **NULLA**. Estensione di KB-78-5 ai knob di build dell'allocatore.
- **KS-DL-96-3**: verdetto di picco a R=1 senza `max_rss` e senza il gap
  pf−rss nello stesso raw ⇒ declassato a **SCREEN** d'ufficio.
- **KS-DL-96-4**: probe slope v2 i cui arm non siano **entrambi** MI_STAT=1, o
  privo di **controllo positivo** (un'allocazione nota di X byte deve comparire
  nel contatore) ⇒ **VOID**; e le sue cifre non si sottraggono mai a pair94.

## Refutazioni

**Sì, una capitale.** La giustificazione di α (`team-leva.md` §3.3) poggia su
una proprietà dell'allocatore che l'albero costruito contraddice **nella
configurazione stessa in cui la predizione sarà giudicata**. Seconda, di
grado: il «regresso media 3,381×» non è stabilito come fatto di programma —
R=1 su una statistica di massimo, e su `max_rss` lo stesso raw dice 2,555×.
Terza, di coerenza: MI_STAT non è esposto fra le feature di
`libmimalloc-sys 0.1.49` (arena, debug, debug_in_debug, extended,
local_dynamic_tls, no_thp, override, secure, v2, win_direct_tls); accenderlo
richiede di patchare `build.rs` **più** la feature `extended` per leggere le
API ⇒ **per costruzione** non è il binario di parità. Il probe è coerente solo
come build di SOLI CONTATORI, mai come sorgente di livelli.

Nota non refutativa ma dovuta: `PURGE_DELAY=0` **non è la configurazione
spedita** (default `purge_delay = 1000` ms, `options.c:140`). Le cifre
descrivono una configurazione che nessun utente esegue, salvo che il binario
imposti l'opzione da codice.

---
## verbale-8-stogov.md

# Verbale sedia 8 — Dmitry Stogov (engine/opcache) — Concilio WP-96

**VERDETTO: CON EMENDAMENTI VINCOLANTI. Una refutazione capitale (Q1) e una
declassazione (criterio 5).**

## Q1 — `stream_get_wrappers`: NON è «correct-or-absent onesto». È split-brain.

Refutato **dal codice**, non da opinione. `crates/php-runtime/src/vm/host.rs:80`
`is_builtin_scheme()` elenca **tutti e dodici** i nomi (`file php http https ftp
ftps data glob phar zip compress.zlib compress.bzip2`) e li usa per **rifiutare**
`stream_wrapper_register` («Protocol ftp:// is already defined.», host.rs:6049).
La lista restituita a userland ne dichiara **cinque**. In PHP c'è **UNA**
tabella (`url_stream_wrappers_hash`): `stream_get_wrappers`, il guard di
`register`, `unregister` e `fopen` la leggono tutti. Qui sono **tre** tabelle
(la terza è il fallback di `stream_is_local`, host.rs:6030) e **si contraddicono**.

Conseguenze per NOME, entrambe osservabili:
1. phpr **occupa** `ftp`/`phar`/`zip` e **nega** di averle: userland non può né
   usarle né fornirle. In PHP l'uscita di sicurezza esiste ed è un idioma
   diffuso — `stream_wrapper_unregister('phar')` (hardening Composer/plugin WP),
   poi eventuale re-register. In phpr quell'`unregister` ritorna **false +
   warning** dove PHP ritorna **true**: divergenza mai catturata dalla suite.
2. §2.4 del catalogo dichiara `stream_get_wrappers` **differito** per i wrapper
   userland ⇒ un `stream_wrapper_register('vfs')` **non compare** nella lista ⇒
   `wp_is_stream('vfs://…')` falso. vfsStream/PHPUnit e Flysystem vivono lì.

Cosa fa PHP davvero: registra i wrapper al MINIT per **build** (senza ext/zip
niente `zip` — quindi «assente per build» è legittimo **in PHP**), e
`allow_url_fopen=0` **non** toglie il nome dalla lista: PHP elenca ciò che
**non aprirà**. Precedente decisivo: **la presenza nella lista significa "questo
schema è dell'engine, non trattarlo come path"**, ed è esattamente ciò che
`wp_is_stream` chiede (`in_array($scheme, stream_get_wrappers(), true)`).
Omettere il nome fa cadere il path in `path_join`/`realpath`/`mkdir` su
`"ftp:/example.com"`: **errore silenzioso**. Dichiararlo dà un **errore
rumoroso** all'open. Su questo asse il "correct-or-absent" va applicato al
**verbo**, non al **nome**.

## Emendamenti

- **A-DS-96-1 (coerenza, non-negoziabile qualunque scelta)**: `is_builtin_scheme`
  **abolita**; una sola registry `nome → {Native | Userland(classe) |
  Declared-Unimplemented}` letta da `stream_get_wrappers`, `register`,
  `unregister`, `restore`, `fopen`, `stream_is_local`. Invariante pinnata:
  `∀n ∈ stream_get_wrappers(): register(n,C)===false` **e** `unregister(n)===true`.
- **A-DS-96-2**: i wrapper userland **entrano** nella lista (oggi differiti), e
  l'**ORDINE** è quello di registrazione — PHP itera hash-order, **non**
  alfabetico. Verificare se phpr ordina: sarebbe una seconda divergenza latente.
- **A-DS-96-3 (graduato)**: `glob://` e `compress.zlib://` **implementati per
  davvero** (zlib è già FFI, glob già esiste come funzione) — due nomi tolti dal
  gap onestamente. `ftp/ftps/zip/phar/compress.bzip2` → `Declared-Unimplemented`:
  elencati, con fallimento **PHP-shaped** all'open (testo del warning esatto,
  `url_stat`→false **senza** warning) e **un phpt di pin per nome**. Uno stub che
  elenca e fallisce a caso è peggio dell'assenza: vietato senza il pin.
- **A-DS-96-4 (battery61, normalizzatore)**: `s/[0-9a-f]{10}\b/` è **troppo
  largo su tre assi**: (a) niente `\b` a sinistra ⇒ normalizza la **coda** di
  ogni md5/sha (32 hex → ultimi 10 cancellati); (b) `[0-9a-f]` include le
  **cifre** ⇒ **ogni intero decimale a 10 cifre**, cioè **ogni timestamp Unix**
  (`1785801803` è nell'`.out` stesso), sparisce — proprio la classe dove
  `time()`/`date()`/`uniqid()` divergerebbero; (c) è per **forma**, non per
  **nome**, contro la regola già scritta nel commento del file. Sostituire con
  cattura **contestuale** (`name="_wpnonce" value="(…)"`, `_wpnonce=(…)`,
  `"nonce":"(…)"`, `_ajax_nonce`) + **conteggio sostituzioni per lato stampato
  e preteso uguale**.
- **A-DS-96-5 (battery61, copertura)**: si confronta solo `head -1` dell'`.hdr`:
  `Content-Type`, `Set-Cookie` (flag HttpOnly/SameSite/path), `Location`,
  `Cache-Control` **non sono mai confrontati**. Probe `5-loginpost` è
  `BYTE-ID … bytes=0`: **due corpi vuoti** — dente **vacuo** per il criterio che
  questa stessa sessione ha appena imparato (lezione 4). Il suo contenuto
  probatorio sta negli header: confrontarli, o marcarlo `VACUOUS-BODY`.
- **A-DS-96-6 (leva, tabella dei simboli)**: il preludio va reso **come la
  function-table interna di PHP** — nomi/arità/segnatura **eager**, **corpi**
  lazy. Pin a tre istanti (start / dopo aver forzato il parse di UN file / fine):
  `get_defined_functions`, `get_declared_classes`, `get_declared_interfaces`,
  `get_defined_constants` identici **e nello stesso ORDINE**. L'ordine di
  `get_declared_classes` è osservabile ed è ordine di dichiarazione.
- **A-DS-96-7 (leva, binding)**: early binding. In PHP `class B extends A`
  top-level si lega a compile-time solo se `A` è già legata, altrimenti
  `DECLARE_CLASS_DELAYED`. Il parse lazy per-file **cambia chi è già legato** ⇒
  cambia l'ordine, `class_exists('B', false)`, e i messaggi «Cannot redeclare
  X()» / «Cannot declare class X, because the name is already in use» —
  che devono restare **identici e nello stesso punto** con e senza pigrizia.
- **A-DS-96-8 (leva, visibilità degli errori di parse)**: un errore di sintassi
  in un file di preludio **mai chiamato** oggi è fatale; col parse lazy diventa
  **invisibile**. Bite-test: iniettare l'errore, pretendere lo stesso fatale.
- **A-DS-96-9 (leva, lifetime)**: nessun artefatto compilato può sopravvivere
  alla propria arena — default-arg AST, attributi ritenuti per reflection,
  stringhe internate condivise. Falsificatore = il corpus **refl 290** eseguito
  **dopo** il drop delle arene, non prima.

## Kill-switch

- **KS-DS-96-1**: se A-DS-96-1/2/3 muovono anche **un solo nome** in corpus 1418,
  refl 290, ORM 3E/13F o hk 1665 → si tiene **solo** il pin di catalogo, il
  cambio di lista si annulla.
- **KS-DS-96-2**: se la leva per-file altera l'**ordine** di
  `get_declared_classes`/`get_defined_functions` di **un** elemento → la leva è
  **morta a quello step**. Non si adatta il test.
- **KS-DS-96-3**: se il bite-test di A-DS-96-4 (divergenza sintetica a 10 hex /
  10 cifre fuori dai nonce) **non** porta rc=1, il NORM-ID di S-94.0 è
  **ADVISORY** e il **criterio 5 torna PARZIALE**.

## Refutazioni

1. **CAPITALE** — «reticenza onesta» è falso: la registry di `register` già
   rivendica i dodici nomi. Non è assenza, è **incoerenza fra due tabelle** che
   in PHP sono una sola; e la metà nascosta (`unregister('phar')` che ritorna
   false) è peggiore di quella misurata.
2. Il normalizzatore NORM-ID è **per forma** e ingoia timestamp a 10 cifre e code
   di hash: la dashboard non è provata identica, è provata **non-distinguibile
   dal filtro che l'ha giudicata**. Senza controllo negativo non è un giudice.
3. `5-loginpost BYTE-ID bytes=0` è un PASS vacuo, dalla stessa famiglia della
   lezione 4 di S-94.0: il dente non annuncia di aver smesso di mordere.

---
## verbale-9-gregg.md

# Verbale 9 — Brendan Gregg (mandato INVERSO: giudico l'oggetto, non l'apparato)

## VERDETTO: **LA MISURA C'È, LE QUATTRO LETTURE SONO SBAGLIATE.**

Il contatore full/media **è davvero azzerato**: coppia stessa-sera, ordine
oracle-prima, conteggi identici (30472/4558029/86W/73S), raw committati,
guardia DB+uploads. Non è dichiarazione di comodo. Ma S-94.0 ha letto
**rapporti contro una banda**, mai **assoluti contro assoluti** — e i suoi
stessi raw, incrociati con `gaps/REPORT_GAP_64.md` (riga 11, «peak phys
time -l: oracle 393,7MB / phpr 1.186,9MB»), ribaltano tre letture su quattro:

| asse | gamba **phpr** (Δ vs WP-64) | gamba **oracle** (Δ) | lettura di S-94.0 | lettura vera |
|---|---|---|---|---|
| media peak | 1186,9→1170,8 MB = **−1,4%** | 393,7→346,3 MB = **−12,0%** | «REGRESSO» | **il numeratore NON è cresciuto**: il rapporto sale perché scende il denominatore |
| full CPU | 788-800u → **796,78u** (dentro il range) | ~376-386u → 421,51u = **+9,3/+12,0%** | «il più basso mai registrato» | **phpr è PIATTO**: il record è dell'oracle che rallenta |
| full peak | 2,035-2,049 → **1,993 GB = −2,4%** | — | «MEGLIO» | **dentro** lo spread inter-coppia documentato (70 MB = 3,4%) ⇒ rumore |
| media CPU | 53,85→55,50u = **+3,1%** | 20,92→21,03u = +0,5% | «poco peggio» | **l'unico movimento vero di phpr**, al bordo dello spread A-A′ (1,26-3,16%) ⇒ SCREEN |

Ho falsificato due mie ipotesi prima di scriverlo: (a) «u+s vs user» — il
rapporto user-only è 1,890 vs 1,873, artefatto ESCLUSO; (b) «workload
cambiato» — il fail-set 2F+86W = 88 è byte-identico a run33, ESCLUSO.

**Le quattro cifre sono osservate, non attribuite.** Peggio: sono attribuite
*male* — a phpr — quando l'aritmetica le attribuisce al banco di prova.
GAP_TREND porta già l'avviso, in chiaro, alla riga WP-30 («il rapporto sale
per rumore dell'oracle, non per una regressione phpr»). Il costo
dell'apparato non è solo il probe slittato: sono i **dieci minuti di
aritmetica su numeri già in repo** che avrebbero reso corretta l'unica
misura prodotta.

Chiudere così **non è accettabile**, ma il difetto non è «regresso senza
canale»: è **regresso inesistente, pubblicato in GAP_TREND come tale**.

## Emendamenti

- **A-BG-76** — GAP_TREND registra per ogni asse le **quattro cifre assolute**
  (num/den × due epoche) accanto al rapporto. Riga con soli rapporti =
  respinta alla rotazione.
- **A-BG-77** — **Δ sulla gamba phpr o non è un claim su phpr.** Le parole
  MEGLIO/REGRESSO sono ammesse solo dopo la decomposizione numeratore/
  denominatore. Correggere `REPORT_GAP_94.md`, `MEASURE94_RESULTS.md`, riga
  WP-94 di GAP_TREND e §FONDAMENTALI **prima** di S-95.0.
- **A-BG-78** — un rapporto tonight-vs-banda-storica è **SCREEN**, mai
  VERDICT: il grado VERDICT copre la coppia, non il confronto cross-epoca
  (nessun controllo stessa-sera del punto storico). Coerenza con LEVER-2.
- **A-BG-79** — **l'oracle è strumento, non costante**: R≥3 sulla sola gamba
  oracle del media group (≈21s CPU × 3 ≈ 1 min) a ogni rotazione, come
  taratura del banco. Il −12% di memoria dell'oracle a CPU invariata (+0,5%)
  è un canale APERTO e nominato.
- **A-BG-80** — il giudice della leva S-95.0 è il **contatore per-unità con
  controllo positivo** (Σ T_i ≈ 25795552 B) più la **gamba phpr assoluta**;
  la coppia conferma, non giudica. Effetto atteso sul media peak ≈ 25-39 MB
  su 1170,8 MB = **2-3%**, cioè al bordo dello spread: pretendere R≥3.

## Kill-switch

- **KS-BG-96-1** — leva giudicata da un RAPPORTO ⇒ verdetto **VOID**.
- **KS-BG-96-2** — «regresso/miglioramento» pubblicato senza le due gambe
  assolute ⇒ riga **ADVISORY**, mai VERDICT.
- **KS-BG-96-3** — S-95.0 apre la leva senza aver corretto la riga WP-94 ⇒
  **STOP**: si costruirebbe la predizione WP-48 su un «prima» mal letto.

## Ordine per S-95.0

**La leva resta prima** — ha un canale proprio, indipendente dalla coppia:
è la scelta giusta per l'oggetto. Ma prima di essa, **30 minuti**: correggere
la riga WP-94 (A-BG-77) e tarare la gamba oracle (A-BG-79). Il canale di
attribuzione del «regresso» **non serve**: non c'è regresso da attribuire.

## Che cosa sappiamo oggi che ieri non sapevamo

1. **phpr è FERMO.** Su tre assi su quattro il suo assoluto non si è mosso
   oltre lo spread in trenta sessioni: media peak −1,4%, full CPU +0,8%,
   full peak −2,4%. Nove sessioni di roadmap footprint: **nessun movimento
   misurabile sull'oggetto.** È la notizia della sessione, ed è dura.
2. **L'unico movimento reale di phpr è in PEGGIO**: media CPU +3,1%.
3. **Il banco di prova è derivato del 12%** (oracle: memoria −12% a CPU
   piatta sul media; CPU +9-12% sul full). Ogni claim cross-sessione
   costruito sui rapporti dal WP-64 in poi va riletto.
4. **La batteria WordPress nativa esiste e morde** (5 BYTE-ID + NORM-ID,
   rc=0): questo sì è avanzamento durevole dell'oggetto.
5. **`stream_get_wrappers`** è l'unica divergenza phpr su 30472 test:
   confermata, non nuova.

