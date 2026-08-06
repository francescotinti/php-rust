# Team METODO-MISURA — Fase 2 Concilio WP-107

**Team**: metodo-misura (disciplina di misura, estimatori, bande, census)
**Membri**: Hejlsberg (RELATORE), Gregg, Leijen
**Data**: 2026-08-07 (S-106)
**Fonti**: verbale-4-hejlsberg.md · verbale-9-gregg.md · verbale-7-leijen.md

## 1. Composizione

### Convergenze (equivalenze dichiarate, ID canonico)

- **KS-HE-107-1** (canonico) ≡ KS-GR-107-2 ≡ nucleo di R-LE-107-2: un
  contrasto tra due A/B distinti — a maggior ragione tra binari a layout
  diverso — è un estimatore post-hoc: indizio con confondenti nominati
  (Gregg nomina reverse+transito bind_params), MAI cifra. Il «~37 ns»
  e il «~9 ns alloc+free» sono VOID come numeri; la DIREZIONE
  (forma 1 sotto, forma 2 sopra) resta firmata.
- **A-HE-107-2 ≡ A-GR-107-1**: stessa correzione documentale
  (riscrivere «37 ns» in report/NEXT_SESSION/memoria come divario
  direzionale, «terza conferma» declassata a indizio). Canonico: A-HE-107-2.
- **KS-LE-107-2** generalizza il punto: nessuna banda ottenuta prezzando
  componenti fa da trigger STOP o bisect (modello positivo: banda R=7
  34,64, distribuzione MISURATA). Complementare, non ridondante, a
  KS-HE-107-1: quello vieta la cifra, questo vieta l'USO della cifra.
- **A-LE-107-5** coerente: «mimalloc TL quasi gratis» = ipotesi di lavoro
  con due conferme indirette e zero misure in isolamento — licenzia lo
  scarto di leve micro-costo, non cifre.
- Bande pre-registrate: KS-GR-107-3 (coppia WP: formula f̂ e bande
  scritte PRIMA dei ratios) e A-LE-107-2 (H-C3: segno+soglie, magnitudine
  orientativa) sono la stessa disciplina applicata a due oggetti.

### Conflitti risolti

1. **Rango «capitale»**: Hejlsberg dichiara R-HE-107-2 CAPITALE;
   Gregg e Leijen «nessuna capitale». Risoluzione: la SOSTANZA è
   identica nei tre verbali (37 VOID come cifra); il rango di Hejlsberg
   punta al fatto AGGRAVANTE che la cifra è già entrata in
   WP_SESSION_105 come «Scoperta 1». Il team adotta la correzione
   documentale come VINCOLANTE (T-MM-107-1); la qualifica di capitale
   resta a verbale Hejlsberg senza effetto direttivo ulteriore.
2. **Admission «senza flip»**: Gregg la dà per rispettata
   (KS-BA-106-1), Hejlsberg obietta che gli aggregati non escludono
   flip compensati. Risoluzione: NON è contraddizione ma split
   admission/licenza — gli aggregati bastano per AMMETTERE al pieno,
   non per CITARE componenti; lo stesso R-GR-107-1 già vieta la cifra.
   Si adotta lo split di Hejlsberg (T-MM-107-4).
3. **f̂ di KS-GR-107-3 vs KS-LE-107-2**: la formula usa il −14% della
   leva (una componente). Attrito apparente risolto: f̂ è PRE-registrata
   (scritta prima della lettura) e serve alla sola LETTURA/attribuzione
   della coppia, mai da trigger STOP/bisect — le letture fuori banda
   ordinano rerun, non bisect. Conforme a KS-LE-107-2 per costruzione;
   la compatibilità va dichiarata così nella SYNTHESIS (T-MM-107-6).

### Aperto (nessuna sedia lo chiude)

- Banda-layout N=1 (0,67): si accresce gratis col churn (T-MM-107-8),
  obiettivo N≥3 entro due sessioni.
- «icache-bound» resta ipotesi N=1: solo razionale di targeting finché
  mancano i contatori L1I (T-MM-107-3).
- Generalizzazione del 73,1% al carico WP: il campione è UNA fetta
  (functions.php, 214 test); rerun su binario pulito (T-MM-107-5) e
  perimetro Bak (copertura ≠ arità, fuori tema di questo team).
- Misura in isolamento della coppia TL: backlog, NON aprire (Gregg).

## 2. Direttive composte

### (a) VINCOLANTI per l'ordine S-106

1. **T-MM-107-1** — Nessun verdetto, criterio o banda di S-106 cita
   «37 ns», «~9 ns alloc» o «costo del contenitore» come numero: solo
   direzione; riscrittura in report/memoria dovuta.
   [assorbe KS-HE-107-1 ≡ KS-GR-107-2, R-HE-107-2, R-GR-107-1,
   A-HE-107-2 ≡ A-GR-107-1, A-LE-107-5]
2. **T-MM-107-2** — Nessuna banda da componenti prezzate come trigger
   STOP/bisect: trigger solo da distribuzione misurata dello stesso
   estimatore (modello R=7 34,64). [assorbe KS-LE-107-2, R-LE-107-2]
3. **T-MM-107-3** — Criterio H-C3 (punto 5): segno + soglie di
   promozione pre-registrate (pavimento 4, max(rumore, layout));
   magnitudine orientativa NON vincolante; icache solo targeting,
   contatori L1I/INST_RETIRED prerequisito di TESI.
   [assorbe A-LE-107-2, KS-HE-107-2]
4. **T-MM-107-4** — Admission-disasm esteso: diff per-target COMPLETO
   (set-difference su tutti i target) + taglia per-arm; «nessun flip»
   solo a diff pulito fuori dai siti toccati; senza diff per-target le
   componenti restano non citabili (admission ≠ licenza).
   [assorbe A-HE-107-1, R-HE-107-1, KS-HE-107-3]
5. **T-MM-107-5** — Ogni lettura census cita hash del binario + MANIFEST
   dell'apparato montato; il rerun arità va su binario PULITO e si
   compone col contatore hit/miss A-BA-107-1 (stesso run, stessa build)
   dentro il punto 6. [assorbe KS-LE-107-1, A-LE-107-3, R-LE-107-3]
6. **T-MM-107-6** — Lettura coppia WP e quota-calls SOLO con formula e
   bande pre-registrate KS-GR-107-3 (f̂=(1−r_full/1,89)/0,14;
   full∈[1,84;1,89], media∈[2,57;2,64]); letture fuori banda ⇒ rerun,
   mai bisect. [assorbe KS-GR-107-3, A-GR-107-2; compatibilità con
   T-MM-107-2 dichiarata in §1]
7. **T-MM-107-7** — Early-stop pre-registrato nel criterio della leva
   (punto 5): smoke R=2 con segno 2/2 opposto e |Δ|>max(rumore, tetto
   banda) ⇒ stop ammesso «caduta indiziata»; il pieno R si spende solo
   dichiarando PRIMA l'uso; promozione mai da smoke.
   [assorbe KS-GR-107-1, R-GR-107-2]

### (b) Raccomandazioni

8. **T-MM-107-8** — Nel gate PIN-106: micro (almeno categoria bersaglio)
   su hash₁ E hash₂ del churn di relink — secondo punto della
   banda-layout GRATIS; obiettivo N≥3 entro due sessioni.
   [assorbe A-HE-107-3, R-HE-107-3]
9. **T-MM-107-9** — Text-budget run_loop: taglia registrata a ogni pin;
   bilancio obbligatorio a +4 KB cumulativi dal riferimento S-104
   (257.632 B) o alla terza leva additiva ⇒ sveglia PGO/outlining
   A-HE-106-4. [assorbe A-HE-107-4]
10. **T-MM-107-10** — Sostituire la lettera di un gate = emendamento
    DICHIARATO (test soppresso nominato + dove verrà risposto); attese
    census scritte negli spigoli REALI dell'istogramma (le16/le32/…).
    [assorbe A-LE-107-1, A-LE-107-4]

## 3. Modifiche all'ordine provvisorio §S-106

**Nessuna modifica alla SEQUENZA dei punti 1-7.** Il team chiede tre
INTEGRAZIONI di contenuto: (i) punto 5 — il criterio della leva incorpora
T-MM-107-3 e T-MM-107-7 (già in parte previsti) e l'admission usa il
diff per-target T-MM-107-4; (ii) punto 6 — il contatore hit/miss e il
rerun arità girano su binario pulito con manifest (T-MM-107-5);
(iii) chiusura PIN-106 — micro su entrambi gli hash del churn
(T-MM-107-8) e registrazione taglia run_loop (T-MM-107-9). Il punto 1
(voci aperte coppia) si rilegge SOLO con le bande T-MM-107-6.
