# Verbale sedia 3 — Klabnik (spec, testabilità, matrici e gate) — WP-98

## VERDETTO

**FAIL — una forgia è ATTERRATA.** Il canale env di git è chiuso (T27-T30
tengono), ma il perimetro A-SK-96 è CIECO ai nomi di file non-ASCII. Ho
provato tutte le vie del mandato; T27..T30 hanno retto ai loro attacchi
letterali (env, GIT_CONFIG_*, PERL5OPT, depth-marker). Ma A-SK-96 non passa
per l'ambiente: passa per come `git ls-files --others` STAMPA i path, e lì il
giudice si fida di un ancoraggio che il quoting di git rompe. Un doc non
committato con una cifra fabbricata e una lettera accentata nel nome ESCE dal
perimetro, non nominato, mai giudicato. `--all` a albero pulito darebbe
`PASS` rc=0 con la cifra in albero.

## Emendamenti A-SK-98-n

- **A-SK-98-1 (il buco)**: la lista untracked va letta con
  `git -c core.quotePath=false ls-files --others -z -- php-rust` e spezzata su
  NUL, MAI su `\n`. Oggi `qx(git … ls-files --others -- php-rust)` (riga 1223)
  con `core.quotePath=true` di default avvolge i path con byte >127 in
  `"php-rust/…\303\262….md"`: l'ancoraggio perl `qr{^php-rust/.*\.md$}` (1204)
  NON matcha la stringa quotata (inizia con `"`, finisce con `"`), e il doc
  sfugge. Il `-z` chiude anche il vettore newline-nel-nome (uno `split /\n/`
  spezza un nome con `\n` in due path fantasma).
- **A-SK-98-2 (dente permanente T31)**: pianta un doc con cifra e nome
  non-ASCII sotto un perimetro-classe, esegui `--all`, e PRETENDI che il path
  sia NOMINATO in una riga FAIL. Braccio-morso: lo stesso su un giudice
  pre-A-SK-98-1 deve NON nominarlo (la cecità è reale). Senza morso è un
  ricordo, non un dente.
- **A-SK-98-3 (grado del passo 2)**: il verdetto di `design96` va marcato
  SCREEN per campo. Confronta una banda derivata da canale R=1 (SCREEN) col
  costo storico di un corpo caldo (WP-41/44, altro workload/era): non
  omogenei. §5.1 nomina una forma d'emissione (`LoadSlot` con flag `take` a
  compilazione) «che potrebbe cambiare il verdetto» e MAI valutata. Il passo
  2 preclude F3 su un confronto che ammette di non aver fatto alla pari.

## Kill-switch KS-SK-98-n

- **KS-SK-98-1**: finché A-SK-98-1 non atterra e T31 non è verde, ogni
  `PASS --all` è provisional PER NOME: il perimetro non copre i nomi non-ASCII
  né i newline. Nessun PASS di cifre è verdict-grade sotto questa apertura.
- **KS-SK-98-2**: il «morso del fix» di `check-liveness-fixtures.sh` dipende
  da un binario pre-fix (`phpr-pre-target`, sha e318fbfc248a8e35) costruito a
  mano e NON committato. Se sparisce, l'arco `morso del fix` stampa `SALTATO`
  e tace: la differenza `would_take 6→4` non è più provata dal repo. Se il
  pre-fix manca all'apertura S-97, l'arco è dichiarato provisional finché non
  si ricostruisce dal commit 83661e4 (oggi raggiungibile).

## Refutazioni capitali (col comando riprodotto)

**SÌ — A-SK-96 non vede i nomi non-ASCII.** Piantati due doc-cifra gemelli,
uno ASCII e uno accentato; solo l'ASCII è stato nominato dal giudice:

```
$ printf 'ASCII: picco 123480 B\n'  > wp98-harness/zzforge-k13-ascii.md
$ printf 'accentato: picco 123472 B\n' > wp98-harness/zzforge-k13-perimetrò.md
$ bash wp81-harness/gate-measure-cifre.sh --all 2>&1 | grep zzforge-k13
FAIL … UNCOMMITTED php-rust/wp98-harness/zzforge-k13-ascii.md … NO manifest entry
# (zzforge-k13-perimetrò.md: NESSUNA riga — sfuggito)

$ git ls-files --others -- php-rust | grep zzforge-k13
"php-rust/wp98-harness/zzforge-k13-perimetr\303\262.md"   # quotato → ^php-rust/ non matcha
$ perl -e 'print( q{"php-rust/…\303\262….md"} =~ m{^php-rust/.*\.md$} ? "IN":"FUORI")'
FUORI
```

Il canale env resta chiuso (nessuna forgia env/config/exclude/argv è
atterrata: T27-T30 nominano correttamente ogni attacco letterale; `.git/config`
core.excludesFile è neutralizzato dal `-c …=/dev/null`, `.git/info/exclude`
cade su `head_blob_sha` vuoto). Il difetto è UNO: il perimetro fidava della
FORMA testuale che git sceglie di stampare.

**Residui**: tutti i miei forge (`zzforge-k13-*.md` + AppleDouble `._`)
rimossi; `.git/config`, `.git/info/exclude`, working tree = baseline
(verificato). Nessun commit.
