# Team «ricevitore» — fase 2 Concilio WP-104 (sedie 1 Hoare, 2 Matsakis, 8 Stogov)

Relatore: sintesi dei soli verbali `verbale-1-hoare.md`, `verbale-2-matsakis.md`,
`verbale-8-stogov.md`. **I verbali individuali restano la fonte vincolante.**

Perimetro: audit INV-RECV-1 · fixture famiglia MOVE · is_gc_container · fix §3.13 ·
H-C2 via predicato unico.

---

## 1) CONVERGENZE (per NOME degli emendamenti)

**C1 — Fixture 19 «ricevitore a soglia» (A-MA-104-1 ∘ A-HO-104-2).**
Le due capitali sono complementari e chiedono la STESSA cosa da due lati: Matsakis
(RC-MA-104-1) prova che le fixture 17-18 siedono a distanza ≥2 dalla soglia (slot
`$o`/`$h` tenuto per tutta la finestra ⇒ un MOVE bacato a 0 handle sarebbe MASCHERATO);
Hoare (RC-HO-104-1) prova che il caso base=1 (ricevitore temp) non è esaminato sui due
osservatori exact-count `==2` (#4 mod.rs ~4126, #6 ~4458). Convergenza operativa: UNA
famiglia fixture 19 a due corni — 19a ricevitore-ultima-ref mid-arm (dtor/init fa
`unset` dell'ultima ref esterna del ricevitore + `gc_collect_cycles()`; #1/#6/#7/#8 a
soglia±1) e 19b ricevitore TEMP `(new C)->prop` + gc mid-arm (base=1 sui `==2`).
Attese PRIMA, oracle, 2 modi. In alternativa al 19b Hoare accetta l'argomento di
raggiungibilità SCRITTO in tavola.

**C2 — Fino a fixture 19: esito audit RISTRETTO.**
Entrambe le capitali impongono la stessa riscrittura: «INV-RECV-1 REGGE su tutti» →
«regge per ricevitori slot-held; base=1 non esaminata; guardia sul regime minimo =
solo audit statico». Il commento in testa alla fixture 17 («la finestra −1 è QUESTA»)
va ridimensionato (RC-MA-104-1). Nessuna promozione fatta (H-C1a/b) cade: lo dicono
esplicitamente Stogov («nessuna misura promossa cade») e la natura delle due capitali
(completezza dell'audit, non la leva).

**C3 — Gate su estensione MOVE e H-C1c (KS-MA-104-2 ∘ KS-ST-104-1 ∘ A-HO-104-2).**
Fixture 19 verde ×2 modi PRIMA di qualunque estensione della famiglia MOVE e prima
dell'apertura H-C1c (si compone con KS-MA-103-2 e KS-ST-103-2, ATTIVI). Tre sedie
allineate; verificato da Hoare che il MOVE non è stato esteso di soppiatto
(PropGetSilent/Dynamic/ThisPropGet clonano ancora).

**C4 — H-C2 SOLO via predicato unico (A-HO-104-5 ≡ A-MA-104-4; KS-HO-104-1 ∘
KS-MA-104-1).**
Convergenza letterale: il drop fast-out DEVE esprimere la specie via
`Zval::is_gc_container` (o fratello nominato con tripwire esaustivo) — mai una terza
lista `matches!` ad-hoc; variante nuova = errore di compilazione, non ingresso nel
fast-out. Reject senza appello se il fast-out salta un drop su `is_gc_container ==
true` o bypassa una `gc_note` che lo slow path emette (KS-MA-104-1).

**C5 — Generator: terza deroga VIETATA, si esige il DATO (A-HO-104-1 ≡ A-ST-104-3).**
Hoare e Stogov chiedono la stessa fixture: generator che tiene l'ultima ref a un
oggetto con `__destruct` / generator-in-cycle + `gc_collect_cycles()`, diff oracle
×2 modi, TIMEBOXED in S-103 punto 5. Il buco si COMPONE con le closure che catturano
solo generator (mod.rs 4011-4013). Se morde: fix o riga in PHPR_DIVERGENCES — il buco
non resta senza dato. ⚠️ Collisione di numerazione: anche Stogov la chiama «fixture
19» — rinumerare (proposta: famiglia 19 = ricevitore a soglia; 20 = generator).

**C6 — §3.13: claim ridimensionato + invarianti dichiarate (KS-ST-104-3 ∘
A-HO-104-4).**
Convergenza su assi diversi ma componibili: Stogov — «FEDELE» vale SOLO per la
famiglia PropGet timbrata (5 siti su ~435 accodamenti); line-faithful, non
mechanism-faithful (Zend chiama `zend_error` sincrono, phpr conserva coda+flush).
Hoare — il meccanismo è SOUND sotto re-entrancy ma su un'invariante VERA e non
DICHIARATA: dichiarare+assertire «diags mai troncata, diags_rendered mai riavvolto
mid-request» + debug_assert anti-sovrapposizione in `mark_pending_diag_lines`
(KS-HO-104-2 a presidio). Corollari Stogov: fixture handler-timing (A-ST-104-1),
censimento famiglia fetch §3.11/§3.12 per NOME (A-ST-104-2). Incoerenza minore
PropGetDynamic (warning coercizione nome a 3585 prima di before_diags 3615) da
catalogare — nessuna obiezione.

**C7 — Igiene dell'apparato di audit (A-HO-104-3 ∘ A-MA-104-2 ∘ A-MA-104-3).**
Nessuna opposizione: marcatori stabili `INV-RECV-1 #n` nel codice (la tavola non
dipende da numeri di riga); estensione dell'audit alle API uniqueness-gated
(`Rc::get_mut`/`try_unwrap`/`make_mut`, `weak_count`) con ZERO registrato per nome
se occorrenze=0 — un get_mut al −1 flipperebbe da fallimento a mutazione in-place;
KS-MA-103-2/103-3 riportati in NEXT_SESSION (regola fonte-unica).

---

## 2) CONFLITTI (con la posizione di ciascuna sedia)

**K1 — Ref in `is_gc_container`: Hoare vs Stogov.**
- Hoare (§1): il design «`Ref` unwrappato dal chiamante» è corretto così com'è.
- Stogov (b): contraddizione doc/codice — il commento dice «unwrapped by the caller»
  ma il match ritorna `Ref(_) => true`; in Zend `gc_check_possible_root` scartoccia
  GC_REFERENCE e guarda `Z_COLLECTABLE` del valore interno: una Ref(Long/Str) non si
  bufferizza MAI; ogni chiamante che non scartoccia = churn che Zend non ha.
  A-ST-104-4: o il predicato scartoccia, o l'assert A-MA-103-2 diventa NON opzionale.
  KS-ST-104-2: nessuna nuova leva sul canale gc_note si promuove finché non sanato.
- Matsakis: neutrale sul punto (verifica solo che il MOVE esclude `Ref` per
  costruzione).
- **Proposta di composizione**: adottare il ramo «assert non opzionale» di A-ST-104-4
  — sana la contraddizione senza introdurre una seconda lista di specie, quindi
  compatibile con KS-HO-104-1; e sblocca KS-ST-104-2 che altrimenti GATE la
  promozione di H-C2 (il fast-out vive sul canale gc_note).

**K2 — Rimedio Generator: soglia di riclassificazione.**
- Stogov (A-ST-104-3): se la fixture morde → «fix o riga in PHPR_DIVERGENCES».
- Hoare (KS-HO-104-3): riclassificare `Generator → true` SENZA fixture+birth-track
  INSIEME = reject.
- **Proposta**: nessun conflitto sul primo passo (la fixture decide se morde); sul
  rimedio prevale il KS più restrittivo — flip a `true` solo con fixture E
  birth-track; se manca il secondo, la via è la riga in PHPR_DIVERGENCES.

**K3 — Portata dell'argomento holder sul caso base=1: Matsakis vs Hoare.**
- Matsakis (§1): l'argomento «handle mosso = holder ESTERNO per ogni soglia» regge
  *indipendentemente* dallo slot (soglie relative, +1 in entrambi gli schemi).
- Hoare (RC-HO-104-1): per gli exact-count `==2` (#4/#6) con base=1 il post-move è
  ESATTAMENTE a soglia — «probabilmente regge» (per il canale cascade di #6) è ciò
  che l'audit doveva eliminare.
- **Proposta**: non è un conflitto di sostanza ma di onere della prova — si chiude
  con C1/C2: argomento scritto in tavola O fixture 19b; fino ad allora vale la
  lettura ristretta di Hoare (più conservativa).

**K4 — §3.13 «sound» vs «fedele»: assi diversi, nessuna collisione reale.**
Hoare certifica la SOUNDNESS del meccanismo coda/marche sotto re-entrancy; Stogov
nega la FEDELTÀ al timing Zend (handler che echoa: interleaving al flush, non alla
lettura) e al perimetro (~435 siti non timbrati). Entrambi i verdetti stanno in
piedi insieme: il claim si riscrive «sound + line-faithful sui 5 siti PropGet», mai
«FEDELE» tout court (KS-ST-104-3).

---

## 3) PRIORITÀ PROPOSTE per l'ordine S-103

Regola di ammissione applicata: apparato solo se BLOCCA l'oggetto. Le capitali
RC-HO-104-1 e RC-MA-104-1 bloccano l'ESTENSIONE del MOVE e H-C1c — NON le promozioni
fatte (H-C1a/b restano promosse; Stogov: «nessuna misura promossa cade»).

1. **Collaudo php-server 2c4242b6** — invariato primo atto (fuori perimetro team;
   Matsakis: non tocca il suo perimetro).
2. **Guardie MOVE = pacchetto fixture 19** (BLOCCA estensione MOVE + H-C1c, quindi
   ammesso per intero): 19a ultima-ref mid-arm + 19b temp-receiver base=1 (o
   argomento scritto), attese PRIMA ×2 modi; riscrittura esito audit «slot-held»
   (C2, costo ~zero, subito); marcatori `INV-RECV-1 #n` (A-HO-104-3); estensione
   audit a get_mut/try_unwrap/make_mut con ZERO per nome (A-MA-104-2). Gate:
   KS-MA-104-2, KS-ST-104-1.
3. **Sanare Ref (A-ST-104-4, ramo assert)** — PRE-CONDIZIONE della promozione H-C2:
   KS-ST-104-2 gate ogni nuova leva sul canale gc_note; ammesso perché blocca
   l'oggetto H-C2. Un solo predicato, nessuna lista nuova (KS-HO-104-1).
4. **H-C2 drop fast-out** via predicato unico `Zval::is_gc_container`
   (A-HO-104-5 ≡ A-MA-104-4), canale contato, banda pre-registrata [8,22], A/B da
   sola; reject-line KS-MA-104-1. Ammessa da tutte e tre le sedie.
5. **Fixture Generator TIMEBOXED (A-HO-104-1 ≡ A-ST-104-3, rinumerata 20)** — la
   terza deroga è vietata: o il dato (morde/non morde) o birth-track; esiti da K2.
6. **Timebox §3.13**: nel timebox già previsto — (i) riscrittura claim per
   KS-ST-104-3 e (ii) assert invarianti A-HO-104-4 (costo minimo, presidiati da
   KS-HO-104-2); (iii) fixture handler-timing A-ST-104-1 e censimento fetch
   A-ST-104-2 SOLO se il timebox regge, altrimenti backlog per NOME (non bloccano
   nessun oggetto S-103). Catalogare l'incoerenza PropGetDynamic.
7. **Chiusura**: KS-MA-103-2/103-3 (+ i KS-104 attivi) riportati in NEXT_SESSION
   (A-MA-104-3, regola fonte-unica); risolvere la collisione di numerazione fixture
   19/20 nella stessa passata.

Fuori ordine ma registrato: fixture 15-bis (A-ST-104-5, float frazionario su typed
int + corno typed-REF §3.12) e corno `unset`-in-dtor — backlog nominato, non
bloccano oggetti S-103.
