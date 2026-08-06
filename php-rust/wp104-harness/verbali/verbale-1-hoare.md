# Verbale sedia 1 — Hoare (design linguaggio/runtime, safe-only) — Concilio WP-104

**VERDETTO su S-102**: CON EMENDAMENTI. **Su §S-103 provvisorio**: CON EMENDAMENTI.

## 1. `is_gc_container` (zval.rs 216-236)

Il predicato regge sul piano che avevo chiesto (A-HO-103-1): match esaustivo senza
wildcard, variante nuova = errore di compilazione; i due livelli Zend sono codificati
correttamente (`Str`/`Resource` refcounted-mai-collectable → false; `Ref` unwrappato
dal chiamante). I tre siti convergono e `gc_note_slow` (mod.rs 4020-4034) elenca i
bracci irraggiungibili come tripwire — corretto.

**Ma il buco Generator è al terzo rinvio.** La mia A-HO-103-2 chiedeva evidenza
birth-track O fixture; S-102 ha fatto NESSUNA delle due — ha solo scritto il commento.
In Zend il generator È collectable (partecipa al GC, tiene `$this` e i locali del
frame sospeso); qui `Generator(_) → false` E le closure che catturano solo generator
non rootano (mod.rs 4011-4013: il buco si COMPONE). Un buco dichiarato senza dente
che morde è un wildcard con buone maniere. §S-103 lo rimanda ancora a «igiene/backlog»:
inaccettabile come terza deroga.

## 2. Fix §3.13 (`diag_line_marks`)

Il meccanismo è SOUND sotto re-entrancy, verificato sul codice: (i) `diags_rendered`
avanza PRIMA di `raise_diagnostic` (5500-5501) ⇒ un flush interno non ri-legge mai
un indice consumato; (ii) la retain interna (`*i >= cut`) non può cancellare marche
di diag non ancora consumati; (iii) `request_end` (3798-3800) azzera la TRIPLA
diags/diags_rendered/marks insieme ⇒ niente riuso di indici cross-request; (iv)
`SuppressEnd` flusha, non tronca. Tre fragilità NON mortali oggi ma senza guardia:
(a) se `raise_diagnostic` erra (handler che lancia), il `?` a 5501 salta la retain
di 5506 — le marche morte restano finché un flush riesce; innocuo SOLO perché
`diags` non è mai troncata mid-request: invariante VERA ma non DICHIARATA né
assertita; (b) `mark_pending_diag_lines` non ha guardia anti-duplicato e `find()`
prende la prima marca: due siti annidati con range sovrapposti darebbero riga
ambigua in silenzio; (c) in `PropGetDynamic` il warning di coercizione del NOME
(3585) cade prima di `before_diags` (3615) e resta a riga-di-flush — incoerenza
minore nello stesso op, da catalogare.

## 3. Audit INV-RECV-1 — REFUTAZIONE CAPITALE (della completezza, non della leva)

**RC-HO-104-1**: la tavola calcola i verdetti sotto l'assunzione DICHIARATA
«base = 2 (created + slot)». Il caso ricevitore TEMP (`(new C)->prop`, catene) ha
base = 1: mid-arm post-move = 2 ESATTO, e due osservatori exact-count hanno soglia
`== 2` (#4 mod.rs ~4126, #6 ~4458 `exclusive`). Per #6 il canale esige che il
ricevitore sia figlio di un oggetto in cascade (che aggiunge +1 di slot), quindi
la leva PROBABILMENTE regge — ma «probabilmente» è ciò che l'audit esisteva per
eliminare. «INV-RECV-1 REGGE su tutti» va riscritto «regge per ricevitori
slot-held; base=1 non esaminata». Verificato invece sul codice: il move NON è
stato esteso di soppiatto (PropGetSilent/Dynamic/ThisPropGet clonano ancora); i
commenti ai due siti MOVE ci sono; i 12 osservatori però sono pinnati a «~riga»,
che il primo refactor sposta.

## Emendamenti

- **A-HO-104-1**: Generator — in S-103 punto 5, TIMEBOXED: fixture di morso
  (generator che tiene l'ultima ref a un oggetto con `__destruct`, drop
  mid-request, diff oracle ×2 modi) O evidenza birth-track. Terza deroga = no.
- **A-HO-104-2**: riga base=1 (ricevitore temp) nella tavola INV-RECV-1 per gli
  exact-count #4/#6 con argomento di raggiungibilità scritto, o fixture 19
  (temp receiver + gc mid-arm); fino ad allora l'esito dell'audit si legge
  ristretto agli slot-held.
- **A-HO-104-3**: marcatori stabili nel codice ai 12 siti osservatori
  (`INV-RECV-1 #n`) — la tavola non deve dipendere da numeri di riga.
- **A-HO-104-4**: dichiarare+assertire l'invariante «diags mai troncata,
  diags_rendered mai riavvolto mid-request»; debug_assert anti-sovrapposizione
  in `mark_pending_diag_lines`.
- **A-HO-104-5**: H-C2 «drop fast-out scalare» DEVE esprimere la specie via
  l'unico predicato (o un fratello nominato con tripwire esaustivo), mai una
  terza lista `matches!` ad-hoc.

## Kill-switch

- **KS-HO-104-1**: nuova lista di specie duplicata senza tripwire di
  compilazione = reject.
- **KS-HO-104-2**: qualunque path che tronchi `diags` o riavvolga
  `diags_rendered` mid-request senza purgare le marche = reject.
- **KS-HO-104-3**: riclassificare `Generator → true` senza fixture+birth-track
  INSIEME = reject.

**Refutazioni capitali**: 1 (RC-HO-104-1, sopra).
