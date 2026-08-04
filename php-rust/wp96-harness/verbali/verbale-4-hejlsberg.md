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
