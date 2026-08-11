# NEXT_SESSION — phpr: OBIETTIVO PARITÀ (≥1×) con l'oracle; ≤3× = tappa (REGOLE §1)
⏱ **FONDAMENTALI**: rif WP **full PULITO = 1,757–1,797** (S-131, MA misurato @
s130: la coppia va RIFATTA sul pin nuovo — E1-KO tocca FieldAssign e WP ne ha) ·
ultima leva SPEDITA **S-131 (E1-KO resolve-once → PIN NUOVO s131)** · sessioni-
senza-leva = 0 · incidenti: storici 9 + **1 app. S-131** (gate quiescenza auto-
morso da «phpr» nell'argv dell'arbitro; fermo pre-misura, fix path neutro).

## Scoreboard (pin **s131 ff66cb84**6e6cd439 + server s131 97ed6e06; micro gate promozione S-131)

**arith 5,6 · prop 5,6 · calls 4,9 · str 4,3 · arr 3,2 · re 2,5** (mosse ≤0,1 =
jitter denominatore) · oggetti POST-E1-KO: **objdatains 7,5 (1220,0 ns, −33) ·
churn 8,3 (1490,0) · dropdef 8,8 · alloc 7,8 (ctor non toccato) · allocni 9,8 ·
objmap 17,3** · **MODELLO PROP_STEP** (s131-propstep-lettura.md, chiusura 93–94%):
E−E2 166,9 @ s130 = prop_step 130,7 (guardie 49,4 · defer 37,0 · key+op 34,3 ·
borrow 1,5) + dispatch 36,3; resolve statement ora ~10 (1 sito); residui NOMINATI:
**lookup-once props-map** (~3 lookup→1 dentro 81,9 non-resolve) · **ctor 70,8**
(4 resolve, tocca New/PropSet) · dispatch 36,3 · MAPPA (net): WP 1,76–1,80(@s130)
≈ compoff 1,86–1,89 ≪ hf 2,55 ≪ hk 4,3 ≪ dbal 8,6 ≈ ORM 8,5 · corpus **1414** ×2.

## §S-132 — ordine proposto

1. **Coppia WP full+media sul pin s131** (ricetta s131-pair.sh RIUSABILE: cambiare
   SOLO i pin attesi; warm-up leg + mediana PER MOTORE + quiescenza rc in header):
   aspettativa = riduzione piccola di segno fisso (E1-KO −33 ns/statement su
   datains); REPORT_GAP_132 col nuovo riferimento.
2. **Leva «lookup-once» sulla props-map** (forma GIÀ NOMINATA dal modello S-131):
   in field_write_prop_step la STESSA chiave subisce ~3 lookup (props.get in
   guardie + contains in defer + contains/get_mut in key+op) — una entry-once
   (get_mut/entry riusata) dentro gli 81,9 ns non-resolve; criterio SUO
   pre-registrato (bande objX ora hanno serie s131: gambe B committate S-131);
   ricetta A/B = s131-ab.sh (A=stash s131, path B NEUTRO — lezione argv).
3. Se il tempo resta: **forma ctor** (70,8 ns = 4 resolve da PropSet/init;
   tocca New/ctor — sonda per-sito prima della forma, ricetta propstep riusabile).
4. Mappa residui per NOME: lexer/inflector/event-manager · wp-cli · PHPUnit-self ·
   strumento DENSITÀ evalcls · §3.20 dbal.
APPARATO + az.rev. S-131 (MISURA, vincolanti): soglia giudice = max(drop-1,
spread storico TRA batch sul pin) (#1) · verdetto R=5 riconcili |D_smoke−D_R5| e
D vs UB del modello (#2) · pair: riferimento anche PER CONFIGURAZIONE (on-only =
default) + firma per gamba (#3) · trange tie-break deterministico (#4) · verdetto
= file NUOVO per tentativo, mai append (#5) · argv dei lanci senza «phpr» ·
Serena pre-misura, mai edit .rs in finestra · rc quiescenza sempre in header.

## Aperture per NOME (si pesca solo se blocca o avanza l'oggetto)

resolve del CTOR (70,8 ns, 4/iter da PropSet/init: forma separata) · dispatch
fuori prop_step 36,3 · «prop_step altro» 8,5 · AssignOp/IncDec fuori perimetro
F4/E1-KO (comporre) · famiglia locale 170 ns · evalcls 316,9× · refl 42,4× · re
+2,00 alloc/iter · leaf-magic ora paga key0 su Denied (freddi, osservare) ·
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
percentuale tra segmenti NON annidati senza controllo a zero eventi · **pattern
del gate quiescenza dentro l'argv del lancio (morso S-131)**.

**Riscritto**: 2026-08-11 (chiusura S-131). Storia: `sessions/` · `gaps/GAP_TREND.md`.
Pre-flight S-132: pin phpr **s131 ff66cb84**6e6cd439 + server **s131 97ed6e06**
(OGNI build canonica rilinka il server: ricontrollare l'hash; stash canonici in
phpr-old-target/release/) · MySQL wp8 con l'elenco · uploads sotto guardia ·
corpus 1414 ×2 modi · target time-probes CALDO · nessuna run detached · ordine
lettura: REGOLE.md → QUI → sessions/WP_SESSION_131.md → wp131-harness/
s131-propstep-lettura.md → wp131-harness/revisione.md → PERF_MAP.md.
