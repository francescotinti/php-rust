# Macro-step DateTime / date() — design pass & handoff

> **Generato con assistenza AI (Claude Opus 4.8, 1M context).** Documento di
> handoff per una **sessione dedicata**. Il resto del diario (`metrics.md`,
> `04-divergences.md`) si aggiorna come sempre a fine di ogni sotto-step.
> Stato repo all'apertura: **steps 0–33 DONE, 594 test, HEAD su main, clippy
> pulito.** Workflow standard [[legacy-port]] + TDD.

## 0. Perché un macro-step a sé

`date()`/`DateTime` è il primo sotto-sistema che richiede **aritmetica di
calendario civile** (anni bisestili, giorni del mese, giorno-della-settimana,
ISO week) + **parsing di formati data** + una piccola **gerarchia di classi
native con stato** (DateTime mutabile vs DateTimeImmutable). Non è un builtin
puro: serve un design pass prima di scrivere codice. Oracle interamente
disponibile (vedi §1).

## 1. Recon oracle — già fatto (rieseguibile)

Oracle: `/tmp/php-src/sapi/cli/php` (PHP 8.5.7). **Il modulo `date` è compilato**
(a differenza di `mbstring`): `function_exists('date')`, `class_exists('DateTime'|
'DateTimeImmutable'|'DateInterval'|'DateTimeZone')` → tutti `true`.

**Setup test obbligatorio**: `date_default_timezone_set('UTC');` all'inizio di
ogni snippet (altrimenti warning + timezone di sistema non deterministica).
Usare timestamp **fissi** per i differenziali.

### 1a. Tabella completa dei format char di `date()` (ts=1718452845 = Sat 2024-06-15 12:00:45 UTC)

```
d=15      D=Sat     j=15      l=Saturday  N=6   S=th  w=6   z=166  W=24
F=June    m=06      M=Jun     n=6         t=30  L=1   o=2024 Y=2024 y=24
a=pm      A=PM      B=542     g=12        G=12  h=12  H=12   i=00   s=45  u=000000
e=UTC     I=0       O=+0000   P=+00:00    T=UTC Z=0
c=2024-06-15T12:00:45+00:00   r=Sat, 15 Jun 2024 12:00:45 +0000   U=1718452845
```
- Escape: `\` rende letterale il char successivo (`date('\\Y=Y',$t)` → `Y=2024`).
- `S` = suffisso ordinale inglese (st/nd/rd/th). `N`=1..7 (lun..dom), `w`=0..6 (dom..sab).
- `z`=day-of-year 0-based. `t`=giorni nel mese. `L`=leap. `W`/`o`=ISO week/ISO year.
- `B` = Swatch internet time. `u`/`v` = microsecondi/millisecondi (0 senza frazione).

### 1b. Funzioni procedurali (oracle-verificate)

```
mktime(0,0,0,6,15,2024)            = 1718409600   (h,m,s,month,day,year)
strtotime("2024-06-15 12:00:00")   = 1718452800
date("Y-m-d H:i:s", 1718452800)    = "2024-06-15 12:00:00"
```
Altre da coprire: `gmdate`, `gmmktime`, `checkdate`, `date_default_timezone_set/get`.

### 1c. OOP (oracle-verificato — vedi `/tmp/dtrecon.php` riprodotto sotto)

```php
$d = new DateTime("2024-06-15 12:30:45");  $d->format("Y-m-d H:i:s") // 2024-06-15 12:30:45
$d->modify("+1 day");                       $d->format("Y-m-d")       // 2024-06-16
$iv = $d->diff(new DateTime("2024-06-20")); $iv->days; $iv->d; $iv->format("%d days")
$di = new DateTimeImmutable("2024-01-01");  $di->add(new DateInterval("P1M")) // ritorna NUOVO, orig invariato
(new DateTime("2024-06-15"))->getTimestamp()
DateTime::createFromFormat("d/m/Y","15/06/2024")->format("Y-m-d") // 2024-06-15
```
- **DateTime è MUTABILE** (`modify`/`add`/`sub` mutano `$this`); **DateTimeImmutable
  ritorna una nuova istanza**. Distinzione centrale.
- `DateInterval("P1M")` = ISO 8601 duration. `diff()` → DateInterval con `y/m/d/h/i/s/days/invert`.

## 2. Decisioni di design da prendere (per il Decider, all'inizio della sessione)

### D-DT1 — Aritmetica di calendario: crate vs hand-roll
**Raccomandazione: crate `time` 0.3** (Strategy A adapter, [[legacy-port]] — stesso
precedente di `regex` allo step 27). Pure-Rust, niente dipendenza dal tz-db di
sistema, deterministico, ha civil date math + leap/dow/ordinal. Alternativa
`chrono` (più completo su parsing/tz ma più pesante). **NON** hand-rollare
l'aritmetica civile (giorni bisestili, overflow mesi in `modify` → bug garantiti
e fuori dallo spirito "idiomatic, non porting"). La **traduzione dei format char
PHP → output** va però scritta a mano (i format char PHP ≠ quelli di time/chrono).
Lo **strtotime parsing** dei formati relativi va scritto a mano su un subset.

### D-DT2 — Rappresentazione di DateTime nel valutatore
Le classi DateTime hanno **stato nativo** (un istante) + metodi nativi → non sono
prelude-PHP puro (come Exception) né builtin puri. Opzioni:
- **(consigliata)** Classi native intercettate: registra `DateTime`/
  `DateTimeImmutable`/`DateInterval`/`DateTimeZone` come classi note; intercetta
  `new` e le chiamate di metodo nel valutatore (come già fatto per get_class/
  json_decode/preg). Lo stato (timestamp + tz) vive in una proprietà interna
  dell'oggetto (es. una prop nascosta con l'epoch i64, o un campo nativo nel
  `Object`). Riusa la macchina OOP (Zval::Object, dispatch metodi) degli step 19+.
- Valuta se serve un nuovo `Zval` o se basta una prop interna sull'`Object`
  esistente. Preferire la prop interna (meno invasivo).

### D-DT3 — Timezone: scope
**Raccomandazione: solo UTC** (+ eventualmente offset fissi `+HH:MM`) nel primo
giro. Il tz-database completo (America/New_York con DST) è enorme → **scope-out
esplicito**. I test PHP usano quasi sempre `date_default_timezone_set('UTC')`.
`DateTimeZone` ridotto a UTC/offset. Documentare in `04-divergences.md`.

### D-DT4 — strtotime / parsing stringhe data
Il parser di PHP è vastissimo (relative: "next monday", "+1 week", "first day of
next month"; assoluti in mille formati). **Scope a un subset**: ISO
`Y-m-d[ H:i:s]`, `Y/m/d`, timestamp `@N`, `now`, e i relativi più comuni
(`+N day|week|month|year`, `-N ...`). Tutto il resto → scope-out documentato.
`createFromFormat` invece è deterministico (format esplicito) → più facile.

## 3. Scaletta TDD proposta (sotto-step, commit+push ad ognuno)

- **34-1 `date()` core formatting**: dato un timestamp + tz UTC, mappa tutti i
  format char di §1a. Builtin puro `date(string $format, ?int $ts = now)`.
  Attenzione: `now` rende il risultato non-deterministico → per i test passare
  sempre il ts. Helper `format_php(epoch, fmt) -> Vec<u8>`. Include `gmdate`
  (= date in UTC). +molti test (uno per gruppo di char).
- **34-2 `mktime`/`gmmktime`/`checkdate`**: costruzione timestamp da componenti
  + validazione data. Puri.
- **34-3 `strtotime` (subset)**: ISO assoluti + `@N` + `now` + relativi comuni.
  Intercettato o puro (no stato se now passato? now usa l'orologio → vedi D-DT5).
- **34-4 `DateTime` core**: `new DateTime(str)`, `->format()`, `->getTimestamp()`,
  `->setTimestamp()`, `setDate/setTime`. Classe nativa intercettata (D-DT2).
- **34-5 `DateTimeImmutable`** + `modify`/`add`/`sub` (mutabile vs immutabile).
- **34-6 `DateInterval`** (`P1Y2M3DT4H5M6S` parsing, `->format('%...')`) +
  `DateTime::diff()`.
- **34-7 `createFromFormat`** + rifiniture + corpus `ext/date/tests`.

(Granularità rivedibile: il Decider può accorpare 34-4/5 o spezzare 34-1.)

## 4. Gotcha noti (dall'oracle)

- `date('now')` e `new DateTime()` senza arg usano l'orologio reale →
  **non-deterministici**. Per il differential, fissare sempre il ts/la stringa.
  Vedi D-DT5: valutare un hook "ora corrente iniettabile" per testabilità, oppure
  testare solo i path deterministici (consigliato: solo path con ts esplicito).
- **D-DT5 — `now`**: l'orologio reale non è disponibile in modo deterministico nel
  test harness. Opzioni: (a) scope-out di `now`/`time()` dai differenziali, testare
  solo input espliciti; (b) iniettare un "clock" fisso nell'`Evaluator`. Decidere
  all'inizio. NB: `Date.now()`/`SystemTime::now()` esistono in Rust ma rendono i
  test non riproducibili — preferire (a).
- `DateTime` mutabile: `$a = new DateTime(...); $b = $a; $b->modify(...)` muta
  **anche `$a`** (object handle semantics, già corrette dallo step 19).
  `DateTimeImmutable` no.
- `diff()->days` è il totale assoluto di giorni; `->d` è la componente giorni.
  `->invert` = 1 se negativo.
- `createFromFormat` ritorna `false` su parse-fail (non eccezione, di default).
- Overflow in `modify`/`mktime` normalizza (`mktime(0,0,0,13,1,2024)` → gennaio 2025).

## 5. Note test harness

- Builtin puri (`date`/`mktime`/`checkdate`/`gmdate`) → testare in
  `php-builtins/tests/builtins.rs` `out()` (registry COMPLETA). NON disponibili
  in `eval.rs::out()` (registry vuota).
- DateTime OOP (intercettato nel valutatore) → testabile sia in `eval.rs` sia in
  `builtins.rs`; usare `builtins.rs` per le asserzioni con `var_dump`.
- `phpt-runner`: `PHP_ORACLE=/tmp/php-src/sapi/cli/php target/debug/phpt-runner
  /tmp/php-src/ext/date/tests/<...>.phpt` per il corpus. Molti test usano
  `date_default_timezone_set('UTC')` nel `--FILE--`.
- Il binario `phpr` è uno stub (non esegue script) → differential solo via harness.

## 6. Scope-out dichiarati (riassunto, da confermare/espandere nella sessione)

- Timezone database completo + DST (solo UTC/offset fissi).
- `strtotime` formati esotici/locale; `IntlDateFormatter`; calendari non gregoriani.
- `now`/`time()` deterministici (vedi D-DT5).
- `DatePeriod`, `DateTimeZone` avanzato, microsecondi reali da clock.
- Formattazione locale-dependent (mese/giorno localizzati): PHP `date()` è
  comunque inglese-fisso per `D/l/F/M`, quindi OK.

## 7. Primo comando della sessione

1. Leggere questo file + `diary/metrics.md` (coda) + il resume di memoria.
2. Decidere D-DT1..D-DT5 col Decider (proporre i default consigliati sopra).
3. Aggiungere il crate scelto a `crates/php-runtime/Cargo.toml` (o php-builtins
   per le funzioni pure) — `cargo add time` / `chrono`.
4. Partire da **34-1 `date()` core** in TDD (RED→GREEN per gruppo di format char).
