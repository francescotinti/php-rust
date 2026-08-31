# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-165 = PROMOZIONE L-MC1d «MethodCall.borrow k≤2 outline»
(pin NUOVO s165): giudice proprio mc2 170,5→155,5 ns/iter D=+14,5 (−8,5%),
R=5 riconc. |0,0|; percorso a 4 bracci col NULL-EDIT come arbitro (attribuzioni:
arrload=unreachable!×2 ~5 ns NON montati; missload=layout run_loop; banda-layout
host-call FONDATA missload 8,0/arrfilter 6,0; backtrace/objmap=quantizzazione
ri-risolta a tick≤1ns)** + az.rev. S-164 **5/5 CHIUSE** (creep arith REFUTATO
Δ=+0,12<0,52 · census AL3 Δ=199999 ESATTO, +1 PROVATO=buffer Vec exts · ORA_REF
4,885 REGGE, banda sentinella ORM [4,83;4,94] ATTIVA · ricetta server provata ×2
feature axum-server · vieto rumore>soglia) + istruttoria dbal ictx CHIUSA
(firma=DENOMINATORE, emenda per-motore da montare nel criterio coppia) ·
leve S-165: **1 PROMOSSA** · incidenti 0 · revisione: REGGE CON RILIEVI (lente
semantica: error-path materializzazione, azioni sotto) · QUESITI UTENTE
invariati: (a) T2/A2 sospendere; (b) census server (15° slitt.); (c) ratifiche
pendenti; (d) emenda §3.

## Scoreboard (pin NUOVO s165 phpr 1fd8757d2f72dc3e + server cf7afe37f29016a8)
**arith 5,4 · prop 5,6 · calls 4,9 · str 4,2 · arr 3,2 · re 2,6** (micro promo
s165; scarti vs s163 = tick del denominatore oracle, guardie A/B sugli stessi
binari D≈0) · giudice proprio **mc2 155,5 ns/iter** (da 170,5) · WP t14 1,761 ·
media 2,341-2,450 · ORM [7,066;7,111] (registrato s162) · dbal [7,459;7,491] ·
corpus **1412×2** · batteria 1748 · denti: run.rs 6914 · mod.rs 25909 ·
host.rs 7726.

## §S-166 — ordine
1. **Fixture semantica fx-mc2** (az.rev.1 S-165, PRIMA della coppia: se morde,
   il pin si emenda prima di spendere la coppia): try/catch su `$o->add($a[],1)`
   con handle-id post-catch · ArgPlace via __get (anche che lancia) ·
   `$x =& $o->retref(1,2)` · __destruct su last-ref d'argomento · Fiber-subclass
   al sito condiviso. Se morde ⇒ az.rev.2: riallineare l'error-path (drop di
   frame PRIMA di recv, o materializzare prima del pop) + az.rev.3: decidere il
   frame pooled su Err (riciclo o divergenza a catalogo). A/B di conferma solo
   se l'edit tocca il fast path.
2. **Coppia WP t15 + ORM DOVUTA sul pin vigente** (az.rev.4; regola
   focus-oggetti: a OGNI pin nuovo): criterio coppia con emenda ictx PER-MOTORE
   (istruttoria s165) e banda sentinella oracle ORM [4,83;4,94] VINCOLANTE.
3. **Leva (obbligo di ritmo)** — candidata per NOME: **MC1-k∞** (togliere il cap
   argc≤2 dal fast path: ammissione già arità-esatta, il cap era prudenziale;
   giudice m-mc3 a 3 argomenti) o, in alternativa, **autoload forme statiche**
   (`"Class::method"`/`["Class","m"]` → call_static_one). Smoke con soglia=4
   vera (rc=8 sul rumore); guardie host-call CON banda-layout fondata.
4. Emende §3 + quesiti (a)-(d) se l'utente ratifica.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)
fx-mc2 semantica error-path · MC1-k∞ · autoload statiche/k≥2 · sonda strmap
(az.3 S-162, PRIMA di leve strmap) · emenda ictx per-motore (nel criterio
coppia) · gamba server census (15°) · elementi array_map-filter non censiti ·
§3.27 · §3.26 · §3.25 · §3.24+§3.23 · slot-load · §3.22 · depr. float→int ·
warning ×2 · div. RMW · objmap → GC · evalcls 316,9× · refl 42,4× · re +2 ·
§3.13/§3.12-i/§3.14/§3.21 · get_gc · latin1 · dbal 10 nomi · attesa-AF1.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)
**S-165: guardia host-call a soglia 4 SENZA banda-layout fondata · verdetto su
guardia con giudice quantizzato senza ri-risoluzione a tick≤soglia/4 · azione
di processo (unreachable!) montata senza prezzo misurato · batteria PRIMA dello
stash senza env ricetta (churn) · fast path INLINE in run_loop (layout: forma
outline #[inline(never)]) · claim «semanticamente identico» senza fixture
sull'error-path.** S-164: Box/Vec-pooling puro senza cambio dispatch · copia-
gate sulle sole righe attese · build di gemello senza ricetta · git apply da
sottodir. S-163: census.done scritto dalla sessione · attesa per analogia.
S-162..S-152: coeff di famiglia trasportato · oracle fuori rif · copia-gate
senza verifica · giudizio da un estremo · leve a scala SUITE. Trasversali:
BOLT/NaN-boxing/PGO · pin senza collaudo · differenze A/B come cifra ·
denominatori a memoria · rc da pipe · edit coi build in volo · promozione
sotto banda · claim di ASSENZA oltre risoluzione.
**Riscritto** 2026-08-31 (chiusura S-165; storia in `sessions/` · `gaps/`).
Pre-flight S-166: pin phpr **s165 1fd8757d**2f72dc3e + server **cf7afe37**
f29016a8 (SOLO via pin-*.sh) · Data 12G al prune · MySQL wp8 con l'elenco ·
uploads sotto guardia · corpus 1412 · stash: phpr-s165 · php-server-s165 ·
bracci storici s165 (mc1-B 9838e732 · mc1b-B f8e09681 · mc1d-D==pin ·
nulledit-C 8ddd78c5) · lock misura da CREARE · **COPPIA DOVUTA** · lettura:
REGOLE.md → QUI → wp165-harness/{s165-promo-verdetto,s165-criterio-nulledit
(banda-layout),s165-ririsolvi-verdetto,s165-census-al3-verdetto,
s165-orm-oracle-r5-verdetto,s165-arith-creep-verdetto}.out/.md +
s165-istruttoria-ictx-orm.md → revisione.md → sessions/WP_SESSION_165.md →
PERF_MAP.md.
