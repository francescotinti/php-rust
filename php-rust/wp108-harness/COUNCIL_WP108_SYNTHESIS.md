# SYNTHESIS Concilio WP-108 — VINCOLANTE (ratifica ordine S-107)

Fase 1 (9/9, `verbali/`) + fase 2 (team attribuzione-misura, semantica-finestre,
pin-census-server). 2026-08-07, chiusura S-106. Eredita i veti §4 WP-107 (in vigore).

## 1. FONDAMENTALI (feedback-council-fundamentals-first)

- **Avanzamento OGGETTO in S-106**: leva H-A1 PROMOSSA — arith **12,4→11,6**
  con **Δ=+7,0 ns/iter 5/5 ATTRIBUITO** (ABAB R=5, pin eb555106); prop
  **11,5→10,6** nella formula RATIFICATA: «direzione+meccanismo firmati dal
  dump, magnitudine NON ripartita». Finestra ST confermata 3/3 (RMW ≡
  ZEND_ASSIGN_OP; unwind/slot identici; nessuna rientranza nuova). Server
  **dde2a64d GRADATO PIENO** (rito D-16, rc=0 voids=0 ×2) ma **PRE-leva H-A1**:
  cifre server NON attribuibili senza regrade. Corpus **1417 fermo**.
- **Contatore misura WP**: ultima full/media = **WP-105 (1 sessione fa)**;
  citabile WP-102 full 1,894 (D-6 salda). **Coppia WP in S-107 = DOVUTA**: il
  «se» del provvisorio CADE — voci S-105 fuori banda (full-off 1,947; media
  2,697/2,734) aperte da 2 sessioni, condizione del punto 4 già vera.
- **Rischio d'oggetto più trascurato**: l'**eccesso ORFANO ~5,7 ns/iter su
  prop** (Δ implicito ~12,7 vs 7,0 misurati) — ciò che ha mosso prop oltre
  H-A1 non ha né nome né governo: un co-fattore non attribuito può gonfiare o
  erodere in silenzio le prossime promozioni. Cura nominata: S-107-D-2.

## 2. DIRETTIVE RATIFICATE S-107-D-n (12 T-AM + 8 T-SF + 11 T-PC = 31 ID → 26)

### (a) VINCOLANTI

1. **S-107-D-1** — Formula unica prop: «direzione+meccanismo firmati dal dump;
   magnitudine NON ripartita (~12,7 vs 7,0 ns/iter, eccesso ~5,7 orfano)».
   Vietato «H-A1 vale −0,9»; cifra = micro sul pin. [T-AM-108-1]
2. **S-107-D-2** — Retro-A/B prop (~10′, punto 1): ABAB R=5 sul giudice prop coi
   due stash phpr-s105 (d4d0fa52) vs phpr-s106 (eb555106); converte la direzione
   in magnitudine attribuita o smaschera il co-fattore. [T-AM-108-2]
3. **S-107-D-3** — Clausola ±0,4 emendata DICHIARATAMENTE: «fermo entro ±0,4
   OPPURE movimento migliorativo con beneficiario nominato dal dump, registrato
   SOLO in forma D-1»; sanatoria S-106 scritta. [T-AM-108-3]
4. **S-107-D-4** — Finestra dichiarata TRASVERSALE ⇒ l'A/B misura ANCHE le
   categorie beneficiarie nominate, stesso ABAB; nel criterio PRIMA del run. [T-AM-108-4]
5. **S-107-D-5** — «L'inliner ripaga i due dispatch» declassata a osservazione;
   −128 B narrato finché manca il diff taglie per-simbolo; conto per-target
   COMPLETO obbligatorio al prossimo admission. [T-AM-108-5]
6. **S-107-D-6** — «Micro su hash₁» = proxy arith-only, NON R-1 pieno; cinque
   categorie restanti in voci aperte; ogni proxy futuro dichiara il perimetro
   che NON copre. [T-AM-108-6]
7. **S-107-D-7** — Ledger forme monomorfe al primo pin S-107: hit-count census
   per-forma (giudici+corpus), forme ~0 fredde; sveglia (+8 forme da S-104 O
   run_loop +4 KB) ⇒ istruttoria cold-partition/outlining o pivot
   superistruzioni-da-census; ogni istruttoria pre-registra il budget forme. [T-AM-108-7]
8. **S-107-D-8** — **Coppia WP DOVUTA** (il «se» cancellato): chain v2 con
   launcher gamba-0 TRE-PIN fail-closed da file (phpr+server+oracle,
   `PIN_ORACLE_ATTESO.txt`); identity dichiara «dde2a64d PRE-H-A1»; NESSUNA
   cifra server dal pair. Istituita KS-GR-108-1: voce fuori banda senza rerun
   2 sessioni = debito datato; alla terza blocca la leva. [T-AM-108-8 ∘ T-PC-108-4]
9. **S-107-D-9** — Leva S-107: candidata = **IncDecSlot+Pop** (stessi helper,
   +1 braccio); **Sweep per-iter VIETATO come peephole** — solo istruttoria
   propria con criterio GC dedicato + gate d'ordine-free; il criterio DICHIARA
   l'accoppiamento §3.11. [T-AM-108-9 ∘ T-SF-108-5]
10. **S-107-D-10** — Doc reg_lower.rs riallineato nei TRE siti (r.20-21, r.190,
    lista v3 + eccezione ST) col testo T-SF-108-1: «LoadVar (warning) never
    folded; LoadSlot (silent) foldabile SOLO nella finestra ST (r.475-482), che
    eredita la disciplina silente di read_slot; silenzio su undef = divergenza
    §3.11, NON contratto; equivalenza al tris ⇐ il pop non muta gli slot; ogni
    nuova finestra nomina QUALE disciplina eredita». Stesso commit: emendare il commento "contratto". [T-AM-108-12 ≡ T-SF-108-1]
11. **S-107-D-11** — Dente throwing-store famiglia Dst (difetto EREDITATO, non
    di H-A1): typed-ref che lancia nello store, try/catch, __destruct,
    flag-on/off/oracle; a divergenza arbitro = oracle, cura MAI de-fusione. Punto 1. [T-SF-108-2]
12. **S-107-D-12** — Cura D-12 COMPLETA (chiusa PRIMA del punto 2): gamba D-10
    a debug_assertions O census con assert==0; PIÙ contatore release
    incondizionato arm ArgPlace (`argplace_decay_hits`) **+ LETTORE**: riga nel
    dump zvalcensus e gate «atteso 0» a fine batteria/corpus, stesso commit.
    «Backstop saldato» SOLO a gate verde. GA_ARITY corretta ai DUE siti
    (bind_params + direct-bind); manifest D-5 dichiara la composizione; il pin di
    parità resta silenzioso sul degrade: dichiararlo. [T-SF-108-3 ∘ T-PC-108-8]
13. **S-107-D-13** — Dente drop-order = PREREQUISITO di admission della leva
    D-20 (metà-Zend): leva su decay/call-path senza dente = VOID; nessun
    allargamento del direct-bind col commento D-9 declassato. [T-SF-108-4]
14. **S-107-D-14** — Kill-switch: estensione di BinarySTDst (forma value, rhs
    const, spelling LoadVar) senza criterio proprio pre-registrato E D-11
    verde = VOID; una finestra che assorbe op CON effetti (LoadVar-warning,
    FetchDim) NON eredita il verdetto H-A1. [T-SF-108-6]
15. **S-107-D-15** — Funnel: ripristinare UN positivo BinaryDst nel `{main}`
    (`$s = $s + $i + $i;`) + negativo «nessun LoadSlot/Swap residuo nel `{main}` ON». Test-only, punto 1. [T-PC-108-1]
16. **S-107-D-16** — Dicitura: «BUILD EMENDATA post-A/B — file enumerati;
    ri-validata sul pin (batteria+corpus+micro su hash₂; hash₁≡hash₂ Δ≤0,4)».
    «Churn» riservato al relink puro a sorgente identico. [T-PC-108-2]
17. **S-107-D-17** — Il grado lega un QUADRUPLO (hash server, phpr, oracle,
    HEAD): dde2a64d = **PRE-H-A1**; ogni CIFRA server S-107 pretende rebuild
    ricetta @ HEAD S-106 + regrade D-16 PRIMA della lettura; citarlo accanto a
    eb555106 senza regrade = VOID. Il regrade NON è prerequisito del pair
    (server = identità d'ambiente): si esegue SOLO se un fronte server con cifre
    si apre — nessun rebuild speculativo. Pre-flight riscritto: «server dde2a64d
    GRADATO @ c7b6eb2 (PRE-H-A1), stash php-server-s106; primo atto di ogni
    fronte server con cifre = rebuild+regrade». [T-PC-108-3 ∘ T-PC-108-5]
18. **S-107-D-18** — Registro dde2a64d: coppia commit (server @ c7b6eb2 vs pin
    phpr @ d569a56) + parity-null per NOME; D-6 emendata ratificata: fail-set
    baseline pinnato PER NOME su file, «vuoti» = diff vuoto contro QUEL file,
    eccezione congelata {wp_is_stream #2}, ogni crescita = voce nuova. [T-PC-108-6]
19. **S-107-D-19** — Attese stack-census S-102 DECADUTE dalla fusione:
    ri-registrazione per NOME (arith E prop) nello STESSO run census D-5 —
    collocato DOPO la cura §3.15, sul binario post-fix (CF-AM-1 PRO-GREGG: il
    fix D-13 riscrive push_call_args, il sito che il census strumenta; vincolo
    Bak salvo: il census precede comunque ogni lavoro calls). Stesso run:
    manifest GA_ARITY due-siti + lettore D-12 (nessuna finestra nuova).
    Confronto col 23/iter storico = VOID. [T-PC-108-7 ∘ T-AM M-1]
20. **S-107-D-20** (qui promossa) — TIMEBOX esplicito sul punto 1: se i denti
    sforano, la leva del punto 3 NON salta (regola di ritmo utente); matrice
    ST per NOME (`-=` su prop, `.=` vs ConcatAssignSlot, jump-target in
    finestra, fixture diagnostic-safe) = backlog NOMINATO, non blocco. [T-PC-108-10, A-KL-108-1]

### (b) Raccomandazioni

- **S-107-R-1** — arr: registrare il SEGNO oltre alla banda; drift monotono
  N≥3 sere oltre metà banda = trend ⇒ istruttoria. [T-AM-108-10]
- **S-107-R-2** — Banda-layout: un contrasto (4,99/4,97) = UN punto, per N≥3
  solo stesso-protocollo; disambiguare «due punti colti» in NEXT_SESSION;
  terzo punto su giorno distinto. [T-AM-108-11]
- **S-107-R-3** — Sigillo trivial-arms: dichiarare SOLO il provato («no-Drop/
  bit-copy»); lista costruttori co-locata col fast path. [T-SF-108-7]
- **S-107-R-4** — Opportunità SOLO con criterio proprio: guardia binary_fast
  senza-clone (residuo 11,6); censimento gc_note famiglia *Dst. [T-SF-108-8]
- **S-107-R-5** — Feature-gate statics GA_ARITY **e** GA_ARGPLACE_DECAY (R-5
  ESTESO) senza churn del pin; se churna, dicitura D-16. [T-PC-108-9]
- **S-107-R-6** — Igiene: rc cargo check in .out; retro-verifiche lettura-only
  dichiarate; riscrivere «parity-null provato dalla compilazione» (non prova
  il codegen-null del runtime); riapertura footprint ⇒ primo atto re-baseline
  peak (1942/1990 non citabili). [T-PC-108-11]

## 3. ORDINE S-107 RATIFICATO (sequenza 1-5 confermata, testi emendati)

1. **Denti — TEST-ONLY, con TIMEBOX D-20**: funnel D-15 · throwing-store D-11
   · dente D-10 con gamba debug/census ESPLICITA + **cura D-12 completa
   contatore+lettore, chiusa PRIMA del punto 2** · retro-A/B prop coi due
   stash (D-2) · terza mutazione OBS-8 + mutante leak-parziale fx20 (eredità)
   · dente drop-order «prerequisito D-20» (D-13). **1-bis, stesso giro di
   commit**: riallineo doc reg_lower.rs (D-10). ⚠️ Il census hit/miss ESCE dal
   punto 1 (→ 2-bis). Sforo ⇒ backlog per NOME, la leva NON salta.
2. **Fedeltà §3.15** (cura D-13 WP-107, testa non negoziabile): Zend-esatta ≥
   vslot, fx21 gamba dinamica, gate ORM/hk, il fix CITA i fail (attesa
   1417→1415), **golden fx21 aggiornato NELLO STESSO commit**; 1417→1415 NON
   tocca i denominatori server 413/3508. Poi get_gc se la finestra regge.
   **2-bis — census D-5, passo autonomo POST-fix**: hit/miss su binario pulito
   + manifest + rerun arità + GA_ARITY due-siti + stackcensus arith E prop +
   lettore argplace_decay_hits — UN solo run (D-19). Siti calls SOLO da qui.
3. **LEVA = IncDecSlot+Pop** (D-9; Sweep VIETATO come peephole). Criterio
   PRIMA (D-3 WP-107) che DICHIARA: accoppiamento §3.11 · beneficiari
   trasversali stesso ABAB (D-4) · budget forme (D-7, ledger al primo pin).
   Admission: conto per-target COMPLETO (D-5) + taglia run_loop; early-stop;
   PIN in chiusura. Protetta dal timebox D-20.
4. **Coppia WP — DOVUTA** (D-8, il «se» CADE): chain v2 con sanature D-16,
   launcher gamba-0 TRE-PIN fail-closed da file, identity con hash oracle e
   riga «server dde2a64d PRE-H-A1». Lettura SOLO con bande KS-GR-107-3; voci
   S-105 fuori banda: rerun, MAI bisect — orologio KS-GR-108-1 attivo. Nessuna
   cifra server dal pair; regrade SOLO prima di cifre server (D-17).
5. **Igiene contatori e code**: feature-gate GA_ARITY + GA_ARGPLACE_DECAY
   (R-5) senza churn · banda-layout terzo punto giorno distinto (R-2) · segno
   di arr (R-1) · igiene .out (R-6).

**Pre-flight S-107**: riga server secondo D-17; resto invariato (pin eb555106 @
HEAD S-106, corpus 1417 ×2 sul pin, flag-ON, debug/ rimossa, uploads sotto
guardia, nessuna run in volo).

## 4. VETI / NON-RIPROPORRE nuovi (KS-108; i §4 WP-107 restano in vigore)

- Estensione di BinarySTDst senza criterio proprio pre-registrato + dente
  throwing-store verde = VOID; op con effetti non ereditano H-A1 (KS-HO-108-1, KS-MA-108-1).
- Direct-bind allargato col commento drop-order declassato = VOID (KS-HO-108-2).
- Contatore senza lettore non è un dente: senza dump+gate è documentazione (KS-LE-108-1, KS-MA-108-2, KS-KL-108-1).
- Magnitudine ripartita a un beneficiario senza A/B proprio = VOID (KS-BA-108-1 ≡ KS-GR-108-2).
- Voce fuori banda senza rerun oltre 2 sessioni = debito datato; alla terza BLOCCA la promozione della leva (KS-GR-108-1).
- «Churn» quando la build emendata include sorgente = mislabel vietato: dicitura D-16 obbligatoria (KS-KL-108-3, KS-PE-108-2).

## 5. RICEVUTA

SYNTHESIS WP-108 RATIFICATA: **20 vincolanti S-107-D-1..20 + 6 racc. R-1..6**
da 31 direttive di team (fusioni: doc reg_lower D-10 = T-AM-108-12≡T-SF-108-1;
cura contatore+lettore D-12 = T-SF-108-3∘T-PC-108-8; coppia tre-pin D-8 =
T-AM-108-8∘T-PC-108-4; leva/Sweep D-9 = T-AM-108-9∘T-SF-108-5). Ordine 1-5:
sequenza confermata; census D-5 ricollocato DOPO §3.15 (2-bis, PRO-GREGG, stesso
run T-PC); coppia WP DOVUTA; Sweep vietato come peephole; timebox anti-fame-leva
promosso vincolante (D-20). Nessun dissenso residuo.
