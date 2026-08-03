# WP_SESSION_93.md — S-93.0 "L'ARENA CHE MORIVA IN SILENZIO E IL CONTATORE CHE NON LO DICEVA" — direttiva utente A/B/C eseguita

**In una frase**: abbiamo scoperto che i quaranta megabyte "mai liberati"
che ogni processo del server sembrava trattenere erano in realtà memoria
di lavoro del caricamento iniziale, già restituita subito dopo l'uso — era
il contatore a non registrare la restituzione — e con questo abbiamo
trovato il vero grande spreco da aggredire: la memoria temporanea del
caricamento, che oggi costa più di quattro volte il dovuto anche al
comando a riga di comando.

**Data**: 2026-08-03
**Scope**: DIRETTIVA UTENTE 2026-08-03 (S-93.0, sopra l'ordine del
Concilio WP-94): A riparazione di autorità in timebox · B la questione
dei worker (B1 nomi, B2 natura, B3 leva misurata) · C giudizio di senso
WordPress-su-Axum. Modello verificato all'apertura: Fable 5.
**Commit**: 2859c81 (A) → 070fabf (B) → d011a86 (C), tutti su main,
pushati.
**Binari**: phpr **d5ce86e3342f3926 INVARIATO** (mai ricompilato) ·
release/php-server **d45b57843eeb1375 INVARIATO** (RIPRISTINATO dal pin
dopo i probe; i build strumentati mem-census sono identità dichiarate in
huge-sites.out). Budget corpus in vigore 24363 (alzato stesso-commit col
glob wp93, A-SK61).

## Ordine eseguito (direttiva A/B/C)

| # | Esito | Commit |
|---|---|---|
| A | **Riparazione di autorità (unica quota apparato, DENTRO il timebox)**: A-SK-82 tether su BASH_SOURCE[0] — REFUSE in OGNI modo se vuoto o != $0 (il forge Klabnik `bash -c "$(cat patched)" /path/pristine` firmava di nuovo un omonimo un livello sopra) + dente T23 (arm-a: giudice guardato via canale -c REFUSE rc=1 per NOME; arm-b: MORSO della copia con guardia strippata = would-have-passed rc=64 — legge WP-92) · A-AH68 BATTERY_NAME dal CONTENUTO ancorato (riga terminale A-SK36 sha-ricomputata), basename in disputa = REFUSE, predicato unico bnc_judge consumato dal percorso reale E da --selftest-identity, stamp ledger match a 5 campi. Selftest T0..T23 PASS rc=0; --all PASS a 2859c81 (judge_sha 3e16e1232bd48e86) e a d011a86 (judge_sha 2f37f386d153d6ea post-glob) | 2859c81 |
| B1 | **I sei siti huge NOMINATI dal vivo** (hook env-gated PHPR_HUGE_TRACE=1 nel GlobalAlloc mem-census, W=1, lldb NEGATO dal sistema — via alternativa sanzionata dalla direttiva): sei chunk di RADDOPPIO (622576 → 1245168 → 2490352 → 4980720 → 9961456 → 19922928, x2+16, somma 39423200) di UNA arena bumpalo — l'arena AST del parse del PRELUDIO stdlib in lower_prelude_uncached (OnceCell per-thread, da main_unit_acquire nel worker_loop). Numero FISSO riprodotto (6 dopo req1, 6 dopo req3). **UNIT_CACHE, STUBS, arene dei Module FALSIFICATI come proprietari** (nessun backtrace li attraversa). Evidenza: wp93-harness/huge-sites.out (ADVISORY, ascii-nudo) | 070fabf |
| B2 | **Natura: SPIKE TRANSIENTE, non riserva né leak** — tutti e sei DEALLOCATI (Drop della Bump, ordine inverso) dentro la prima richiesta. La conclusione «mai liberata» di huge-worker.out è REFUTATA: la free esiste, è la statistica malloc_huge dei dump m90 a non decrementare MAI (cumulativa; current==total anche a exit_collect_mi). Il controllo positivo del decremento (Concilio WP-94 E2) è ESEGUITO dal lato Rust: NON morde sul canale stat | 070fabf |
| B3 | **LEVER-2 (mi_collect on-thread post-preludio) REFUTATA CON MISURA**: arm A/B same-binary (PHPR_PRELUDE_COLLECT), slope ctrl 18814309 vs lever 18792472 B/worker → delta 21837 (0.12 per cento, NULLO), coerente con arena committed invariata a exit_collect_mi nei raw m90. **Canale della leva vera quantificato PRIMA (WP-48)**: contatore PHPR_PRELUDE_STATS committato — prelude-arena allocated_bytes=39534144, coda mai usata 13738592; costo CLI parity su hello: peak footprint 44630520 vs oracle 10093048 (rapporto 4.42). Leva nominata per la prossima sessione: **arene PER-FILE del preludio** (gate parità COMPLETI obbligatori) | 070fabf |
| C | **Giudizio di SENSO scritto, zero codice** (wp93-harness/GIUDIZIO_C_AXUM.md): il pool serve alla MISURA; WordPress resta sul modo nativo (WP-61); portage NON giustificato oggi, condizione di riapertura nominata (braccio a carico reale sul canale per-worker dopo le leve hello-grade e il canale m91). Debito ribadito: battery61 riproducibile sul modo nativo = criterio 5, in cima al §WP-94. Fronte NON dichiarato chiuso (criteri 1/3/5 aperti) | d011a86 |

## 🔵 Scoperte

1. **La Σ huge di m90 era un cumulativo travestito da retained**: la
   statistica malloc_huge non decrementa mai nei dump (anche dopo
   mi_collect forzato), mentre il trace Rust mostra le sei dealloc — un
   contatore che non sa dire «restituito» trasforma uno spike in una
   riserva fantasma.
2. **Il costo del preludio è per-PROCESSO e per-THREAD**: ogni worker E
   ogni processo phpr CLI ripaga il parse dell'intero preludio stdlib
   (~406 KB di sorgente) in un'arena che tocca 39534144 byte — su hello
   il CLI di parità sta a 44630520 di peak contro 10093048 dell'oracle.
3. **lldb attach è NEGATO in questa sessione macOS** (Not allowed to
   attach) — il canale di naming è stato l'hook env-gated nel
   GlobalAlloc: più fedele (vede le size richieste) e riusabile.
4. **La catena x2+16 identifica bumpalo a occhio nudo**: size_next =
   2*size_prev + 16 (footer di chunk) — firma utile per attribuire huge
   futuri senza backtrace.
5. **A-SK-77 ha morso live per la seconda volta**: il bump del budget a
   24363 ha invalidato la citazione storica in NEXT_SESSION — il gate
   l'ha rifiutata subito (riga emendata al valore in vigore).

## ⭐ Lezioni

1. ⭐⭐ **Un invariante di stat si prova con la coppia alloc/free del
   canale VIVO, mai col solo snapshot**: venti raw concordi dicevano
   «mai liberata»; una singola run con trace su ENTRAMBI i lati l'ha
   refutata in un minuto.
2. ⭐⭐ **Il proprietario di un blocco si nomina dal backtrace, non
   dalla lista dei sospetti**: i tre sospetti nominati dalla direttiva
   (UNIT_CACHE, STUBS, arene Module) erano tutti innocenti — il vero
   nome (arena di parse del preludio) non era nella lista.
3. ⭐ **Una leva si refuta con l'arm A/B sulla STESSA binaria**: env-gate
   + confronto on/off elimina ogni confondente di build; il delta nullo
   è un verdetto, non un fallimento.

## Residui / NON fatto (dichiarati, per NOME)

- **Leva per-file del preludio**: nominata e quantificata, NON attuata
  (tocca l'ordine di hoist ⇒ gate parità completi + ricertificazione
  baseline phpr — sessione dedicata).
- **Attribuzione dello slope ~18.8 MB/worker per NOME** (composizione
  residuo-committed vs retained vive): resta al canale m91 (A-MS-53 →
  A-DL-57/58/60, PT-1) — criterio 1 del fronte PARZIALE.
- **battery61 riproducibile (modo nativo)**: criterio 5, non fatta —
  primo slot del §WP-94.
- **Certificazione battery delle modifiche sorgente** (tutte env-gated
  dormienti): alla prossima build in campagna (battery-91pre).
- Tutto il backlog WP-94 non-A (ancoraggi campo, perimetro root,
  sigilli E1→E3, sanatorie doc, checker D1→D4): invariato, per NOME.
