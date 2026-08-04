# Verbale sedia 7 — Leijen (allocatore, footprint, layout) — Concilio WP-99

**Oggetto**: report S-97.1 (H-A1 caduta, 7 varianti dormienti dietro `PHPR_REG_LOWER`) e programma H-B1.
**VERDETTO: CONCORDO CON EMENDAMENTI.**

## 1. La premessa della domanda è REFUTATA dall'evidenza (a favore del report)

Il sospetto «`with_capacity(n)` senza `shrink_to_fit` ⇒ capacity ritenuta per unità» è
FALSO per ordinamento verificato: `lower_func` gira dentro `compile_body`
(`compile/func.rs:163`), mentre il funnel WP-48 `m.shrink()` gira a valle, a fine
`compile_program` (`compile/mod.rs:451`), e `Func::shrink`
(`bytecode.rs:1495-1497`) fa `ops.shrink_to_fit()` + `lines.shrink_to_fit()`.
L'eccesso (n−m)×48B ops + (n−m)×taglia`Line` è **transiente compile-side**, non
ritenuto. Il parallelismo `lines`/`ops` (una linea per finestra, indicizzata alla
testa) è preservato. Due caveat pinnati, non refutazioni: (a) `Module::shrink`
salta i `Func` Rc-condivisi (`Rc::get_mut`) — coperto dal commento «shrunk dal
modulo proprietario», ma un chiamante FUTURO di `lower_func` fuori dal funnel di
`compile_program` riterrebbe lo slack in silenzio; (b) sotto mimalloc lo
shrink recupera a granularità di size-class, già prezzato in WP-48.

## 2. Flag-off: zero byte di dati, ma «zero-delta» va PERIMETRATO

Le 7 varianti non allargano `Op`: pin **doppio e indipendente** verificato
(`==48` in `reg_lower.rs:571`; `≤48` + `Frame ≤176` in `vm/mod.rs:22051`).
Discriminant invariato — ma il census è a 185 righe: +7 porta l'enum a ~192
varianti, e il confine 256 (discriminant u8) ha ormai headroom ~60 senza alcun
dente che lo guardi. Il costo flag-off residuo è **codice**: 7 corpi handler nel
run_loop (lezione WP-38: branch mai-preso = +2,9%; caveat WP-98: non è tariffa).
Il tempo è coerente (7,83 vs 7,88 di ha2) — ma «flag-off zero-delta» è provato
per TEMPO e PARITÀ, **non per footprint**: taglia binario/icache non riportate,
e il footprint non è misurato da m90.

## 3. REFUTAZIONE CAPITALE: −42% di opcode ≠ −42% di byte

Il 19→11 è un conteggio di **dispatch dinamici sul micro `arith`**; i byte di
`ops` Vec sono **statici** e la frazione di finestre fondibili su codice
WordPress generico è ignota (i vincoli: sorgenti LoadVar/PushConst, stessa
linea, nessun target nel mezzo — su codice reale morde molto meno che su un
loop aritmetico). Chi presentasse lo stream più corto come leva di footprint
commette un errore di categoria. Plafone: anche azzerando TUTTE le `ops` Vec
ritenute si recupera solo la massa che la mappa WP-58/59 prezza — contro un
peak di 1901,11 MiB una promozione flag-on deve reggersi **sulla CPU
soltanto**; il guadagno footprint va misurato, mai dedotto dal census.

## 4. Il rischio che il mio perimetro deve nominare

La roadmap footprint è FERMA e non misurata da m90 — ma l'emissione **flag-off è
già cambiata** (H-A2, Sweep eliso, S-97.0) e il binario è cresciuto di 7
handler, senza alcuna coppia peak. La deriva si accumula non misurata. H-B1 è
attesa footprint-neutrale (niente layout, niente allocazioni): se il peak
cambiasse, è un segnale non richiesto da investigare, non da festeggiare.

## Emendamenti

- **A-LE-99-1**: dente sul discriminant: assert `N_OPS < 256` (census già
  esporta `N_OPS`) accanto al pin 48B — il confine u8 non si attraversa in silenzio.
- **A-LE-99-2**: al prossimo collaudo di parità WordPress (dovuto: l'emissione è
  cambiata), acquisire la coppia peak `/usr/bin/time -l` **nello stesso run** —
  zero run aggiuntivi, m90 si aggiorna gratis.
- **A-LE-99-3**: una riga di commento in `lower_func`: «lo slack di
  `with_capacity(n)` è recuperato dal funnel WP-48 a valle
  (`compile/mod.rs:451`); un chiamante fuori da `compile_program` lo ritiene».

## Kill-switch

- **KS-LE-99-1**: promozione di `PHPR_REG_LOWER` a baseline SENZA coppia peak
  stessa-sera (`/usr/bin/time -l`, entrambe le gambe) ⇒ **respinta**.
- **KS-LE-99-2**: se un pin 48B scatta o `N_OPS` ≥ 256 ⇒ stop e ridisegno del
  layout PRIMA di qualsiasi misura CPU.

## Refutazioni capitali

Sì, una (§3): «lo stream −42% è un guadagno di footprint» — conteggio dinamico
di micro spacciato per byte statici di carico reale; qualunque claim futuro in
quella forma è respinto in assenza di misura. (La premessa «shrink assente» del
mio stesso mandato è invece refutata dall'evidenza, §1: il report non ha il
difetto ipotizzato.)
