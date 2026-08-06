# Verbale 6 — PEDERSEN (confine per-richiesta, lifecycle, collaudo server) — Concilio WP-105

## VERDETTO

Il grading di **31aa7c2e** col launcher emendato è VALIDO nel suo perimetro,
ma il perimetro dichiarato è PIÙ LARGO dell'evidenza. Confermo «gradato
MINIMO-emendato»; REFUTO tre claim di confine e ordino la restrizione a
registro. La bozza S-104 è accettabile con gli emendamenti sotto.

## Refutazioni

**R1 — «cross-worker» non provato.** Gamba E gira su workers=1: prova il
confine cross-RICHIESTA stesso worker, mai il cross-WORKER dopo un fatal.
L'interleave (gamba D) non include cbE: fatal sotto concorrenza mai
esercitato. Inoltre il braccio phpr fa UNA sola cbE (l'oracle due): il
caso errore-poi-errore (marca che si accumula tra due fatal) è cieco; e
cbE arriva sempre a server caldo (dopo cb1×3+cb2×2) — fatal come PRIMA
richiesta mai visto. Il commento del launcher («contaminazione
cross-worker di diag_line_marks») promette ciò che D non giudica.

**R2 — copertura dei due worker non dimostrata.** In gamba D nulla prova
che ENTRAMBI i worker abbiano servito richieste (12 curl sequenziali su
connessioni fresche: la distribuzione è un'ipotesi di routing, non
un'evidenza). Se un worker è rimasto muto, workers=2 ≡ workers=1.

**R3 — «confine HTTP» = solo corpo.** `curl -s` scarta status e header:
`cmp` giudica il body. Un pin che rispondesse 200 dove l'oracle dà 500
(o viceversa) graderebbe uguale. Il confine dichiarato è HTTP, il
giudicato è il payload.

**R4 — symlink: divergenza REALE aggirata, non catalogata.** php -S
canonicalizza il docroot, phpr no: i byte dei warning divergono per
chiunque serva da docroot symlinkato (/tmp su macOS!). Il DOC canonico
nel launcher è legittimo (giudica il motore), ma la divergenza vive solo
in un commento e in una riga di PIN_REGISTRY — viola «fedeltà o assenza
CONSAPEVOLE»: va in `PHPR_DIVERGENCES_FROM_PHP.md` con decisione
esplicita (canonicalizzare come l'oracle, o assenza dichiarata).

**R5 — lo stash a regola narrativa recidiverà.** da5c2948 è la prova: la
lezione era già nota (KS-PE-100-3 vieta cifre su pin non collaudati, ma
nulla lega grading→stash). Una regola che dipende dal ricordarsi non è
disciplina di lifecycle.

## Emendamenti

- **A-PE-105-1** (launcher S-104): gamba E estesa — (i) cbE×2 consecutive
  lato phpr (errore-poi-errore byte-id); (ii) cbE come prima richiesta a
  server freddo; (iii) cbE dentro l'interleave workers=2 con cb2
  post-fatal giudicata. Fino ad allora, il registro dice «cross-richiesta
  stesso-worker», MAI «cross-worker».
- **A-PE-105-2**: catalogare la divergenza symlink-docroot in
  `PHPR_DIVERGENCES_FROM_PHP.md` (decisione, non aggiramento); il DOC
  canonico resta come scelta d'harness DICHIARATA nel launcher.
- **A-PE-105-3**: stash MECCANICO nel launcher — a fails=0 il launcher
  stesso copia il binario in uno stash keyed-by-hash, riverifica lo
  sha256 della copia e scrive il path nel `.done`; niente copia ⇒ niente
  verdetto «gradato».
- **A-PE-105-4**: confine giudicato con `curl -sD` — status line + header
  stabili (carve-out per Date/Server/volatili) confrontati con l'oracle,
  almeno sui bracci cb2/cbE.
- **A-PE-105-5**: gamba D esige evidenza di copertura di entrambi i
  worker (contatore per-worker o marcatore nel log), o si rinomina.

## Kill-switch

- **KS-PE-105-1**: NESSUNA cifra server (coppia WP, peak, CPU, parità
  full) attribuibile a 31aa7c2e finché non passa il grado PIENO (option
  413 + restapi 3508 per NOME, env -i, ×2 modi). Il minimo-emendato
  autorizza i soli gate fixture.
- **KS-PE-105-2**: un verdetto «gradato» il cui stash non esiste su disco
  al momento del `.done` è VOID; il pin non entra a registro come
  «collaudato: sì».

## Priorità S-104

1. Ordine §1 (verdetto A/B) invariato: i pin S-99/S-100 sotto misura sono
   gradati PIENI — le cifre lì sono lecite.
2. Ordine §2 (leva H-C2): la coppia WP bimodale che salda il debito
   S-103 esige un pin server NUOVO post-leva, collaudato minimo-emendato
   **+ A-PE-105-1/-3/-4** e poi PIENO PRIMA delle cifre. NON spendere il
   grado PIENO su 31aa7c2e: ruota comunque con la leva — si grada PIENO
   il pin che porterà le cifre.
3. A-PE-105-2 (catalogo symlink) entra nell'igiene timeboxata di §5,
   insieme al generator-in-cycle già in §4.
