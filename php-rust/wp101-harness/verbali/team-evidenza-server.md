# Team «evidenza-server» — Klabnik (3) + Pedersen (6) — Concilio WP-101

## Nota di fatto (relatore)
Il morso Klabnik R1 (dente anti-putenv: chunk del `{main}` agganciato per
substring, entrambi i bracci vacui) è stato CONFERMATO A MACCHINA e RIPARATO
in chiusura di S-99: match sull'HEADER del chunk; i due bracci ri-passano sul
bersaglio giusto. Commit in main. A-KL-101-1 è quindi ESEGUITO; resta aperta
la parte «assert che il chunk non sia il {main}» come dente permanente.

## Convergenze
1. **«Collaudato: sì» è più largo dell'evidenza, per entrambe le sedie.**
   Klabnik R2 e Pedersen R1 dicono la stessa cosa da due lati: la gamba
   server poggia su DUE richieste sequenziali identiche (sentinella), mentre
   option 413 + restapi 3508 girano via phpr CLI — collaudano l'EMISSIONE,
   non il lifecycle. Riconciliazione: il registro GRADUA per gamba
   (A-PE-101-1: emissione-CLI / server-smoke / server-N-req /
   server-HTTP-suite). «Server collaudato largo» = N≥16 richieste, payload
   interleaved diversi, workers>1, almeno una restapi-shaped via HTTP
   (A-PE-101-3). Nessuna cifra WP attribuibile a un pin senza gamba
   esplicita (KS-PE-101-2).
2. **Il server non ha MAI servito flag-on: gate bloccante prima del flip.**
   KS-KL-101-1 e KS-PE-101-1 coincidono: flip VOID finché sentinella +
   option + restapi non passano col registro ACCESO, e la parità server nei
   DUE modi non passa sulla STESSA rotazione. Serve il launcher bimodale
   (A-KL-101-6): la lista chiusa `env -i` oggi non sa esprimere i due modi.
3. **Il flip inverte la semantica di «flag assente» e i denti vanno
   riscritti PRIMA.** Klabnik R7(i) e Pedersen R2 convergono: opt-out con
   grafia definita e pinnata (`PHPR_REG_LOWER=0`), bracci anti-putenv
   riscritti nel modo nuovo (assente⇒on, =0⇒off, putenv impotente in TUTTE
   le direzioni), commento-garanzia dello script di parità aggiornato
   (A-PE-101-4 ⊇ A-KL-101-5). Chi lo chiede: entrambi; ordine: opt-out →
   bracci → script → solo poi flip.
4. **Strumenti che quadrano su sé stessi**: assert conteggi↔nomi
   (A-KL-101-3, KS-KL-101-3) come precondizione di ogni gate citato al flip.

## Conflitti
Nessun conflitto sostanziale; una differenza d'enfasi: Klabnik vincola il
flip anche alla definizione operativa di A-KL-100-2, Pedersen no. Il team la
adotta (è il solo giudice dell'identità off↔on): **diff normalizzato
per-test dei dump/output off↔on, criterio pre-registrato = zero differenze**
(A-KL-101-4); il cronometro non giudica mai l'identità (KS-KL-101-2).
Secondo scarto: Pedersen aggiunge identità di ricetta (`--build-info`,
A-PE-101-5) che Klabnik non tratta — adottata senza obiezione.

## Priorità per l'ordine S-100
1. Grafia opt-out definita e pinnata nel funnel (pre-flip, bloccante).
2. Riscrittura bracci anti-putenv + funnel/M5 per il mondo post-flip
   (A-PE-101-4; sopra il fix header già in main).
3. Launcher parità bimodale + collaudo server flag-ON: sentinella estesa
   (N≥16, interleaved, workers>1, restapi via HTTP) + option + restapi.
4. A-KL-101-4: diff per-test off↔on, zero differenze, pre-registrato.
5. Registro pin: campo `collaudato:` graduato per gamba + `--build-info`.
