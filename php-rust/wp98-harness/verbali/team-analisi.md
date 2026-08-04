# Team ANALISI — nota di riconciliazione (fase 2, Concilio WP-98)

Sedie: Hoare (v1), Matsakis (v2), Stogov (v8). I verbali individuali restano la
fonte VINCOLANTE; qui si riconcilia o si registra il dissenso.

## 1. Convergenze — buchi di soundness ANCORA aperti dopo il fix di S-96.0

Elenco unificato, senza duplicati. `M` = provato a macchina, `A` = argomentato.

| # | Buco | Sedia | Prova |
|---|---|---|---|
| B1 | Invariante non presidiata: un op con `defs` **e** `edges` insieme farebbe ricadere il kill su tutti gli archi normali (classe A-TH-97-1 riaperta). Serve `debug_assert!` | Hoare (A-TH-98-1) | A (verificato che oggi nessun op lo ha ⇒ latente) |
| B2 | Il loop che allarga `nbits` incatena `uses`/`defs`/`fall_defs` ma **non** gli edefs per-arco (`CatchMatch::var`): `Bits::clear` panica invece di degradare | Hoare (A-TH-98-1) | A |
| B3 | Esaustività sulle VARIANTI, non sui CAMPI (`Op::X { .. }`): un campo `Slot` nuovo compila muto | Hoare (A-TH-98-2) | M parziale — 3 varianti campionate, nessuna porta `Slot` ⇒ l'elenco regge, la cura no |
| B4 | `renounce()` ha `nbits` più stretta di `analyze` (si allarga solo su `LoadSlot`/`LoadVar`): `mark` scarta, `Bits::get` risponde «non rinunciato». Direzione: F2 **meno** conservativa | Hoare (A-TH-98-3) | A |
| B5 | Arco di ri-lancio di `EndFinally` verso handler **esterno** non modellato (`edges` = `after` + soli `ParkJump`) | Hoare (A-TH-98-4) | A, nessuna fixture |
| B6 | **Canale cross-frame**: `current_frame_args` (mod.rs:10493) legge gli slot vivi di OGNI frame; `renounce()` scorre solo `func.ops`, quindi un osservatore nella *callee* non fa scattare `observes_scope` | Matsakis (RC-MS-98-1) e Stogov (A-DS-98-3), **indipendentemente** | A (puntatori esatti: host.rs:4343, mod.rs:13288) |
| B7 | Lo stesso canale non è chiudibile **per forma** da una lista di nomi: `getTrace()` di qualunque Throwable costruito più in basso, handler d'errore/eccezione, tick, `ob_start`, `usort`, `spl_autoload`, `ReflectionFiber::getTrace()` | Stogov (A-DS-98-3) | A |
| B8 | `CallNsFallback`: `observes_scope` interroga `name` e ignora `fallback` — `namespace X; extract($a);` esegue `ho_extract` | Stogov (A-DS-98-1) | A, fixture obbligatoria |
| B9 | Nome risolto a runtime non name-checked (`CallValue`, `CallValueArgs`, `CallNamed`, `CallSpread`, `MakeFcc`): `$f='extract'; $f($a);` | Stogov (A-DS-98-2) | A |
| B10 | `$http_response_header`: builtin che **scrive** lo scope del chiamante, in nessuna lista | Stogov (A-DS-98-4) | da verificare |
| B11 | `would_take_safe_str` è fedele all'**output**, non alla memoria: il take sposta il momento della COW, osservabile da `memory_get_usage`/`_peak`/`debug_zval_dump` | Stogov (A-DS-98-5) | A |

Nessun buco è oggi provato da una fixture che morde. B6/B7 sono lo stesso buco
trovato da due sedie per vie diverse: è la convergenza più forte del team.

## 2. Il delta sospetto su `slot_reads_rc` — DECISIVO SUL GRADO

Fatto ricontrollato dal relatore sul raw: F1 dà `slot_reads=60598107`,
`slot_reads_rc=53561199`; F2 **dichiara** «riprodotti IDENTICI al run F1»; il
riconteggio dà 60598093 / 53561185 ⇒ **−14 su entrambi**. Il determinismo era
stato stabilito due volte e ora è rotto: la premessa di fatto di Hoare (R1) è
confermata, e `grade=VERDICT` non è difendibile su un run non riprodotto.
La premessa di *meccanismo* («nessuna modifica del changeset può toccare
`note_slot_read`») resta però ARGOMENTATA: la verifica a macchina non è stata
possibile (Serena indisponibile in fase 2).

Corroborazione: le altre due sedie **non affrontano il punto** — non lo
corroborano né lo contraddicono. Convergono però sulla stessa conseguenza
pratica per altre vie: Matsakis (RC-MS-98-2, la banda Str è un maggiorante,
numeratore di conteggio × tasso di costo medio) e Stogov (il perimetro è un
TETTO; KS-DS-98-1). **Le bande cadono per tre strade indipendenti**; solo la
prima colpisce il *grado* del raw.

Contro-nota a verbale (relatore, non appianabile in favore di nessuno):
`would_take`, `would_take_rc` e `sites_movable` hanno delta **esattamente 0**.
Un rumore d'esecuzione generico li avrebbe mossi. Il −14 è quindi confinato a
letture in siti non-movibili: o una sorgente di non-determinismo che tocca solo
quella classe, o un canale del changeset non ancora nominato. In nessuno dei due
casi −21/−18/−6 sono attribuibili ad A-SK-97-1. Sopravvive solo
`delta_would_take = 0`.

## 3. Conflitti — posizioni non appianate

**(a) Verdetto del passo 2 (Str-first vs piano B).** Esito concorde («non
vince»), motivazione in disaccordo.
- *Hoare* (R2): la formula scritta è sbagliata di segno. 0,84–1,21% lordo contro
  +2,9% di pedaggio non è «non distinguibile da zero», è **nettamente negativo**
  nella forma a braccio nuovo; **ignoto** nella forma per-braccio, mai valutata
  (§5.1). La rinuncia a `nm -S` è circolare.
- *Stogov*: il verdetto **sopravvive** ma non per le ragioni scritte — essendo
  `would_take_safe_str` un TETTO, il perimetro vero è ≤ e la strada lunga esce
  *ancora più debole*. Nessuna banda citabile finché A-DS-98-1/2/3 sono aperti.
- *Matsakis*: **esito confermato, motivazione refutata** — la banda è un
  maggiorante perché `Str` è la variante col drop più economico, sotto la sua
  quota di conteggio.

**(b) Forma della leva.** *Matsakis* (A-MS-98-3) dichiara il flag dentro
`LoadSlot` ownership-**equivalente** a `TakeSlot` (cambia solo il lato costo) e
chiede che l'analisi stia nel COMPILATORE, come funzione pura degli ops, per via
della unit cache TL (WP-81). *Hoare* (KS-TH-98-3) e *Stogov* (KS-DS-98-4) dicono
che **qualunque leva che aggiunga un braccio prima di O1 è stop**. Tensione
reale: la forma-flag che Matsakis lascia aperta è precisamente quella che
aggiunge un braccio nel corpo caldo. Registrata, non risolta.

**(c) `would_take_safe_ref`.** Il raw lo legge come «buco minuscolo ⇒ guard poco
costoso». *Matsakis* lo refuta: misura solo il canale visibile nel **tag della
cella**, è un limite inferiore di un canale enumerato, e KS-MS-98-2 impone di
ritirarlo, non difenderlo. *Hoare* e *Stogov* non lo toccano: la refutazione ha
una sola firma.

**(d) Priorità.** Stogov mette O1 (241,7 KiB in una funzione contro ~192 KiB di
L1i) sopra ogni perimetro; Hoare concorda (voce 1 §WP-97 solo INSIEME a O1);
Matsakis non si pronuncia.

## 4. Che cosa va rifatto prima di qualunque emissione (lista minima, in ordine)

1. **Ritirare `grade=VERDICT`** dal riconteggio → SCREEN, e ri-eseguire R≥2 a
   binario invariato finché il −14 non ha una causa nominata (o è registrato
   come sorgente di rumore). Nessuna cifra del raw si cita prima.
2. **Fixture negative che mordono sul binario pre-leva** per B8 e B9
   (`CallNsFallback.fallback`; `$f='extract'; $f($a);`). Se mordono, F1/F2 di
   S-95.0 e S-96.0 decadono a TETTO (KS-DS-98-1) e ogni banda si ritira.
3. **Chiudere il canale ARGOMENTI/cross-frame** (B6+B7): escludere gli slot
   `< n_params` da `movable_safe` e **ri-contare** (A-MS-98-1) — il delta È la
   taglia del canale invisibile. Trappola nel gate: callee che chiama
   `debug_backtrace()` e legge un parametro del chiamante, più un'eccezione con
   trace, più `ReflectionFiber::getTrace()` (KS-MS-98-1).
4. **Riparare i tre difetti interni**: `nbits` di `renounce()` = `nbits` di
   `analyze` + assert (B4); allargamento di `nbits` anche sugli edefs per-arco
   (B2); `debug_assert` dell'invariante `defs` ⊥ `edges` (B1).
5. **Fixture per l'arco di ri-lancio di `EndFinally`** verso handler esterno
   (B5) — stessa classe che è già costata una sessione.
6. Solo allora: ri-contare, riscrivere il netto come `gain(str) − guard(safe)`
   (A-MS-98-2), etichettare `would_take_safe_str` come fedele-all'output e
   portare la COW/`memory_get_usage` nel gate per NOME (B11, A-DS-98-5).
7. **Nessuna emissione — opcode o flag — prima di O1** (KS-TH-98-3, KS-DS-98-4).

Voci rimaste in piedi e non evase: A-TH-98-5 (ri-registrare A-TH-97-4 `gc_note`
e A-TH-97-5, evaporati dal backlog) e A-TH-98-6 (design96 arbitra con un +2,9%
che non trascrive: un documento di decisione non auditabile non decide).
