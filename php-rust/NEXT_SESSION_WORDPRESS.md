# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **full PULITO = 1,758–1,805** (S-129, MA misurato @
s127b: la coppia va RIFATTA sul pin nuovo — F4 tocca FieldAssign e WP ne ha) ·
ultima leva SPEDITA **S-130 (F4 prelude-gate → PIN NUOVO s130)** —
sessioni-senza-leva-spedita = 0 · incidenti: storici 8 + **1 app. S-130**
(«7/6 gate verdi» fixture + registro server committato tardi, rev. PROCESSO).

## Scoreboard (pin **s130 0fdf1c49**b16c24ba + server s130 7fb79069; micro gate promozione S-130)

**arith 5,5 · prop 5,6 · calls 5,0 · str 4,3 · arr 3,3 · re 2,5** (↗ apparenti =
jitter denominatore oracle; phpr netto invariato) · oggetti POST-F4: **objdatains
7,7 (1253,3 ns) · churn 8,6 · dropdef 9,0 · allocni 9,8 · alloc 7,8 · objmap 17,0**
· **E1a MISURATA**: per-statement 5 resolve = 39–44 ns ≈24% di E−E2 (~165); UB
resolve-once statement-only 31–35 ns; il GROSSO di E−E2 ~120 ns = prop_step
NON-resolve (3× prop_key Box + contains/get_mut/replace + borrow) · MAPPA (net):
WP 1,76–1,81(@s127b) ≈ compoff 1,86–1,89 ≪ hf 2,55 ≪ hk 4,3 ≪ dbal 8,6 ≈ ORM 8,5
· corpus **1414** ×2.

## §S-131 — ordine proposto

1. **Coppia WP full+media sul pin s130** (ricetta s128/s129: gate ictx/s con
   mediana PER MOTORE, quiescenza gate separato, user-only canonica + companion):
   aspettativa modello = riduzione piccola ma di segno fisso (F4 morde FieldAssign
   prop-rooted); REPORT_GAP_131 col nuovo riferimento.
2. **Modello del costo prop_step NON-resolve** (~120 ns, ora il segmento dominante
   NOMINATO): sonde dedicate dentro `field_write_prop_step` sulla ricetta
   time-probes (3× `fs.prop_key` separate da contains/get_mut/replace e dal
   borrow) — il modello DECIDE la forma E1 di S-132 (resolve-once statement-only
   UB 31–35 vale solo se compone con la cura del resto; prop_key ritorna Box:
   candidato «key-once» = una resolve+key per statement riusata sui 4 siti).
3. Se il tempo resta: A/B della forma nominata dal modello (criterio suo,
   PRE-REGISTRATO, bande fondate v2 — le objX ora hanno serie s130).
4. Mappa residui per NOME: lexer/inflector/event-manager · wp-cli · PHPUnit-self ·
   strumento DENSITÀ evalcls · §3.20 dbal.
APPARATO + az.rev. S-130 (PROCESSO, vincolanti): quiescenza gate SEPARATO prima
di ogni run E il suo rc citato nell'HEADER di ogni verdetto A/B (#3) · fixture
chain: lista gate CONGELATA per NOME nello script di promozione, fail su
inventario diverso (#2) · criterio E1a p.2 da emendare + quota ctor/statement
per call-site CON rerun (#4) · pin-server verifichi tree pulito post-atto (#1) ·
Serena prima della prima misura, MAI edit .rs in finestra di misura · verdetti
avversi committati PRIMA di ogni emenda · bande fondate dalle gambe A committate.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

warm-up leg nella ricetta pair (le 2 escluse S-129 erano PRIME di sequenza) ·
resolve del CTOR (4/iter, 72 ns su objalloc: forma separata, tocca New/PropSet) ·
AssignOp/IncDec fuori perimetro F4 (comporre) · famiglia locale 170 ns · evalcls
316,9× · refl 42,4× · re +2,00 alloc/iter · oracle-denominatore leg-first ·
micro-trim morte (is_empty SipHash · has_destruct · FxHash per-id) · fame
frontend (kpc/sudo) · $z++/$z-- undef non warna · §3.13 · §3.12-i · §3.14 ·
get_gc · drift TODO.md · latin1-cliff · `$GLOBALS['x']->p` resta FieldAssign.

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
**percentuale tra segmenti NON annidati senza controllo a zero eventi (morso
E1a S-130: 67% vs 24%)**.

**Riscritto**: 2026-08-11 (chiusura S-130). Storia: `sessions/` · `gaps/GAP_TREND.md`.
Pre-flight S-131: pin phpr **s130 0fdf1c49**b16c24ba + server **s130 7fb79069**
(OGNI build canonica rilinka il server: ricontrollare l'hash dopo ogni build;
stash canonici in phpr-old-target/release/) · MySQL wp8 con l'elenco · uploads
sotto guardia · corpus 1414 ×2 modi · target census e time-probes CALDI · nessuna
run detached · ordine lettura: REGOLE.md → QUI → sessions/WP_SESSION_130.md →
wp130-harness/revisione.md → wp130-harness/s130-e1a-lettura.md → PERF_MAP.md.
