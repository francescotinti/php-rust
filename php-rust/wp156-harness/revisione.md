# Revisione S-156 (revisore singolo, lente SEMANTICA)

## VERDETTO: REGGE CON RILIEVI — il claim di invarianza semantica è confermato al sorgente su tutte le piste; nessun rilievo lo invalida.

## Rilievi
1. **Ordine argomenti: CONFERMATO.** `for slot in buf[..n].iter_mut().rev() { pop }` (run.rs:3676) riempie slot n-1←top … slot 0←primo: identico a `split_off(len−n)` di `pop_keys` (mod.rs:8210). Un'inversione sarebbe stata vista: fx-ce righe 14/18 (`class_exists($n,false)` con autoloader armato → "AL:" e var_dump divergenti) e la parità HA-OK 20000000 del giudice stesso (`class_exists(false,$cn)` darebbe 10M).
2. **Fallback: CONFERMATO.** Ricostruzione Vec via `mem::replace` in ordine 0..n, valori identici; argc=0 degenere corretto (loop vuoto, Vec vuoto). Minore: il fallback paga DUE match slice persi (run.rs:3680 + delegazione mod.rs:14665), il commento ne dichiara uno — solo costo, non semantica.
3. **Delegazione: CONFERMATA.** Spread (run.rs:3640) e callable dinamici (calls.rs:1113) passano dal dispatcher Vec che delega alla slice (mod.rs:14665): stessi corpi per i 6 nomi da ogni chiamante.
4. **Corpi: CONFERMATI solo-lettura.** Diff 4929916 su host.rs = 8+/8−, pura firma `Vec<Zval>`→`&[Zval]` + `&args`→`args`; i corpi usavano già `first()/get()/deref_clone()`, nessun `into_iter`/consumo.
5. **Visibilità: contratto INVARIATO A==B** (in entrambi i rami gli args escono dal frame prima del dispatch; né Vec né buf erano root GC). MA il contratto storico stesso è una potenziale divergenza vs Zend: in PHP `debug_backtrace()` dentro l'autoloader innescato da `class_exists` mostra il frame di class_exists con args. Pre-esistente, NON introdotta da HD2 — da sondare e catalogare (rilievo, non invalidazione).
6. **Dente loc: CONFERMATO.** loc_dente.rs è test-only (crates/php-runtime/tests/), il pin resta 42efea3e34feb390 AL BYTE prima e dopo la salita (t1 riga 2, promo righe 2/5); la riga 8 ammette la salita «dichiarata a verbale» — fatta (commit 8815db1, commenti +30/+29 nell'allowlist).
7. **Bande al limite (fuori lente ma a verbale):** guardia backtrace D=−8,3 ESATTAMENTE alla soglia −8,3 (margine zero sull'unico nome convertito pesante); riconciliazione smoke↔R5 FUORI BANDA (4,5>4,0) e conferma post-pin +5,0 vs attesa ~+16 (drift-tree non quantificato).

## Azioni
1. S-157: ri-misurare la guardia backtrace con comparatore (> vs ≥) pre-registrato nel criterio; margine zero non deve ripetersi.
2. Arbitrare a verbale il fuori-banda smoke↔R5 e quantificare il drift-tree prima di scrivere +16,0 in PERF_MAP come cifra di leva.
3. Sonda una tantum oracle↔pin: `debug_backtrace()` dentro autoloader innescato da `class_exists` — catalogare o escludere la divergenza in PHPR_DIVERGENCES_FROM_PHP.md.
4. Prossime tranche slice: per ogni nome convertito a 2+ argomenti, un caso fixture con ruoli distinti (modello fx-ce riga 14/18).
5. Correggere il commento del braccio: il fallback perde due match, non uno.
