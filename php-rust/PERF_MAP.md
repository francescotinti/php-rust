# PERF_MAP — phpr vs PHP oracle 8.5.7, mappa multi-workload

Aggiornata: **2026-08-28 (S-160)** · pin phpr **s160 ceeb6e76** + server
**s160 001a4b2b** NUOVI (**S-160 = PROMOZIONE leva L-AF1 «array_filter
plumbing 0-alloc per-elemento: 1-array + closure ANONIMA simple_call arità-1
+ mode==0, ammissione hoistata, per-elemento via call_closure_one RIUSATO
(L-AM1); ogni altra forma sul cammino pieno INVARIATO» con catena rc=0 CON
RETTIFICA in-sessione**: A/B R=5 vs GEMELLO f2d17f18 == pin s159 AL BYTE a
freddo (N=2) **D=+16,0 ns/elemento su m-arrfilter** (196,0→180,0; smoke +14,0
DENTRO banda VINCOLANTE [8;22]; riconc. 2,0<4,0; **FUORI-UB SOPRA dichiarato:
UB 12,0±2,5, eccedenza ~0,5-3,0 su componenti NON prezzate nominate
(dispatch call_callable · match mode · Rc-bump) ⇒ sonda surplus DOVUTA S-161,
orologio §4**), 17 guardie ok (arrmap +2,0: L-AM1 presidiata), bl run_loop
6033==6033; dente loc host.rs 7683→7708 PRE-dichiarato; fx-af 13 forme + fx-am
v2 20 forme BYTE-ID; conferma post-pin m-arrfilter **D=+15,0 segni 5/5**
(RETTIFICA: prima corsa su file inesistente — la copia promo aveva H stantio
wp159, fx-af/conferma FALSI VERDI ri-derivati con esito ESATTO, incidente #21
PROPOSTO + emenda §3 proposta: verifica POSITIVA dei path + marcatore preteso);
micro promo **5,4·5,5·4,7·4,3·3,2·2,6** (arr/re tick denominatore, guardie
verdi); **istruttoria phpr1 CHIUSA**: transitorio INTERNO primo-run (istruzioni
identiche, Δ-daemon≈0), rimedio rodaggio+quiescenza NEL criterio ORM → 4/4
gambe PULITE (prima volta); **coppia t10+ORM saldate @ s159**: WP t10 mediana
1,760 COMPATIBILE (6/6 pulite, banda_ON 0,008 == record) · media 2,434–2,458 ·
ORM 7,077–7,097 (attesa-AM1 COMPATIBILE su scaletta a DUE estremi, Δ_norm
[+0,04;+0,24] INTERO in banda; **replica-AL1 CHIUSA**: attesa 0,02-0,05
sotto-risoluzione CONFERMATA su gambe pulite) · dbal 7,541–7,550; verdetti
`wp160-harness/s160-*.out`; **coppia @ s160 DOVUTA → S-161**); storico S-159
(pin **s159 f2d17f18** + server
**s159 c8e43b58**; **PROMOZIONE leva L-AM1 «array_map plumbing
0-alloc per-elemento: 1-array + closure ANONIMA simple_call arità-1,
ammissione hoistata UNA volta per chiamata, per-elemento senza vec![v] via
intake = braccio WP-37 di bind_params a n=1 (call_closure_one /
push_closure_frame_one); ogni altra forma sul cammino pieno INVARIATO» con
catena piena rc=0 al t1**: A/B R=5 vs GEMELLO 92b0aea3 == pin s158 AL BYTE
anche a freddo **D=+11,0 ns/elemento su m-arrmap** (126,0→115,0; smoke +10,0
DENTRO banda VINCOLANTE [8;22] coi denti NEL giudice; riconc. 1,0<4,0;
**riconc. UB DENTRO il modello: UB TARATO 12,0±2,5 — prima leva della serie
dentro l'UB falsificabile**), 15 guardie ok, disasm run_loop bl 6033==6033;
denti loc 25810/7683 PRE-dichiarati; fixture fx-am 14 forme BYTE-ID; micro
promo 5,4·5,5·4,7·4,3·**3,1**·2,5 (arr 3,3→3,1 companion L-AM1); conferma
post-pin m-arrmap +8,0 segni 5/5 (rev. #5: cifra a REGISTRO [8;11],
drift-tree non quantificato); **SONDA surplus m-refl SALDATA** (az.rev.
S-158 #1, entro orologio §4): conteggi census ESATTI Δ=2 alloc/iter, rimisura
su stash FERMI **D=+24,0±5,0** (il registro L-RF2 [21;29] si RISOLVE: +29 e
+21 entrambi nel rumore della cifra nuova), **coeff cammino vec![args] TARATO
12,0 ns per PASSAGGIO-DISPATCHER (±2,5; rev. #3: il census conta passaggi,
non alloc pure)** — fonda gli UB delle leve della stessa classe;
**coppia t9+ORM saldate @ s158**: WP t9 mediana 1,767 COMPATIBILE (N=6 pulite
6/6, banda_ON 0,058) · media 2,441–2,462 · ORM 7,090–7,141 col DOPPIO
giudizio pre-registrato (attesa-RF2 **NON RISOLTA lato peggiorativo** —
RETTIFICA rev. #1: Δ_norm [−0,77;−0,16] a cavallo di −0,293, la scaletta dal
solo dn_max e' emendanda a due estremi; RIF CONTESO; replica-AL1 RESTA APERTA:
phpr1 ictx segnalata 3ª finestra consecutiva — apertura primo-run ORM) · dbal
7,440–7,630; verdetti `wp159-harness/s159-*.out`; **coppia @ s159 DOVUTA →
S-160**); storico S-158 (pin **s158 92b0aea3** + server
**s158 f381b366**; **PROMOZIONE L-RF2 «tranche-2 slice
__reflect_*: 6 nomi (method_info·prop_details·prop_attr_new·class_real_name·
method_names·class_loc) da dispatcher Vec a slice, firme &[Zval]» con catena
piena rc=0 al t1**: A/B R=5 vs GEMELLO RICOSTRUITO 369ee345 (contenuto==pin
s157, 91 B/4 cluster LC_UUID+data+firma, regioni PRE-registrate) **D=+29,0
ns/iter su m-refl** (2 chiamate/iter: method_info cache-hit +
class_real_name; riconc. smoke 0,5<4,0 IN banda; **UB 13,8+rumore ECCEDUTO
⇒ sonda del surplus DOVUTA entro S-160, orologio §4**; cifra a REGISTRO
**[21;29]** — conferma post-pin +21,0, scarto>rumore da drift-tree non
quantificato; §4: direzione+meccanismo firmati, magnitudine NON ripartita;
estrapolazione famiglia a ~7 ns/chiamata, NON ai 14,5 misurati — az.rev.
S-158 #1-#2), 15 guardie ok NESSUN morso; disasm
run_loop Δ=0; dente loc 25779 pre-dichiarato; micro promo
5,4·5,5·4,8·4,3·3,3·2,5; conferma post-pin m-refl +21,0 segni 5/5; **coppia
t8+ORM saldate @ s157**: WP t8 mediana 1,754 COMPATIBILE (N=6 pulite 6/6,
banda_ON 0,008) · media 2,454–2,470 · ORM 6,952–7,093 col giudizio
ORACLE-NORMALIZZATO (emenda S-157 recepita nel criterio PRIMA della coppia:
Δ_norm [−0,22;+0,57] NON RISOLTA lato migliorativo, leg1 ictx segnalata) ·
dbal 7,477–7,486 companion; az.rev. S-157 #3-#5 operative (hash misurati,
sentinella LS, ramo morso-allo-smoke); verdetti `wp158-harness/s158-*.out`;
**coppia @ s158 DOVUTA → S-159**); storico S-157 (pin **s157 76787303** +
server **bdd32a98**; **PROMOZIONE L-AL1 «miss/autoload plumbing 0-alloc» t1
rc=0**: D=+22,0 su m-missload vs gemello c19079d3, UB 20,7 in banda, objchurn
morso REFUTATO a N=12M; coppia t7 1,795 · ORM 7,028–7,067, falso allarme HD2
arbitrato con replica3; dbal LC_ALL=C 3921/626 vs 3929/594; §3.25 a
catalogo; verdetti `wp157-harness/`); storico S-156 (pin **s156 42efea3e** + server **ef89630f**; **PROMOZIONE
leva HD2-hostcall «args-Vec host-builtin: arità ≤4 su slice, 6 nomi» t2 rc=0**:
A/B R=5 vs GEMELLO 2023cbb9 D=+16,0 su m-hostargs, UB 13,8+rumore in banda,
13 guardie ok, disasm bl +19; t1 STOP dente loc A4 → salita DICHIARATA
25742/6815; micro promo 5,4·5,5·4,7·4,2·3,2·2,6; census ORM post-CE1
RIFONDATO: chiamate class_exists ∈ [1,23M;2,46M], residuo miss/autoload
E ∈ [4,82M;6,05M], funnel CE1(b) apporzionato; verdetti `wp156-harness/`);
storico S-155 (coppia @ s154
SALDATA: WP t6 mediana 1,771 COMPATIBILE, banda_ON 0,022 record · media
2,456–2,510 · ORM 6,972–7,053 a CAVALLO di 7, Δ +0,41/+0,50 giù fuori
rumore, funnel istruito · dbal 7,385–7,422 · ce-count k=1 = args-Vec, CE1
0-alloc confermato · gdc ~636 chiamate NON pagante; verdetti
`wp155-harness/s155-*.out` + `s155-ce-istruttoria.md`); storico S-154 (**PROMOZIONE L-CE1 «class_exists lookup no-alloc
via LcKey» con catena piena rc=0**: A/B R=5 vs GEMELLO D=+22,0 ns/iter su
m-classexists (111→89, −20%), segni 5/5, riconc. smoke 2,0, conferma
post-pin +22,0 ESATTO 5/5; guardia backtrace morsa a 1 tick e ARBITRATA
(N=600k D=+0,0, refutata — emenda §6-bis); identità candidato a CONTENUTO
48 B LC_UUID+firma (emenda t2: pin non cold-riproducibile, banner mimalloc
da cache calda); batteria 1748/0/2 (cap mod.rs 25712 dichiarato), corpus
1412×2 ZERO flip, fixture 10/10 + fx-ce byte-id, ORM fail-set==16, hk 0E/0F;
**coppia t5 + ORM @ s153**: t5 COMPATIBILE 1,757 (N=5, banda 0,013) · media
2,450–2,485 · **ORM 7,051–7,073** (Δ+0,72/+0,77 GIÙ fuori rumore,
OLTRE-attesa BT2 dichiarata, magnitudine NON ripartita, oracle −1,4% di
giornata) · dbal 7,440–7,450; **FUORI-UB BT2 SPIEGATO**: k_new=13 ESATTO,
alloc 214–221 + hash/memcpy 46–52 ≈ D 266,7; testa hostcall rifondata
(debug_backtrace 6,149M==473k×13, residuo 60,87M Δ0,04%, class_exists 9,74M,
get_declared_classes 4,56M); coppia @ s154 DOVUTA → S-155; verdetti
`wp154-harness/s154-*.out`); storico S-153 = pin **s153 8370c257** + server
**s153 f030c6fc** (**S-153 = PROMOZIONE L-BT2 «debug_backtrace a chiavi
statiche + ZStr condivisi» con catena piena rc=0**: A/B R=5 vs GEMELLO
D=+266,7 ns/iter su m-backtrace (733→467, −36%), segni 7/7, riconciliazione
|0,0|, guardie 12/12, FUORI-UB sopra ⇒ sonda k post-leva dovuta (S-154);
batteria 1748/0/2 (s125+2 denti; cap dente aggiornati con salita dichiarata
mod.rs 25707 · host.rs 7661), corpus 1412×2 ZERO flip, fixture 10/10,
conferma post-pin +333,3 (5/5, tick 66,7), ORM fail-set==16 nomi, hk 0E/0F;
**L-TD1 borrow-unify teardown/sweep CADUTA a R=5 gemello** D=−3,3 vs soglia
4 con 4 borrow/iter rimossi certi ⇒ prezzo borrow in-contesto ≤~1 ns (mock
hot-hot 4,27–4,41 FALSIFICATO come prezzo; NO-GO A3c rafforzato); EMENDA
§7-bis: braccio A di ogni A/B = GEMELLO dal tree corrente; coppia WP+ORM al
pin s153 DOVUTA → S-154; verdetti `wp153-harness/s153-*.out`); storico
S-150 = pin **s150 cbbe7173** + server **18c27407** (**PROMOZIONE BT1 con catena piena rc=0**: corpus
1414→**1412** (flip PASS per NOME `debug_backtrace_limit`+`bug64239_2`,
mutato `debug_backtrace_options`), fixture **10/10** (fx-backtrace nel set),
guardie 8/8 a R=5 + disasm bl 6014 INVARIATO (incidente 17 riparato),
conferma m-backtrace D=+19000 5/5, **bilaterale NETTO 5,50×** (pavimenti
misurati); identità candidato↔braccio giudicato provata AL BYTE
(`wp150-harness/s150-identita-candidato.md`); **SCOMMESSA ORM VINTA
OLTRE-ATTESA: Δ +6,07/+6,70 s ⇒ ORM 8,370–8,427 → 7,104–7,149 · dbal
8,20–8,37 → 7,283–7,491** (attesa 0,8–3,1 = pavimento solo-alloc dichiarato;
BT1 unica leva s145→s150); coppia t4 6/6 pulite **MEDIANA 1,781 COMPATIBILE**
(primo giudizio a mediana); census controllo: spiegazione path CADUTA
(+3,2% s148↔s149 APERTO); **FR1 CHIUSA esito (b): +3,00 = prezzo STRUTTURALE
(+3180 B/+26 bl), nessun revert** — verdetti `wp150-harness/s150-*.out`;
storico S-149 = census per-NOME-builtin
dentro hostcall (identità Σnomi+unnamed==hostcall.n ESATTA ×2; repliche
0,000%) — l'other ha UN nome: debug_backtrace other=130,15M = 5,2× soglia
(73,95% dell'other; n=275,0M = 81,9% del tag; 11,7 GB); tutte le altre sotto
soglia anche per FAMIGLIA (__reflect_* 0,50×, array_* 0,32×); causa a
sorgente: options/limit IGNORATI (Doctrine chiama IGNORE_ARGS,2) ⇒ leva
BT1 A/B VINTO D=+19000 ns/iter (19733→733, −96,3%, segni 7/7, guardie 6/6,
fixture fx-backtrace BYTE-ID) — anche cura di FEDELTÀ; scommessa ORM
pre-registrata attesa ↓ 0,8–3,1 s (`wp149-harness/s149-decisione-bt1.md`);
prezzi PROPRI pair16 6,37–6,38 / pair48 11,21–11,27 ns (splitoff3 replica 5%
⇒ t3 sonda dovuta prima dell'uso); pop-diretti 1×–2× solo micro-judged,
args-Vec ~1× kill; promozione BT1 = S-150; verdetti
`wp149-harness/s149-{tr4,sonda-pair,ab-bt1}-verdetto.out`; coppia t3:
1,786–1,802 N=6 PRIMA finestra 6/6 pulite COMPATIBILE — su 3 finestre
conferma NON piena (t2 resta fuori, rett. rev.); banda_ON RIFONDATA
multi-finestra 1,722–1,823 (0,101, 17 coppie); revisione REGGE CON RETTIFICA
(incidente 17: guardie R=3 vs R=5)**; storico S-148 = census ATTRIBUZIONE per TAG (hostcall.other
165,6M > none 94,6M; kill hashbrown/pool-Frame/gc ai perimetri; t2 fuori
banda basso, `wp148-harness/s148-attrib-verdetto.out`); storico S-147 = coppia dbal+ORM RIMISURATA @ s145
(ORM 8,370–8,427 ↓ indicativa, dbal 8,20–8,37) + CENSUS UNICO ORM: KILL
KS-146-1 SCATTATO SUL PONTE (rett. rev. SEMANTICA) — ponte slot-load per
criterio (con LoadVarPushConst) 0,244 s < soglia 0,293 s (0,83×) ⇒ ZERO
codice sul ponte; famiglia FR1-ext 0,414 s (1,41×) RESTA micro-judged e
PROCEDE (concilio p.2); ponte+This 0,99× (confine da nominare in S-148);
TETTO canale movimenti 1,27 s ≈ 3,4% del gap (tetto su binario census);
take_str SAFE 0,029 s ⇒ TakeSlot chiuso a fortiori; repliche ESATTE,
367,55M == sonda; verdetto `wp147-harness/s147-census-verdetto.out`**; storico S-146 = coppia WP saldata
+ concilio B3 + guardia dimrmw: **regressione FR1 CONFERMATA +3,00 ns/iter su
m-dimrmw 10×, 5/5 ⇒ leva FR1 in ISTRUTTORIA — dimread resta;** verdetto
`wp146-harness/s146-ab-dimrmw-verdetto.out`; deliberato concilio VINCOLANTE
in `wp146-harness/concilio/sintesi.md`) (leva **L-FR1 «dim-read fuso a chiave costante» SPEDITA**:
peephole `PropGetSlot;PushConst(k);FetchDim` → `PropDimGetConst` in place,
PropIc condivisa, composito intatto come fallback per costruzione; hit =
elemento through-borrow, l'`Rc<PhpArray>` della prop NON viene clonato.
Giudice NUOVO m-dimread 3M iter: A/B R=5 **D=+16,7 ns/iter (60,0→43,3,
−28%) segni 7/7**, rumore drop-1 0, soglia 4; guardie m-dimrmw +0,01s=1 tick
DICHIARATO al limite di risoluzione, m-diminc/arr/prop verdi; disasm run_loop
A 71694 istr/5988 bl → B 72489/6014 agli atti; bilaterale: oracle 10 ns/iter
⇒ rapporto dim-read 6,0×→4,3×. Catena promo piena rc=0: batteria 1747/0/2
con inventario = baseline s125 + SOLO dente rczval dichiarato (t1 rosso su
ancora test census, t2 rosso su nomi VOLATILI compile-fail (line N) — emende
dichiarate agli atti), corpus 1414×2 nomi+contenuto+off↔on, fixture 9/9,
micro R=5, ORM 16 nomi==baseline, hk 0E/0F; conferma post-pin = IDENTITÀ di
byte col braccio B giudicato (ricetta riprodotta ×2). **Coppia WP al pin
nuovo DOVUTA → S-146 p.1**; verdetti in `wp145-harness/`) ·
storico S-142 (pin **s142 bba8a734** + server **eeb284b6**, leva **L-RD1
«teardown array inline» SPEDITA**: Drop for
PhpArray drena Packed/Hashed con match esaustivo, niente call per-elemento
sul cammino eseguito — disasm agli atti: il bl residuo è unwind-only; A/B
S-141 D=+5,0 AL BORDO con segni 7/7, conferma post-pin S-142 D=+5,0 al CENTRO
banda segni 5/5; catena piena rc=0 incl. ORM 3E/13F per NOME; invarianza
semantica VERIFICATA: parità Hashed A==B byte-id, nesting ~74–76k invariato;
divergenza PRE-esistente catalogata §3.22 unset-elemento/__destruct differito;
quota ORM dal census rd1_*: v. wp142-harness/s142-census-verdetto.out) ·
storico S-140 (HC1 «hint-check senza clone»: borrow-first
in coerce_or_check_hint, ramo Ref invariato; giudice NUOVO m-hintcall 7,3×
bilaterale, D=+6,7 su 6 check/iter, catena promo completa incl. gate ORM
3E/13F per NOME; census ORM: 35,6M hint-check ≈ 0,13% suite ⇒ HC1 non muove
Doctrine — REPERTO S-140: profilo SUITE = CHURN 32% vs DIMPROP 6%, 44% dei
clone INLINE da run_loop → filone TakeSlot. COPPIA WP @ s140 FATTA: on-only
1,765–1,777 N=6 COMPATIBILE — **banda_ON 0,033 CONFERMATA cross-finestra,
az.rev. S-139 #1 CHIUSA**; objmap 43,4 → piano gc-cycle-collector; dbal/ORM
riferimento S-139: dbal 8,15–8,23 ind., ORM 8,59–8,71) ·
metodo: user CPU, pavimenti per-binario, N per voce come indicato; criteri pre-registrati in
`wp125-harness/s125-criterio-{pair,mappa}.md` e `wp126-harness/s126-criterio-{orm,mappa2}.md`
(+ emenda S-127: **cifra canonica = NETTO-pavimento**, raw companion; gate contesa in ictx/s);
cifre dai verdetti `.out`. Regola di lettura: rapporti PER workload, MAI aggregato.

## Workload reali

| workload | rapporto phpr/oracle | N | note |
|---|---|---|---|
| **WordPress full-suite** | **S-155 t6 @ s154: on-only 1,758–1,780 (N=6 pulite) · MEDIANA 1,771 COMPATIBILE ∈ [1,738; 1,799] · banda_ON fondata 0,022 (record; mediane storiche t1=1,799 · t2=1,738 · t3=1,789 · t4=1,781 · t5=1,757 · t6=1,771); peak 6/6 BASSE (doppio livello NON riprodotto), deriva assente; verdetto `wp155-harness/s155-pair-verdetto-t6.out`** · storico S-150 t4 @ s150: on-only 1,745–1,800 · mediana 1,781 | **6/6 pulite** (t6) | S-150 t4 @ s150; parità 6/6 (solo `wp_is_stream #2`); peak t4 1774–1847 = MISTO dichiarato (nessuna firma); deriva: ρ_A=0,03 nessuna, ρ_B=−0,14 no accoppiamento; attesa BT1 su WP «piccola/nulla» RISPETTATA; verdetto `wp150-harness/s150-pair-verdetto-t4.out` (storico: t3 1,786–1,802 · t2 1,722–1,742 · t1 1,733–1,823) |
| **WordPress gruppo media** | **2,456–2,510 CANONICA user-only** (S-155 t6 @ s154, 6 gambe pulite; companion 2,429–2,495) | 6 | S-155 t6 @ s154 (storico t4 @ s150: 2,480–2,555; t3: 2,504–2,540) |
| **symfony http-foundation** (1854) | **2,547–2,559** (raw 2,55–2,57) | 2/lato | S-126; canonica sul CONTEGGIO diff 17 nomi = 0,92% ≤1% (≥3 nomi sono unit puri, NON famiglia `php -S` — emenda S-127); sys alto (I/O) |
| **symfony http-kernel** (1665 test) | **4,29–4,32** | 2/lato | parità 0E/0F; contesa ok |
| **doctrine/collections** (242) | **8,22 net** (raw 6,20) | 2/lato | S-126; INDICATIVA: oracle netto 0,09 s (denominatore sotto-scala); parità 0/0 |
| **doctrine/dbal** (3929, sqlite) | **7,385–7,422 net** (raw 7,165–7,200) | 2/lato | **S-155 RIMISURATA @ pin s154** (verdetto `wp155-harness/s155-orm-coppia-verdetto.out`; fail-set 10 nomi stabile; oracle segnalate ictx, user stabile; summary phpr vuota classe S-126 #3) · storico S-154 @ s153: 7,440–7,450 · **S-150 @ pin s150: 7,283–7,491** (verdetto `wp150-harness/s150-orm-coppia-verdetto.out`; oracle1 SEGNALATA ictx; fail-set stabile 10 nomi ==): **↓ da 8,20–8,37 @ s145 — companion della scommessa BT1 (deprecations dbal passano da debug_backtrace); direzione firmata, magnitudine non ripartita in proprio** · storico S-147 @ s145: 8,20–8,37 (verdetto `wp147-harness/s147-orm-rimisura-verdetto.out`; oracle1 SEGNALATA ictx; fail-set stabile 10 nomi == baseline ⇒ canonica): vs 8,15–8,23 @ s138 FERMO/lieve ↑ dentro il rumore · storico **S-139 @ s138: 8,15–8,23** (verdetto `wp139-harness/s139-rimisura-verdetto.out`; floors 0,06/0,19): fail-set stabile 10 nomi == baseline (0,25% ≤1% ⇒ canonica); vs 8,36–8,45 @ s134: **direzione ↓ INDICATIVA (az.rev. S-139 #5: ENTRAMBE le gambe oracle SEGNALATE al gate ictx — l'adiudicazione stesso-lato <1% è precedente S-135 ma applicata fuori criterio; declassata da «lieve ↓» a indicativo)**; summary phpr VUOTA (classe S-126 #3, fail-set dai .failnames) |
| **doctrine/orm** (3484 test) | **6,972–7,053 net — a CAVALLO di 7 (gamba migliore 6,97; «sotto 7» si dichiara a intervallo intero, rett. rev. S-155)** | 2/lato | **S-155 RIMISURATA @ pin s154** (stesso verdetto s155; parità 16 nomi==; phpr net [34,30;34,35] vs [34,76;34,80] @ s153 ⇒ Δ +0,41/+0,50 GIÙ FUORI RUMORE, OLTRE-attesa CE1 0,03–0,07: istruttoria `s155-ce-istruttoria.md` — meccanismo candidato = funnel resolve_class_autoload 11 siti, magnitudine NON ripartita) · storico S-154 @ s153: 7,051–7,073 · **S-150 @ pin s150 = GIUDIZIO SCOMMESSA BT1: VINTA OLTRE-ATTESA** (stesso verdetto; parità 16 nomi ==; contesa ok): **Δ phpr −6,07/−6,70 s vs s145 [41,60; 42,22]→[35,52; 35,53]** — attesa pre-registrata 0,8–3,1 s = PAVIMENTO solo-alloc (costruzione ~50 frame non prezzata, dichiarato); BT1 UNICA leva s145→s150 ⇒ direzione+meccanismo firmati · storico **S-147 @ s145: 8,370–8,427** (stesso verdetto; parità 16 nomi == baseline; leg1+oracle1 SEGNALATE ictx, leg2 PULITA 8,370): vs 8,59–8,71 @ s138 ⇒ **direzione ↓ INDICATIVA** (3 leve HC1+RD1+FR1 spedite in mezzo, nessun A/B proprio: magnitudine non ripartita, REGOLE §4); **denominatori KILL KS-146-1: soglia 0,7% = 0,293 s** · storico **S-139 @ s138: 8,59–8,71** (verdetto s139; oracle `memory_limit=-1` §3.14; parità 16 nomi == baseline; phpr1 ictx segnalata ma stesso-lato <0,2% ⇒ valida): vs 8,43–8,56 @ s134 ⇒ **FERMO/lieve ↑** — REPERTO pre-registrato (criterio p.6): le TRE leve dim-write s135→s138 (AP1+FD1+RMW) NON muovono la suite (l'attesa ↓ è FALSIFICATA: `$this->elements[$k]=$v` non è fetta misurabile del tempo ORM, o il perimetro FD1 lì non morde) ⇒ la prossima leva si sceglie sul profilo SUITE (churn clone/drop, insert/lookup — come già indicava S-135) |
| **composer install OFFLINE** | **1,863–1,891 net** (raw 1,820–1,847) | 2/lato | S-128 @ s127b, PRIMA misura col numeratore vivo (cure ondata-2); composer ESTRATTO, vendor_ok bilaterale, contesa ok (ictx/s); floors 0,07/0,06; sys≈user (~2,3 s/lato) ⇒ **cifra user-only NON confrontabile col full (user+sys): su user+sys sarebbe ~1,3** (rev. S-128 az.5); residuo phpcs config-set (§3.19-quinquies); verdetto `wp128-harness/s128-compoff-verdetto.out` |

## Micro-categorie (R=5, pin s156 dalla catena promo HD2-hostcall; tappa ≤3×)

| arith | prop | calls | str | arr | re | hintcall | dimread |
|---|---|---|---|---|---|---|---|
| 5,4 | 5,5 | 4,7 | 4,2 | **3,2** | **2,6** ✅ | **7,3** (S-140, non rimis.) | **4,3** (m-dimread; FR1 istruttoria CHIUSA S-150: +3,0 su dimrmw10 = prezzo strutturale, nessun revert) |
(rif s153: 5,5·5,5·4,7·4,3·3,2·2,6 · promo s154 in NEXT: 5,5·5,6·4,8·4,3·3,3·2,5)

m-backtrace (giudice BT1/BT2): phpr **733→467 ns/iter** @ s153 (D proprio
+266,7 firmato); bilaterale ~3,0× DERIVATA (oracle rif s150 ~133 non
rimisurato — direzione, non cifra canonica).

(S-145: tutte le voci entro 1 tick dai rif s142 5,5·5,6·4,7·4,2·3,2·2,6 —
la fusione non tassa i freddi; rif storici s138: 5,6 · 5,6 · 4,8 · 4,3 ·
3,2 · 2,6.)

RMW (giudici leva S-138, A/B + conferma post-pin): **m-dimrmw 320→146,7
ns/iter (D=+173,3)** · **m-diminc 270→113,3 (D=+156,7)**.
**HC1 (S-140, pin s140)**: m-hintcall6 D=+6,7 — **QUALIFICA rev. S-140: tick
di quantizzazione 3,3 ⇒ 6,7±3,3; evidenza portante = conferma post-pin +10,0
+ segni 5/5 (t2) e 5/5 (conferma)**; ~1,1–1,7 ns/check; census ORM 35,6M
check ≈ 0,13% suite (guadagno reale = prezzo×conteggi: HC1 non muove le
suite — REPERTO; az.rev. #5: census preventivo della quota PRIMA di spedire).

calls: la (*) di s127 è SCIOLTA in S-129 (phpr netto IDENTICO 2,14 s; si muove
solo il denominatore oracle 0,43–0,44). re 2,5/2,6 = run-to-run del denominatore.

Allocazioni/iter vs oracle: arith/prop/calls 0=0 · **str 2,00=2,00 (PARITÀ, S-125)** ·
arr 2,05≈2,03 · re 7,00 vs 5,00 (+2, apertura per NOME).

## Micro-ORM (S-136 sul pin s136 POST-leva-FD1 — verdetto s136-submicro; evalcls/refl da S-126)

| evalcls (compile/classe via eval) | refl | objchurn | └ objalloc | └ objdatains | └ objdropdef | └ objallocni | └ objmap |
|---|---|---|---|---|---|---|---|
| **316,9** (2,38 ms vs 7,5 µs) | **42,4** | 7,0→**6,7** (1180,0 ns, collaterale FD1 −86,7) | **6,4** (810,0) | 6,5→**5,9** (963,3 ns, −96,7 = FD1; riconc. A/B 13,4 ≤ 26,7) | **7,5** | 8,1→**7,9** (736,7; l'osservazione +13,3 di S-135 rientra) | **11,7** (116,7 ns, fermo) |

Profilo ORM phpr (indizio unilaterale): churn visibile multi-% (Zval clone/drop, slot_of,
gc_note/sweep/collect_cycles, insert/lookup, malloc/free); compile ≤~1% leaf, reflection <0,5%.

## Lettura (direzione+indizio, NON attribuzioni firmate — REGOLE §4)

- Il gap **cresce con la densità di lavoro-motore puro**: WP ~1,8 ≈ compoff ~1,9 ≪ hf 2,6 ≪ hk 4,3 ≪
  dbal 7,4 ≈ ORM 7,1 (cifre net, S-150). WP, compoff e hf sono diluiti da I/O; le suite object-dense mostrano il soffitto.
- **dbal 8,6 conferma ORM 8,5 senza mock-eval pesante** ⇒ il driver è il lavoro-oggetti, non il
  sentiero compile: coerente con l'istruttoria (compile ≤1% leaf nel run reale).
- **L-OL1-F1 «stampo» SPEDITA (S-127, pin s127 834f5e01)**: template Props per classe,
  default COW — objalloc −20,4% (7,7×), churn 8,9×. **S-129: MODELLO DEL TEMPO seg.3
  CHIUSO** (s129-modello-tempo.md): statement Field* ≈300–340 ns QUASI INVARIANTE per
  forma (oracle 23–37; locale 170); torta per-passo (chiusura 96%): **E−E2
  (dispatch+prop_step) ~155 ns (52%)** — residuo per sottrazione; l'attribuzione
  «resolve-per-NOME» è INDIZIO (profilo+disasm), sonda diretta E1a dovuta in S-130
  (rev. S-129) · preludio byref/indirect/lazy ~73 ns (25%, sondato E corroborato
  dall'A/B F4) · walk interno 48 (16%);
  i 2 alloc residui = n.clone() del nome in byref_hook_root+field_lazy_root (census
  22/22). **F4 «prelude-gate» SPEDITA S-130** (criterio emendato pre-registrato:
  rumore trimmed drop-1 simmetrico + bande fondate 6,7/6,7/13,3): smoke +80,0 →
  R=5 D=+80,0 vs soglia 16,7, direzione 14/14 cumulata, promozione piena rc=0 →
  **pin s130 0fdf1c49**. **Sonda E1a S-130** (s130-e1a-lettura.md): controllo
  objalloc k=4 svela le resolve del CTOR (criterio p.2 EMENDATO a verbale S-131).
  **MODELLO PROP_STEP S-131** (s131-propstep-lettura.md, chiusura 93–94%): E−E2
  166,9 = prop_step interno 130,7 (guardie 49,4 · defer 37,0 · key+op 34,3 ·
  borrow 1,5 · altro 8,5) + dispatch 36,3; resolve statement 40,3 su 5 siti
  enumerati per NOME (3× prop_key + prop_key_read + prop_indirect_guard ≈0);
  ctor 70,8 (17,7/resolve, più care). **E1-KO «resolve-once» SPEDITA S-131**
  (criterio pre-registrato: smoke +45,0 → R=5 D=+23,3 vs soglia 13,3, guardie
  9/9, promozione rc=0) → **pin s131 ff66cb84**. **L-LO1 «lookup-once» SPEDITA
  S-132** (criterio pre-registrato con soglia az.rev. #1 = spread-batch 10,0:
  smoke +23,3 → R=5 D=+20,0, riconciliazione 3,3 in banda e dentro UB 30,
  guardie 9/9, promozione rc=0) → **pin s132 6af6e497**: UN accesso alla
  props-map nel ramo non-leaf (slot WP-29 dalla resolve, fallback by-name).
  **Leva ctor «resolve-once» SPEDITA S-133** (sonda a soli conteggi conferma lo
  split 2+2 magic_applies/fallback-PropSet; criterio pre-registrato soglia
  max(4, drop-1, spread-batch 6,7): smoke +36,7/+31,7 → R=5 objalloc D=+46,7
  [DICHIARATO fuori UB 35,4, +11,3 non ripartita] + objdatains D=+30,0,
  guardie 8/8, promozione rc=0) → **pin s133 c87439a9**: UNA resolve hoistata
  post-hook in prop_set_entry, condivisa dal magic-check e dal blocco
  key/slot/IC; hooked-set resta a zero resolve.
  **Leva «IC non-plain» SPEDITA S-134** (eccedenza s133 prima NOMINATA dal
  disasm — seconda lookup dipendente back-to-back, `s134-eccedenza-lettura.md`;
  criterio pre-registrato coi componenti non prezzati DICHIARATI per nome,
  soglie spread-batch s133 26,7/13,3: smoke +150,0/+130,0 → R=5 objalloc
  D=+136,7 + objdatains D=+133,3, riconciliazioni in banda, guardie 8/8,
  promozione rc=0 su catena a 9 gate) → **pin s134 61896da1**: bit NP/TY nei
  2 bit alti dello slot del PropIc; fill dal cammino pieno SOLO con fatti di
  classe provati (no set/virtual-hook, no `__set` — load-bearing per il
  typed-unset —, asym ok, non readonly, key==name); il hit salta resolve +
  magic-probe + asym/readonly/hook-lookup e MANTIENE coercizione typed,
  presenza slot e typed_refs per-scrittura.
  Residui NOMINATI: dispatch 36,3 · contabilità del non-resolve residuo da
  RI-DERIVARE sul pin s134 (il hit IC copre parte dei ~60 ns/statement) ·
  cammini non cacheabili per costruzione (readonly, private mangled, `__set`
  presente, slot assente).
  **Leva «AP1 fast-path» SPEDITA S-135** (scelta dai numeri: bisezione objmap
  → canale dominante = macchineria dim-set 183,3 = 77%, poi modello del tempo
  AssignPath su m3: arm 66,8, path_op 52,2 = 78%, walk-plumbing 38,4,
  chiusura 86% INCOMPLETO dichiarato; criterio pre-registrato con UB
  FALSIFICABILE 47,7 = prezzi misurati; A/B r1 rc=5 agli atti — guardia
  objalloc su banda sotto-fondata — emenda rev. S-112: guardie alla formula
  del giudice; r2 R=5 objmap D=+56,7 ≤ 57,7, riconc. smoke 6,7 ≤ 10,
  guardie 9/9, promozione rc=0) → **pin s135 6518a1e1**: nel braccio
  AssignPath, caso 1-chiave/no-append/base GIÀ Array = specializzazione
  letterale del cammino pieno (coerce → make_mut → set_returning_displaced →
  gc_note → push), tutto il resto al pieno invariato; sonda dim-write residua
  (objdatains 2 resolve/iter sul cammino prop-dim, fuori perimetro) a
  catalogo. Sonda conteggi S-135: eccedenza S-134 ATTRIBUITA (5 canali 2→0,
  depr 0→0 falsificato, layout escluso; `s135-eccedenza-chiusura.md`).
  **Leva «FD1 fast-path dim-write su prop» SPEDITA S-136** (dal reperto sonda:
  2 resolve/iter su `$e->data['k']=$i` → lowering `FieldAssign{[Prop,Index]}`
  verificato col dump; modello tempo FieldAssign su m-dimwrite, chiusura 94%:
  arm 118,2 = walk_driver 37,2 · leaf 18,9 · plumbing 17,6 · prop_step_altro
  14,4 · guardia 11,3 · resolve 6,7 · dispatch 7,0 · pop 4,5; criterio con UB
  falsificabile 69,6 = somma canali bypassati; A/B R=5 objdatains D=+83,3,
  soglia 13,3, riconc. smoke 1,6, **FUORI-UB +0,4 DICHIARATO** — eccedenza
  +13,7 non ripartita, sonda dovuta; guardie 10/10, `re` morsa allo smoke
  rientrata a R=5 col drop-1 vero; promozione rc=0) → **pin s136 1e14793e**:
  cella PropIc su `Op::FieldAssign`, fast path `[Prop,Index]` con
  `field_write_walk` RIUSATO sul child Array (leaf identico per costruzione)
  + driver-loop replicato; fill dal ramo F4 a esito Ok coi fatti di classe
  (slot key==name, non readonly, asym ok; hooks esclusi da F4). Perimetro
  fuori: child Ref/Str/assente, nkeys≠1, unset-prop, readonly, asym-negata.
  **Eccedenza FD1 CHIUSA (S-138)**: disasm refuta l'artefatto-inlining (bl +63
  = timer); sonda arm-only v2 (inerzia 0,000) dà arm 51,9 pulito; **A/B pin
  s135↔s136 sul giudice del modello: D_mdw 63,3 vs UB 69,6 IN BANDA** — la
  «eccedenza» era aritmetica CROSS-GIUDICE (D 83,3 misurato su objdatains, arm
  su m-dimwrite); coerenza-arm 51,9+63,3=115,2 ≈ 118,2. Dim-write SBLOCCATO.
  **Leva «FD1-ext RMW» SPEDITA S-138** (criterio s138-criterio-rmw.md: cella IC
  su FieldAssignOp/FieldIncDec; fast = admission FD1 + peek entry + op silente
  {Add,Sub,Mul}×{Long,Double} + field_write_walk riusato; fill dal ramo piano
  via field_prelude_skip; il pieno pagava DUE walk + preludio. A/B R=5:
  m-dimrmw D=+173,3, m-diminc D=+156,7, guardie 7/7, objdatains ±0,0;
  fuori-modello +110 ATTRIBUITO con sonda monobinaria kill-switch (scarto
  +3,7/17,3: arm_full 266,9 − arm_fast 89,9 = 177,0 ≈ D); promo rc=0 + conferma
  post-pin in banda 5,0/5,0) → **pin s138 fa17dabd + server a9aded45**.
  Aperture per NOME: FieldRead/dim-read IC (famiglia sbloccata) · divergenze
  RMW del pieno (undefined-key, float-key, str-increment, overloaded-notice) ·
  14% modello AssignPath (86%) · **objmap «valore-oggetto» 43,4 ATTRIBUITO
  (S-137, census) al round-trip GC nota→sweep→demozione — leva note-time
  REFUTATA (precedente WP-21), cura = piano gc-cycle-collector**.
- Aperture per NOME: `evalcls` **316,9×** (cliff compile-per-classe; serve strumento di densità
  prima di ogni leva) · `refl` **42,4×** · re +2,00 alloc/iter.

## Voci da misurare (per NOME)

lexer/inflector/event-manager · wp-cli · PHPUnit-self ·
DBAL: catalogare i 10 nomi Portability/parser-unicode in PHPR_DIVERGENCES.
