# Team CATENA — nota di relatore (fase 2, Concilio WP-98)
Sedie: Klabnik (v-3), Pedersen (v-6), Hejlsberg (v-4). **I verbali individuali
restano la fonte vincolante**; questa nota riconcilia dove si può e REGISTRA i
dissensi dove non si può.

## 1. Un difetto o due? — due meccanismi, una forma
Klabnik: il perimetro si fida della FORMA che git sceglie di stampare (quoting
nello SPAZIO). Pedersen: l'identity si fida di un NOME (`head=`) il cui
referente può ancora cambiare (mutabilità nel TEMPO). Meccanismi distinti,
stessa classe: **il giudice tratta come dato ciò che è una rappresentazione
prodotta altrove**. E i due si toccano materialmente: l'amend che orfanizza
`7847cc0` è quello che porta in `83661e4` la cura del giudice (+13/−2 su
`gate-measure-cifre.sh`). *La cura dell'uno è la causa dell'altro.*

**Il pattern va nominato: P-AMEND-ORFANO** — un artefatto registra `HEAD`, poi
un `--amend` sostituisce l'oggetto: la citazione sopravvive, il referente no,
e al primo `gc` la provenienza diventa **irrisolvibile** (non solo
indimostrata). Il team adotta **A-PP-98-3** come dente, non come lezione:
o `refs/measure/<run>` prima del run, o `head=` scritto dopo l'ultimo amend.

## 2. La cura di Klabnik: forgia CHIUSA, classe APERTA
Verificato: riga 1265 usa `-c core.quotePath=false … ls-files --others -z`,
split su NUL; stessa correzione su `check-ignore -v`; **T31 con il suo morso**
(braccio pre-98 che deve vedere 1/2 gemelli, altrimenti il dente è vacuo).
Ma la classe ha almeno **due altri siti**, verificati a macchina:

- **`gate-measure-cifre.sh:897`** — `split /\n/, qx(git ls-tree -r --name-only
  HEAD)`. `ls-tree --name-only` **quota** (provato: `"perimetr\303\262.md"`).
  È l'autorità *committed-only* (A-SK55) e la fonte di `@headtree`/`%headset`:
  un file di corpus o un `m9*.campaign.ledger` con nome non-ASCII sarebbe
  **invisibile all'ancoraggio** e la campagna non verrebbe mai validata.
  Cura simmetrica: `-c core.quotePath=false … -z` + `split /\0/`.
- **~20 siti `git status --porcelain`** (measure8x/9x-campaign, battery-8xpre,
  e la cattura di `tree_dirty`). Qui i due difetti si sommano: porcelain
  **quota** *e* collassa una directory untracked in una riga (`?? d/`) —
  esattamente il «6 che sottostima senza limite» di Pedersen. Solo `-z` **e**
  `-uall` insieme chiudono entrambi (provato). Minore: `diff --name-only`
  (`battery-equivalence.sh:217/223/498`) quota allo stesso modo.

## 3. Dove iscrivere il debito di Hejlsberg
Convergenza netta: **A-AH-98-3 (iscrivere) + A-PP-98-7 (i riferimenti come
dente)**. Iscrizione in tre luoghi — TODO master PER NOME, `NEXT_SESSION`,
marker `TODO(port)`-grade sopra `zvalcensus.rs:81-85` — e **A-PP-98-7 che fa
risolvere quei riferimenti a HEAD**, altrimenti FAIL. Senza il dente,
l'iscrizione è un altro verbale: KS-AH-98-2 è la sola forma che morde.

## 4. Dissensi REGISTRATI (non appianati)
- **Verdetto passo 2.** Klabnik: **non omogeneo** — banda SCREEN R=1 contro
  costo storico di altro workload/era; F3 preclusa su un confronto mai fatto
  alla pari (A-SK-98-3). Pedersen: **NULLO, non contrario** — uno spareggio con
  entrambe le premesse refutate *decade*; F3 **SOSPESA**, non archiviata.
  Hejlsberg: **regge** — «argomentato invece che subìto»; è il §WP-97 punto 1 a
  non stare in piedi, e **non va messo per primo**.
- **La forma `LoadSlot{take}`.** Klabnik: **mai valutata**, e §5.1 ammette che
  cambierebbe il verdetto. Hejlsberg: **valutata e refutata** (RC-1: layout
  neutro, ma tassa 60,6 M letture per servirne 9,99 M — «la peggiore delle
  due»). Il team nota che Hejlsberg stesso dichiara il netto «indistinguibile
  da zero… il numero che nessuno ha intenzione di produrre»: **Klabnik ha
  ragione sul GRADO (argomento, non misura), Hejlsberg sulla DIREZIONE.**
- **Verdetti complessivi divergenti**: FAIL (K) · RESPINTO IN PARTE (P) ·
  REGGE-nel-perimetro (H).
- **Convergenza operativa**: KS-SK-98-2 (pre-fix `e318fbfc` non committato ⇒
  morso `SALTATO` e muto) e A-PP-98-2 (nessun `.out` è autorità se non
  riproducibile) sono **la stessa regola**: ricostruibile dal commit, o
  `grado=interno` scritto in testa al file.
