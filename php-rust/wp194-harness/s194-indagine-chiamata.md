# S-194 — indagine sul costo di una chiamata a builtin (registro dei costi). Generato con assistenza AI (Claude Fable 5.2).
Fase I del piano D-bis: nessuna modifica al codice di produzione. Braccio con la ricetta del pin sha `8193209ab9c247ba` (tree = `6d2a0e6d`). Fonti: sonda `s194-primitive-I1.out` (istruzioni ritirate per iterazione, deterministica), profili `s194-profilo-builtin_{strlen,abs,count}-D.out`, chiamanti `s194-chiamanti-builtin_{strlen,abs,count}.out` (campioni = direzione; 1 % ≈ 9,2 / 7,6 / 8,1 istruzioni), bytecode `out/indagine-ops-*.txt`.

## 1. Quanto costa la sola chiamata (per differenza, istruzioni nette per giro)
| Prova | phpr | di cui la chiamata | oracle | di cui la chiamata |
|---|---|---|---|---|
| `$s += $i` (riferimento) | 234 | — | 46 | — |
| `$s += abs($i)` | 558 | ~324 | 160 | ~114 |
| `$s += abs(-5)` | 532 | ~298 | 158 | ~112 |
| `$s += count($h)` | 609 | ~375 | 71 | ~25 (istruzione dedicata) |
| `$s += strlen($u)` | 718 | ~484 | 64 | ~18 (istruzione dedicata) |
| `$s += strlen($u) + strlen($w)` | 1389 | 2ª chiamata + somma = 671 | 107 | 43 |
| `$s += intdiv($i, 3)` | 742 | ~508 (2º argomento: +184 su `abs`) | 184 | ~138 |
| `$f = pi()` (0 argomenti) | 397 | ~190 più di `$x = 1` (207) | 94 | ~75 |
| `$s += (int)str_contains($u, "l")` | 1592 | — | 236 | — |
Lettura: una chiamata vera a builtin costa oggi **~300 istruzioni a vuoto + ~180 per argomento**; l'oracle ~75 + ~30. Per `strlen`/`count` l'oracle non chiama: ha un'istruzione dedicata (il confronto onesto per «una chiamata» è `abs`/`intdiv`/`pi`).

## 2. Le quattro voci, misurate
**(1) Tre livelli di funzione** — `abs`: `value_builtin_call` 10,0 % ≈ 76 istr. + `run_value_builtin` 16,9 % ≈ 128 istr.; `strlen`: 100 + 130; `count`: 83 + 133. Dentro `value_builtin_call` (run:1402): prologo/epilogo con il `Result<Zval,PhpError>` passato in memoria (righe 1403/0/1487 ≈ 60 % della funzione), sei test di flag, `cur_line` calcolata SEMPRE (frame + tabella righe) anche se serve solo a `flush_diags`. Dentro `run_value_builtin` (mod:6062): due `mem::take` di `HashMap` std (righe 6086/6089, ~5 %), costruzione del `Ctx` e chiamata (6091, ~3-4 %), epilogo con i drop di due mappe e due `Vec` (6108, ~2,5 %). **Ipotesi verificata**: `libdyld` (0,9 % per `abs`/`count`, 1,7 % per `strlen`) è chiamato SOLO da `value_builtin_call` alle righe della `HashMap` (1481, 1486) = accesso thread-local di `RandomState::new()` a ogni mappa vuota creata: 7-16 istruzioni per chiamata buttate.
**(2) Controllo `__toString`** (solo builtin con flag di coercizione; `strlen`): `compute_stringify` 4,0 % + `any_object` 3,4 % ≈ 68 istr. per restituire una mappa vuota. **Voce gemella scoperta su `count`**: `as_countable` fa `deref_clone` dell'argomento (clone dell'array + drop, righe 3911→1435: 2,0 % + 2,9 % ≈ 40 istr.) per scoprire che non è un oggetto.
**(3) Copia e distruzione dell'argomento** — `LoadVar` clona sulla pila (1,6-2,5 %), il gestore sfila (3910: 1,8-2,4 %) e dopo la chiamata distrugge (3912: 2,2-2,8 %); il risultato viene spinto (3936) e risfilato dalla somma con un altro drop (2158: 1,2-1,4 %). In più, per i builtin di stringa: `Ctx::to_zstr` + `convert::to_zstr` 5,8 % ≈ 53 istr. per clonare una stringa di cui `strlen` legge solo la lunghezza. Totale ≈ 90 istr. per argomento intero, ≈ 140 per argomento stringa. Misura indipendente: il 2º argomento di `intdiv` costa +184 (comprende il suo `PushConst`).
**(4) Il ciclo** — `run_loop` + `to_bool` = 50-60 % dei campioni: `match op` 8-11 %, `func.ops[ip]` 3,3 %, guardia di profondità 2,5-3,7 %, `top` 2 %, back-edge `CmpJmpSS` con `binary_fast` + `to_bool` ≈ 8 %, `Sweep` 1,3-2,1 %. Sono le 200 istruzioni del ciclo vuoto (oracle 35) più il dispatch dei 3 op del corpo.

## 3. Chi toglie cosa
| Voce | Istruzioni (abs / count / strlen) | La toglie |
|---|---|---|
| livello `value_builtin_call` sul cammino comune | ~70 / ~75 / ~90 | **D-bis** (B3) |
| `mem::take` + drop delle mappe, `cur_line` a vuoto, TLS | ~80 / ~85 / ~95 | **D-bis** (B1, B3) |
| `compute_stringify` senza oggetti | — / — / ~62 | **D-bis** (B2) |
| `as_countable` con clone | — / ~40 / — | **D-bis** (B2) |
| copia/sfila/distruggi dell'argomento e del risultato | ~90 per argomento | pacchetto **B** (operandi dentro l'op) |
| clone della stringa nel builtin | ~53 (stringhe) | apertura per nome «contratto di prestito dei builtin» |
| dispatch e back-edge | ~200 + 3 op | pacchetto **A** |
| `strlen`/`count` come istruzione dedicata | il resto fino all'oracle | **scelta dell'utente** (oggi esclusa dalla regola «niente istruzioni nuove») |
Attesa del D-bis da questo registro: `abs` −150, `count` −200, `strlen` −250, `substr` −250 (stessa catena di `strlen`).

## 4. Trovato per strada (non toccato)
- **Divergenza dall'oracle**: `ord("hello")` in PHP 8.5 emette un avviso di deprecazione (stringa non di un byte); phpr no (fixture `builtin_ord` NULLA, tolta dalla sonda). Apertura per nome.
- `str_contains` a 1592 istruzioni (oracle 236): due coercizioni a stringa con clone + precalcolo `__toString`; rientra nelle voci (2) e (3).
- Il cammino dinamico (`invoke_named`: `$f($x)`, `call_user_func`, callback di `array_map`) chiama `run_value_builtin` direttamente con lo stesso precalcolo: nessuna fixture lo misura oggi.
- **Altre due divergenze preesistenti dall'oracle** (uguali sul pin s185, viste dalle prove di comportamento del D-bis): `strlen(null)` senza l'avviso di deprecazione «Passing null to parameter»; `print_r` di un oggetto ignora `__debugInfo()`. Aperture per nome.

## 5. Consuntivo dopo il D-bis (`s194-primitive-Dbis.out`, istruzioni nette; stima del §3 → misurato)
| Prova | D | D-bis | delta misurato | stima §3 | oracle |
|---|---|---|---|---|---|
| `abs($i)` | 558 | 457 | −101 | −150 | 160 |
| `count($h)` | 609 | 454 | −155 | −200 | 71 |
| `strlen($u)` | 718 | 501 | −217 | −250 | 64 |
| `substr($w, -30)` | 1368 | 1118 | −250 | −250 | 145 |
| `pi()` | 397 | 290 | −107 | — | 94 |
| `intdiv($i, 3)` | 742 | 639 | −103 | — | 184 |
Per passo: B1 −33 · B2 −97 (stringhe) / −40 (`count`) · B3 −67/−88. La chiamata vera più semplice (`abs`) è ora a **2,85×** l'oracle (era 3,50×; prima di D 5,3×). Catena di chiamata residua ≈ 100 istruzioni in una sola funzione; il resto è ciclo e pila (pacchetti A e B).

## 6. Dopo le istruzioni dedicate (deroga utente 21/11; `s194-primitive-ded.out`)
| Prova | prima di D | D | D-bis | dedicate | oracle | rapporto |
|---|---|---|---|---|---|---|
| `strlen($u)` | 1017 | 718 | 501 | **315** | 65 | 4,8× |
| `count($h)` | 894 | 609 | 454 | **317** | 71 | 4,4× |
| `strlen($u) + strlen($w)` | — | 1389 | 953 | **583** | 108 | 5,4× |
L'op dedicato in sé costa ~80 istruzioni (315 − 234 di `$s += $i`); quelle 234 sono carico + somma, e sotto ci sono le 200 del ciclo vuoto.
- **Divergenze preesistenti del pin dall'oracle viste dalle prove `s194-dedicate-comportamento.php`** (non toccate, aperture per nome): `strlen(null)` senza avviso di deprecazione · `strlen` di un oggetto senza `__toString` dà un avviso invece del `TypeError` · sotto `declare(strict_types=1)` `strlen(5)` e `strlen(null)` non lanciano `TypeError` · **in questo file gli avvisi «Undefined variable» non escono affatto** (`strlen($undef)`, `count($undef)`, `echo $undef`), mentre in `s194-dbis-comportamento.php` uno esce: da istruire.
