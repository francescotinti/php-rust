# Step 35 — API procedurale date — design pass & handoff

> **Generato con assistenza AI (Claude Opus 4.8, 1M context).** Documento di
> handoff per una **sessione dedicata** (deciso con l'utente). Stato repo
> all'apertura: **steps 0–34 DONE, 624 test, HEAD su main, clippy pulito.**
> Workflow standard [[legacy-port]] + TDD. Oracle PHP 8.5.7 in
> `/tmp/php-src/sapi/cli/php`.

## 0. Cos'è e perché

Lo step 34 ha costruito l'API **OOP** di data/ora (`DateTime`,
`DateTimeImmutable`, `DateInterval` come classi del prelude + builtin puri
`date`/`mktime`/`strtotime`/…). PHP espone anche l'**API procedurale**
equivalente (`date_create`, `date_format`, `date_diff`, `date_add`, …), che è in
larghissima parte **zucchero sintattico** sopra l'OOP. Implementarla:
1. sblocca molti test del corpus `ext/date/tests` che usano lo stile procedurale;
2. è a basso rischio (wrapper sottili su metodi già verificati);
3. richiede **un solo pezzo di infrastruttura nuovo**: funzioni globali nel
   prelude (vedi D-PD1).

## 1. Recon oracle — già fatto (rieseguibile, `/tmp/dtproc.php`)

Tutto con `date_default_timezone_set('UTC')`. Valori verificati:

```
date_create('2024-06-15 12:30:45')            → DateTime ; date_format(...,'Y-m-d H:i:s') = 2024-06-15 12:30:45
date_create_immutable('2024-01-01')           → DateTimeImmutable
date_diff(date_create('…06-15'),date_create('…06-20'))  → ->days=5 ; date_interval_format(iv,'%d days')='5 days'
date_add($a, date_interval_create_from_date_string('1 day'))   → +1 giorno (muta $a, ritorna $a)
date_sub($a, new DateInterval('P1M'))         → -1 mese
date_time_set($a,8,30,0); date_date_set($a,2020,2,29)          → 2020-02-29 08:30:00
date_timestamp_set($a,1718452845); date_timestamp_get($a)      → 1718452845
date_modify($a,'+2 hours')                    → +2h
date_create_from_format('!d/m/Y','15/06/2024')→ DateTime 2024-06-15 00:00:00
getdate(1718452845)   → [seconds=45,minutes=0,hours=12,mday=15,wday=6,mon=6,year=2024,yday=166,
                          weekday='Saturday',month='June', 0=1718452845]
localtime(1718452845) → [45,0,12,15,5,124,6,166,0]   (sec,min,hour,mday,mon0,year-1900,wday,yday,isdst)
localtime(1718452845,true) → assoc keys tm_sec/tm_min/tm_hour/tm_mday/tm_mon(0)/tm_year(-1900)/tm_wday/tm_yday/tm_isdst
```
Note: `mktime`/`gmmktime`/`checkdate`/`strtotime`/`date_default_timezone_set|get`
**già fatti** allo step 34. `date_interval_create_from_date_string('1 day')`
parsa una stringa **relativa** (non ISO) → componenti interval.

## 2. Decisioni di design (per il Decider, a inizio sessione)

### D-PD1 — Dove vivono i wrapper procedurali OOP — **infra chiave**
I wrapper (`date_create`, `date_format`, `date_diff`, `date_add`, …) chiamano
`new`/metodi → **un builtin puro NON può** (ha solo `Ctx`, non l'evaluator).
**Raccomandazione: funzioni globali del PRELUDE** (PHP puro in `PRELUDE_SRC`),
coerente con la scelta dello step 34 (classi nel prelude). Es.:
```php
function date_create($datetime = "now") { return new DateTime($datetime); }
function date_format($object, $format)   { return $object->format($format); }
function date_diff($base, $target, $absolute = false) { return $base->diff($target); }
function date_add($object, $interval)    { return $object->add($interval); }
// … ecc.
```
**Infra da costruire**: oggi `lower_prelude()` (lower.rs:328) ritorna SOLO
`classes` + `class_index`; **funzioni/closure/static vengono scartate** (commento
esplicito riga 326-327). Va esteso per:
1. far girare anche `hoist_function`/lowering delle funzioni del prelude;
2. ritornare `functions: Vec<FnDecl>` + `fn_index` del prelude;
3. **mergiarle** in testa a `Program.functions` con lo stesso accorgimento di
   `hoist_classes` per le classi (offset degli indici: le funzioni si risolvono
   per **nome→indice** via `fn_index`, vedi lower.rs:794-796 — verificare che le
   chiamate-per-indice interne restino coerenti dopo il merge, esattamente come
   gli id di classe del prelude vengono offsettati).
- **Alternativa scartata**: intercettare ogni funzione nell'evaluator (come
  `json_decode`/`get_class`). Più invasiva, ~14 funzioni, meno idiomatica del
  prelude PHP. Usarla solo se il merge delle funzioni-prelude si rivelasse troppo
  costoso (improbabile).

### D-PD2 — getdate / localtime
**Builtin PURI** in `php-builtins/src/date.rs` (non toccano oggetti, ritornano
array). Riusano `decompose`/gli accessor `time`. `getdate` = array assoc + chiave
`0`=ts; `localtime` = array indicizzato stile C `struct tm` (mon 0-based,
year-1900), con `$associative=true` → chiavi `tm_*`.

### D-PD3 — date_interval_create_from_date_string
Parsa una stringa **relativa** ("1 day", "2 weeks 3 hours", "1 year") →
componenti DateInterval. Aggiungere un helper Rust `__interval_from_date_string`
(sottoinsieme: riusa la logica di `parse_relative` dello step 34-3, ma **senza
base** e producendo l'array componenti `y/m/d/h/i/s` invece di un epoch).
La funzione-prelude costruisce un `DateInterval` e ne imposta le prop (come fa
`DateTime::diff`: `$iv = new DateInterval('PT0S'); $iv->d = …;`).

### D-PD4 — date_diff $absolute
Terzo parametro `$absolute` (default false): se true, l'intervallo ha `invert=0`.
Onorarlo nel wrapper (`if ($absolute) $r->invert = 0;`). Minore.

## 3. Scaletta TDD proposta (sotto-step, commit+push ad ognuno)

- **35-1 INFRA + primi wrapper**: estendere il prelude alle funzioni globali
  (D-PD1) e provarlo con `date_create`/`date_create_immutable`/`date_format`/
  `date_timestamp_get`. È il sotto-step di rischio: una volta verde, il resto è
  meccanico. Test in `php-builtins/tests/builtins.rs` `out()` (registry COMPLETA).
- **35-2 mutatori/diff**: `date_diff` (+`$absolute`), `date_add`, `date_sub`,
  `date_modify`, `date_date_set`, `date_time_set`, `date_timestamp_set`.
- **35-3 format/interval**: `date_create_from_format`,
  `date_create_immutable_from_format`, `date_interval_format`,
  `date_interval_create_from_date_string` (+ helper `__interval_from_date_string`).
- **35-4 getdate/localtime**: builtin puri (D-PD2).
- **35-5 corpus + rifiniture**: ri-girare `ext/date/tests` (ora molti test
  procedurali diventano runnable) — contare i nuovi pass, classificare i fail
  come scope-out. Aggiornare `diary/metrics.md` + `04-divergences.md`.

(Granularità rivedibile dal Decider: 35-2 e 35-3 si possono accorpare.)

## 4. Gotcha noti

- **Test harness**: i wrapper-prelude chiamano builtin (`date`/`mktime`) →
  testabili solo con la **registry completa** (`php-builtins/tests/builtins.rs`
  `out()`), non in `eval.rs::out()` (registry vuota). Come step 34.
- **Redefinizione**: PHP darebbe fatal se l'utente ridefinisce `date_create`;
  raro, ignorare (il prelude vince/coesiste — verificare il comportamento del
  merge sul conflitto nome).
- `date_add`/`date_sub` su `DateTime` **mutano** e ritornano l'oggetto; su
  `DateTimeImmutable` ritornano una **nuova** istanza — i metodi `add`/`sub`
  dello step 34 già lo fanno, quindi i wrapper sono `return $o->add($i);` punto.
- `getdate()`/`localtime()` senza argomento usano `time()` (non-det, D-DT5) →
  testare solo con ts esplicito.

## 5. Scope-out dichiarati (da confermare/espandere)

- `date_sun_info`/`date_sunrise`/`date_sunset` (astronomia).
- Funzioni timezone procedurali (`timezone_open`, `date_timezone_get/set`,
  `timezone_*`) → dipendono da `DateTimeZone`/tz-db (scope-out step 34, D-DT3).
- `date_get_last_errors`, `date_parse`, `date_parse_from_format` (ritornano array
  diagnostici dettagliati del parser nativo).
- `strftime`/`gmstrftime` (deprecate in PHP 8.1).
- `date_interval_create_from_date_string` su stringhe relative esotiche
  (subset come strtotime, D-DT4).

## 6. Primo comando della sessione

1. Leggere questo file + la coda di `diary/metrics.md` (sezione Macro-step 34) +
   il resume di memoria `php-rust-next-step7.md`.
2. Decidere D-PD1..D-PD4 col Decider (default consigliati sopra).
3. Partire da **35-1**: estendere `lower_prelude` alle funzioni globali (modello
   = `hoist_classes` per l'offset prelude) in TDD (RED con `date_create`).
