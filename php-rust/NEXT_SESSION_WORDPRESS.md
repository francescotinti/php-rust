# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **full ON-ONLY CANONICO = 1,752–1,768** (S-132 @ pin
s131, az.rev. #3 per-config; N=2 coppie — intervallo di due punti, non banda;
da RIFARE su s132: L-LO1 morde FieldAssign non-leaf) · ultima leva SPEDITA
**S-132 (L-LO1 lookup-once → PIN NUOVO s132)** · sessioni-senza-leva = 0 ·
incidenti: storici 10 (9 + 1 app. S-131) · S-132: 0 nuovi.

## Scoreboard (pin **s132 6af6e497**5ef8d0bf + server s132 ad17a10d85cc8471; micro gate promozione S-132)

**arith 5,5 · prop 5,5 · calls 5,0 · str 4,3 · arr 3,1 · re 2,5** (mosse ≤0,1 =
jitter denominatore) · oggetti POST-L-LO1: **objdatains 7,2 (1200,0 ns, −20) ·
churn 8,2 (1473,3) · dropdef 9,0 (denom. 0,41 sotto-scala) · alloc 7,8 (ctor non
toccato) · allocni 9,7 · objmap 17,3** · residui NOMINATI dal modello prop_step
(s131-propstep-lettura.md): **forma ctor 70,8** (4 resolve, tocca New/PropSet) ·
dispatch 36,3 · non-resolve residuo ~60 ns/statement · MAPPA (net): WP 1,75–1,77
on-only (@s131) ≈ compoff 1,86–1,89 ≪ hf 2,55 ≪ hk 4,3 ≪ dbal 8,6 ≈ ORM 8,5 ·
corpus **1414** ×2.

## §S-133 — ordine proposto

1. **Fixture destructor-window PRIMA di ogni leva** (az.rev. S-132 #1, lente
   SEMANTICA: il «by construction» di L-LO1 ha un buco nella finestra di
   teardown GC — `Props::new()` a layout vuoto + `__destruct` che ripopola una
   prop dichiarata poi usata non-leaf → oggi MagicDescend/errore dove prima
   scriveva; NESSUN gate lo copre): scrivere il caso, diff vecchio/nuovo
   (stash s131 vs pin s132), decidere cura o divergenza a catalogo.
2. **Coppia WP full+media sul pin s132** (ricetta s132-pair.sh RIUSABILE:
   cambiare SOLO i pin attesi; tentativo NUOVO = file nuovo; per-config on-only
   canonico + firma per gamba): aspettativa = riduzione piccola di segno fisso
   (L-LO1 −20 ns/statement su datains) → REPORT_GAP_133.
3. **Forma ctor** (70,8 ns = 4 resolve da PropSet/init, 17,7/resolve: cammino
   Denied/Dynamic del ctor): sonda per-sito PRIMA della forma (ricetta
   propstep riusabile), poi criterio pre-registrato con soglia = max(drop-1,
   spread-batch sul pin s132 — le gambe B di S-132 sono nei verdetti).
4. Se resta: dispatch fuori prop_step 36,3 · mappa residui per NOME
   (lexer/inflector/event-manager · wp-cli · PHPUnit-self · densità evalcls ·
   §3.20 dbal).
APPARATO + az.rev. S-132 (SEMANTICA, vincolanti): #1 fixture teardown (p.1
sopra) · #2 `debug_assert!` slot0 ⇒ presenza slot == contains(key0) in
field_write_prop_step · #3 disciplina `replace_slot` (Err su indice estraneo)
anche a `get_slot_mut`, o invariante documentata sul tipo Props · #4 FATTA in
S-132 (nota N=2 in PERF_MAP) · #5 commento L-LO1: delimitare hooked-backed
non-leaf come pre-esistente (edit .rs solo fuori finestra di misura, build
emendata dichiarata se rilinca) · argv dei lanci senza pattern del gate ·
Serena pre-misura, mai edit .rs in finestra · rc quiescenza sempre in header.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

dispatch fuori prop_step 36,3 · «prop_step altro» 8,5 · AssignOp/IncDec fuori
perimetro F4/E1-KO/L-LO1 (comporre) · famiglia locale 170 ns · evalcls 316,9× ·
refl 42,4× · re +2,00 alloc/iter · leaf-magic ora paga key0 su Denied (freddi) ·
micro-trim morte (is_empty SipHash · has_destruct · FxHash per-id) · $z++/$z--
undef non warna · §3.13 · §3.12-i · §3.14 · get_gc · drift TODO.md · latin1-cliff.

## NON riproporre (i veti restano; dettaglio nei concili archiviati)

BOLT su Mach-O · NaN-boxing · threaded-dispatch · PGO sui giudici · verdetti su
build emendata senza ri-banda · pin/stash senza collaudo-nell'atto · contenitori
sul call path · differenze tra A/B distinti come cifra · componenti prezzate ·
magnitudine ripartita senza A/B proprio · «icache» NON-premessa · pre-filtro che
tassa i freddi · guardie non-bersaglio BILATERALI · denominatori a memoria ·
output di run nel repo · rc di gate da pipe · tee/log pre-mkdir · admission sul
dump intero · xctrace senza guardie disco · run pesanti come task · edit coi
build in volo · promozione sotto banda · gate a soglia fissa senza banda ·
corpus-gate solo-nomi · strumentazione nei sorgenti del pin (#[path] fuori-crates
= ricetta) · leve micro senza banda v2 · alloc-removal senza modello del costo
SOSTITUTIVO · probe senza riferimento vivo · ordine FISSO di misura · delta tra
census di epoche diverse senza datare i raw · verdetti da script non committati ·
SSO inline · inline-array init+drain args · claim di ASSENZA oltre la risoluzione
· smoke con fam-min > R · notti su PhpStr-full · guardie su giudici diversi dalle
loro bande · misure con LSP in volo · F2 keys-scratch · quiescenza nello stesso
comando del lancio · output TRACKED mossi da orchestratori · rumore-soglia =
range PIENO senza formula robusta · guardia su categoria senza banda propria ·
percentuale tra segmenti NON annidati senza controllo a zero eventi · pattern
del gate quiescenza dentro l'argv del lancio (morso S-131).

**Riscritto**: 2026-08-12 (chiusura S-132). Storia: `sessions/` · `gaps/GAP_TREND.md`.
Pre-flight S-133: pin phpr **s132 6af6e497**5ef8d0bf + server **s132 ad17a10d**85cc8471
(OGNI build canonica rilinka il server: ricontrollare l'hash; stash canonici in
phpr-old-target/release/) · MySQL wp8 con l'elenco · uploads sotto guardia ·
corpus 1414 ×2 modi · nessuna run detached · ordine lettura: REGOLE.md → QUI →
sessions/WP_SESSION_132.md → wp132-harness/revisione.md → PERF_MAP.md.
