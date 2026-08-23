# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: **S-155 = coppia @ s154 SALDATA + istruttoria CE1
CHIUSA, pin INVARIATO s154** · WP t6 mediana 1,771 COMPATIBILE (6/6 pulite,
banda_ON 0,022 record; attesa CE1 «piccola/nulla» rispettata) · **ORM
6,972–7,053 = PRIMA VOLTA SOTTO 7×** (Δ +0,41/+0,50 giù fuori rumore,
OLTRE-attesa 0,03–0,07 dichiarata) · dbal 7,385–7,422 · **sonda ce-count:
k_post=1 SPIEGATO al sorgente — 1×32 B = args-Vec di pop_keys attribuito al
nome (scope s149 apre PRIMA di pop_keys); hit-path CE1 = 0 alloc CONFERMATO;
controllo fe-count k=1 b=16,0 ESATTI; canale == H-D S-103 col SITO nominato**
· OLTRE-attesa ORM: meccanismo candidato NOMINATO = resolve_class_autoload è
FUNNEL di 11 siti (instanceof stringa, new $class, catch, reflection…),
magnitudine non ripartita senza census · **gdc-count: per_classe=3,0 fisso=1
ESATTI ⇒ ~636 chiamate, 0,031 s ≪ 0,293 ⇒ fetta NON PAGANTE (declassata)** ·
CORREZIONI attese (istruttoria): chiamate class_exists ORM ∈ [3,25M; 4,87M]
(mix forme, k_pre 3 vs 2); testa census post-CE1 attesa ≈ chiamate×1, NON ~0
· leve spedite S-155: **0 (tentate 0) — ANOMALIA dichiarata** (misure dovute
+ gdc refutata prima del criterio) ⇒ S-156 DEVE tentare una leva · incidenti
**19** (=) · QUESITI UTENTE: (a) T2/A2 — raccomandazione: SOSPENDERE (fetta
borrow ≤0,17 s < risoluzione; rientro = prezzo in-contesto NUOVO sopra
soglia), ratifica al contatto; (b) census server (5° slittamento).

## Scoreboard (pin s154 phpr bddc050320a6af4c + server b3cf348f69739edc)
**arith 5,5 · prop 5,6 · calls 4,8 · str 4,3 · arr 3,3 · re 2,5** (promo
s154; micro NON rimisurate in S-155, pin invariato) · WP t6 1,771 (mediane
t1..t6: 1,799·1,738·1,789·1,781·1,757·1,771; banda giudizio [1,738;1,799]) ·
media 2,456–2,510 · **ORM 6,972–7,053** · dbal 7,385–7,422 · corpus
**1412×2** (di promo s154) · batteria 1748/0/2 (cap 25712/7661).

## §S-156 — ordine
1. **Census ORM col probe s155** (3e6b5008482c32d0, stash ×2; ricetta
   s154-census-orm.sh riusabile come copia dichiarata): rifonda la testa
   per-NOME post-CE1 con le attese CORRETTE (class_exists ≈ chiamate×1 ⇒
   scioglie il mix [3,25M;4,87M]; debug_backtrace ≈ 473k×13; identità §3
   ×2 repliche) + totali per l'apporzionamento del funnel CE1(b).
2. **LEVA TENTATA (obbligo di ritmo, A/B qualunque verdetto)** — candidata
   prima: **args-Vec dei host-builtin** (estensione HD2-forma-2 a
   CallHostBuiltin: sito NOMINATO in S-155, run.rs scope/pop_keys; prezzo
   ~7 ns/chiamata × N hostcall dal census p.1 = attesa FONDATA prima
   dell'edit). Alternative per NOME: __reflect_* famiglia ~8,7M ·
   array_map 7,7M (PRIMA istruttoria di forma: k dipende da arità/callback).

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)
args-Vec hostcall H-D (sito nominato S-155) · census ORM probe s155 (p.1) ·
__reflect_* ~8,7M · array_map 7,7M (istruttoria forma) · gamba server census
(5°) · CI: corpus-FAIL d'ambiente sui 3 test backtrace (bug64239_2,
debug_backtrace_limit, debug_backtrace_options — nei gate di record passano;
apparato) · MethodCall.borrow k=2 · §3.24+§3.23 backtrace · slot-load 0,99× ·
§3.22 · depr. float→int · warning ×2 · div. RMW · objmap 43,4 → GC · evalcls
316,9× · refl 42,4× · re +2 · §3.13/§3.12-i/§3.14/§3.21 · get_gc · latin1 ·
dbal 10 nomi · gdc DECLASSATA (~636 chiamate, 0,031 s: micro-judged se mai).

## NON riproporre (i veti restano; dettaglio nei concili archiviati)
**S-155: attese-conteggio su census per-NOME senza il termine di PLUMBING
(scope prima di pop_keys) · leve su fette < risoluzione suite senza
dichiararle micro-judged (gdc) · attese per-nome per cure su FUNNEL.**
S-154: identità pin a hash esatto su build fredda (si chiude a CONTENUTO) ·
guardie quantizzate con soglia sotto il tick · argv "phpr" in foreground.
S-153: braccio A non-gemello (§7-bis) · leve borrow senza prezzo in-contesto.
S-152: A3c senza numeri nuovi · leve a scala SUITE · calcolatori senza
collaudo. S-151: cifre census pre-BT1 · arena/bump-reset · spacchettare
exec/ops_* · Fase-5 registri. S-150: A/B fuori ricetta · attese senza
pavimento · forge non collaudate. Trasversali: BOLT/NaN-boxing/threaded-
dispatch/PGO-sui-giudici · pin/stash senza collaudo-nell'atto · differenze
tra A/B come cifra · componenti prezzate · denominatori a memoria · rc da
pipe · run pesanti come task · edit coi build in volo · promozione sotto
banda · strumentazione nel pin · claim di ASSENZA oltre risoluzione · misure
con LSP in volo · giudice sotto-risoluto.
**Riscritto** 2026-08-23 (chiusura S-155); storia in `sessions/` · `gaps/`.
Pre-flight S-156: pin phpr **s154 bddc0503**20a6af4c + server **b3cf348f**
69739edc (SOLO via pin-*.sh; `--braccio` per i bracci) · MySQL wp8 con
l'elenco (daemonizer, datadir esterno) · uploads sotto guardia · corpus
1412×2 · batteria s125+rczval+loc_dente (cap 25712/7661) · stash: pin
bddc0503 · gemelloA-s154 2023cbb9 · **probe census-s155 3e6b5008 ×2
(POST-CE1: È il probe giusto per il census p.1)** · attesi smoke da SECONDO
attore prima d'ogni record · braccio A = GEMELLO dal tree (§7-bis) · lock
misura da CREARE a ogni finestra · CI in smaltimento coda post-lock ·
lettura: REGOLE.md → QUI → wp155-harness/{s155-pair-verdetto-t6.out,
s155-orm-coppia-verdetto.out,s155-sonda-ce-verdetto.out,
s155-ce-istruttoria.md,s155-gdc-verdetto.out} → sessions/WP_SESSION_155.md
→ PERF_MAP.md.
