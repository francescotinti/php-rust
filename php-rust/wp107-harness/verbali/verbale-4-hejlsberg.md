# Verbale sedia 4 — Hejlsberg (compilatori, codegen, layout, incrementalità) — Concilio WP-107 su S-105 e programma S-106

**VERDETTO**: la PROMOZIONE della forma 2 REGGE sul criterio pre-registrato
11cc23a (T∧C, 5/5, census 0,0000) e non la contesto. REFUTO la
sovrastruttura interpretativa: una componente di costo post-hoc («~37 ns
il contenitore») già promossa a Scoperta, una licenza di citazione
concessa su firme aggregate, un campione di layout gratuito lasciato
cadere. **Una refutazione è CAPITALE (R-HE-107-2).**

## Refutazioni

**R-HE-107-1 (admission ≠ licenza)**. La firma «nessun flip» della forma 2
poggia su QUATTRO aggregati: bl 5408→5416, dropZval +2, alloc/dealloc
identici, +324 B. Gli aggregati non escludono flip COMPENSATI: un sito
inlinato + un sito outlinato altrove = bl netto invariato, invisibile a
`s105-admission-disasm.sh`, che confronta totali e top-12, non
l'istogramma per-target completo in set-difference. Per l'ADMISSION
(procedere al pieno, KS-GR-106-2) basta; per la LICENZA che il verdetto
si autoconcede («KS-BA-106-1 rispettato: le componenti si possono
citare») NON basta. Peggio sulla forma 1: alloc 117→116 e dealloc
363→370 sono variazioni FUORI dal sito toccato, liquidate come
«perturbazione localizzata» senza il diff per-target che solo può dirlo.

**R-HE-107-2 (CAPITALE)**. Il «~37 ns/iter = costo del contenitore» è
(T2=+23)−(T1=−14) tra DUE binari a layout diversi (+324 B/+8 bl vs
+740 B/+16 bl), ciascuno vs A: una difference-of-differences che somma
DUE errori di layout mai campionati (banda N=1, 0,67) ed è un estimatore
NON pre-registrato — il criterio 11cc23a fissa Δ∈[6,14] e i co-primari,
non la decomposizione. È la stessa falla dell'estimatore post-hoc di R=7
condannata da Leijen UNA SESSIONE FA, e stavolta è già entrata in
WP_SESSION_105 come «Scoperta 1» e «terza conferma della tesi S-104».
La DIREZIONE (forma 1 sotto, forma 2 sopra: 0/5 vs 5/5) è firmata; il
NUMERO 37, la ripartizione «~9 all'alloc», l'etichetta «costo del
contenitore» e il rango di «conferma» NO.

**R-HE-107-3 (occasione persa a costo zero)**. Il churn di relink
5e8c84c9→d4d0fa52 è una coppia SAME-SOURCE/layout-diverso: micro su
ENTRAMBI gli hash avrebbe dato il secondo punto della banda-layout senza
una build in più. Si è misurato solo hash₂. I tre binari della sera da
soli non bastavano (sorgenti diversi ≠ campione di layout), ma la coppia
di churn sì — ed è GRATIS a ogni gate PIN-106.

## Emendamenti

- **A-HE-107-1**: estendere l'admission-disasm con diff per-target
  COMPLETO (uniq -c su tutti i target, set-difference) + taglia per-arm
  quando la patch aggiunge un braccio; «nessun flip» si dichiara solo a
  diff pulito fuori dai siti toccati.
- **A-HE-107-2**: riscrivere in report/memoria «37 ns» come «divario
  direzionale tra forme, magnitudine NON firmata»; «terza conferma della
  tesi S-104» decade a indizio.
- **A-HE-107-3**: nel gate PIN-106, micro (almeno la categoria bersaglio)
  su hash₁ E hash₂: banda-layout ad accrescimento gratuito, N≥3 entro
  due sessioni.
- **A-HE-107-4 (text-budget)**: run_loop è a 257.956 B; la taglia si
  registra a OGNI pin (già nel disasm) e un bilancio di testo è
  OBBLIGATORIO a +4 KB cumulativi (~1,5% sul riferimento S-104
  257.632 B) o alla terza leva additiva consecutiva ⇒ si sveglia
  PGO/outlining A-HE-106-4.

## Kill-switch

- **KS-HE-107-1**: nessun verdetto, criterio o banda di S-106 cita
  «37 ns» o «costo del contenitore» come numero — solo la direzione.
- **KS-HE-107-2**: «icache-bound» resta ipotesi N=1 NON firmata: in
  S-106 citabile SOLO come razionale di targeting; ogni leva la cui TESI
  è icache (H-C3, H-ICS) ha i contatori L1I/INST_RETIRED come
  prerequisito di tesi (composizione WP-106) o riscrive il criterio
  senza quella premessa.
- **KS-HE-107-3**: leva con admission a soli aggregati = ammessa al
  pieno, ma componenti di costo NON citabili finché manca il diff
  per-target.
