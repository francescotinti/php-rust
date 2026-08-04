# Verbale sedia 5 — Bak (VM: alloc-rate, path caldi, corpi handler) — WP-97

## VERDETTO

**NON REFUTATO nel merito; PROCEDI CON EMENDAMENTI VINCOLANTI.** F1+F2 sono
lavoro come lo pretendo: contatori esatti, determinismo dimostrato, negativo
che morde, direzione dell'errore scelta. Ma l'aritmetica delle bande è
un'ESTREMO SUPERIORE spacciato per stima centrale, e il §WP-96 contiene una
frase falsa sul mio perimetro.

**Le bande.** La moltiplicazione frazione×canale è aritmeticamente esatta
(verificata cifra per cifra nei due .out) ma il fattore 4,5–6,5% ha due
difetti: (a) è SCREEN R=1 (la mia consulenza lo dichiara), quindi
`grade=VERDICT` vale per i CONTEGGI, non per le righe `guadagno_cpu_*` — la
banda ALTA è una decisione SCREEN-grade; (b) il canale «Zval clone+drop
attribuibile a run_loop/binary_value_ab» include i drop FINALI (la free vera)
e il traffico Zval non-slot dei 185 bracci: un take evita solo la COPPIA
transiente inc/dec, non la distruzione finale, che avviene comunque una volta.
Quindi la banda è un tetto, e il margine sopra la soglia ALTA è 0,61pp.
Robustezza: anche con canale gonfiato 2×, safe cade in MEDIA (0,95%) e str
resta sopra l'abbandono — la decisione «si prosegue» regge; l'etichetta ALTA
no.

**Il corpo caldo.** «Non aggiunge opcode al percorso caldo, ne cambia uno
esistente» (design95 §finale) è FALSO per F3: `TakeSlot` è un braccio NUOVO
di `run_loop`, eseguito ~22,7M volte (perimetro safe). La lezione WP-39..44
impone il tetto, non lo slogan.

## Emendamenti (A-LB-97-n)

- **A-LB-97-1 (tetto corpi caldi)**: Δ netto bracci CALDI ≤ 0. `TakeSlot`
  sostituisce 1:1 una `LoadSlot`/`LoadVar` (op-census: totale dispatchato
  INVARIANTE, quota TakeSlot = quota sottratta ai due bracci). Corpo di
  `TakeSlot` ≤ corpo di `LoadSlot` (move + store Undef + guard Ref; niente
  altro). `nm -S run_loop`: taglia predetta PRIMA del commit, misurata dopo;
  se sfora la predizione, si outlinea (O1) PRIMA di rivendicare.
- **A-LB-97-2 (controllo positivo F4, specifica completa)**: tre contatori,
  non uno — `takes_executed`, `ref_fallbacks`, e l'identità
  `takes_executed + ref_fallbacks = would_take_safe` dinamico (tolleranza =
  rumore suite, ~decine su 22,7M, come in nota-determinismo). Il numero
  predetto si SCRIVE prima di F4. Dichiarare che conteggio (build census) e
  cronometro (build parità) vivono su BINARI DIVERSI, stessa suite.
- **A-LB-97-3 (falsificatore mancante in P3)**: guadagno F4 SOTTO la banda
  min del perimetro scelto (oltre lo spread A/A) = modello del canale
  falsificato; va NOMINATO e il canale ri-derivato (split transiente/finale),
  non assorbito in silenzio.
- **A-LB-97-4 (decisione perimetro)**: la scelta F2-intero vs nucleo _str si
  prende A INIZIO F3 col conto dei raw; se si sceglie F2-intero, i test
  trappola distruttori sono BLOCCANTI nello stesso commit.

## Kill-switch (KS-LB-97-n)

- **KS-LB-97-1**: identità A-LB-97-2 violata → nessuna lettura di tempo è
  valida; F4 si ferma lì.
- **KS-LB-97-2**: `ref_fallbacks` > 5% di `would_take_safe` → il perimetro
  statico perde; bande da ri-derivare prima di allargare l'emissione.
- **KS-LB-97-3**: qualunque divergenza d'ordine `__destruct` sui gate →
  ripiego IMMEDIATO al nucleo _str (banda MEDIA), non un fix inseguito.
- **KS-LB-97-4**: op-census non invariante o taglia run_loop oltre predizione
  senza outline compensativo → la leva non è provata, Δ tempo non rivendicabile.

## Refutazioni capitali

**Una**: la frase «non aggiunge opcode al percorso caldo» è refutata — F3
aggiunge un braccio caldo e deve pagare il tetto WP-39..44 (A-LB-97-1). La
consulenza Bak è stata usata correttamente su tetto dispatch e
contatore-prima-dell'orologio; NON sul conteggio dei corpi. I conteggi
F1/F2 e la decisione di proseguire NON sono refutati.
