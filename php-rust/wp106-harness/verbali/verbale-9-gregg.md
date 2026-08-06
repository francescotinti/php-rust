# Verbale sedia 9 — GREGG (metodologia di misura, attribuzione) — Concilio WP-106 su S-104

## VERDETTO

S-104 è una sessione ONESTA sull'oggetto. KS-GR-105-1 è saldato nella
lettera E nello spirito **questa volta**: l'A/B della leva H-C2 è stato
ESEGUITO due volte (forme coerenti −10,33/−11,33 ns/iter, 5/5), il
meccanismo NOMINATO dal disasm (bl 1101→0, run_loop +8.000 B), il revert
verificato al byte. Una caduta con canale refutato è conoscenza d'oggetto,
non apparato. MA: i rapporti sono fermi da tre sessioni (prop 11,5 da
S-102; leve spedite: 0 da S-101). «Leva tentata» diventa un alibi se il
contatore non distingue lo SFORZO (A/B eseguito) dall'ESITO (rapporto
mosso). Il contatore va sdoppiato — vedi A-GR-106-1.

## Refutazioni

- **R-GR-106-1**: la lettera di KS-GR-105-1 («A/B eseguito, qualunque
  verdetto») misura il rito, non l'oggetto. Se S-105 la soddisfa di nuovo
  con una seconda caduta, avremo quattro sessioni a rapporti congelati e
  contatore formalmente a zero. Il contatore che serve ai RAPPORTI conta
  gli esiti.
- **R-GR-106-2**: «run_loop è ICACHE-BOUND» è un'inferenza da UN
  esperimento che conflaziona due variabili (dispatch eliminato E +8 KB di
  testo); la banda-layout a sostegno è N=1 (0,67, dichiarata nominale).
  Prima che la legge guidi la selezione di TUTTE le leve future serve una
  contro-prova size-only (perturbazione di sola taglia, o contatori L1i se
  accessibili). Fino ad allora: IPOTESI FORTE con una conferma, non legge.
- **R-GR-106-3**: il disasm bl-count è stato usato POST-HOC. Il flip
  dell'inliner era visibile al PRIMO build di B: il check size/bl-count
  come gate di AMMISSIONE avrebbe risparmiato metà finestra (R=5 × 2
  forme spese su un binario che misurava il codegen, non la leva).
- **R-GR-106-4**: l'A/B non registra la sanity dell'output del giudice
  per braccio (hash A vs B); la parità vive solo sul pin di chiusura.

Su (b): l'avanzamento d'oggetto è VERO ma quasi tutto NEGATIVO (canale
drop refutato, ret_cell escluso per layout E misura, metrica full-peak
esaurita, direzione peak firmata 7/7 p=0,0078, stub memory_get_usage
scoperto). Legittimo — la refutazione pota lo spazio di ricerca — ma la
cartografia SCADE se S-105 non la converte in un Δ firmato. Il free
inchiodato (1×32 B alloc + 1×32 B free/chiamata, realloc≡0) è l'unico
pezzo già pronto a diventare leva.

## Emendamenti

- **A-GR-106-1** (testuale, in testa a NEXT_SESSION per S-105): «A/B
  eseguiti: 1 (H-C2, caduta con canale refutato); leve spedite: 0 da
  S-101; **sessioni-senza-Δ-rapporti = 3** (S-102/103/104, prop 11,5
  ferma)». Δ-rapporto = movimento ≥0,3 oltre banda tra-sere su almeno un
  giudice, su pin promosso.
- **A-GR-106-2** (protocollo prossima leva): (i) ADMISSION disasm: dopo
  il build di B, bl-count + taglia run_loop; |Δtesto| > 2 KB o flip
  inliner ⇒ STOP e pin dell'inlining PRIMA dell'A/B; (ii) **SMOKE A/B
  R=2** con kill pre-registrato (2/2 segno sfavorevole e |Δ| >
  max(rumore, 3) ⇒ abort: niente forma 2, niente gate pieno); (iii) hash
  dell'output del giudice per braccio; (iv) SECONDA CANNA pre-caricata:
  il criterio della leva di riserva si scrive nella finestra d'attesa.

## Kill-switch

- **KS-GR-106-1**: se S-105 chiude con sessioni-senza-Δ-rapporti = 4,
  anomalia DICHIARATA in testa al report; WP-107 rialloca la categoria
  bersaglio (via da prop).
- **KS-GR-106-2**: leva S-105 senza admission-disasm e smoke R=2 NEL
  launcher = A/B VOID.

## Priorità S-105 (massimizza Δ sui rapporti)

1. **SiteTag H-D (breve, gated residuo≡0) → attribuzione args-Vec →
   LEVA su calls 7,6**: canale inchiodato su ENTRAMBI i lati, elimina
   volume di lavoro (coerente con l'ipotesi icache) — massima probabilità
   di Δ firmato. Banda del criterio da numeratore MISURATO (null-lever
   alloc), mai da conteggi (NON-riproporre).
2. Se la leva spedisce: **coppia WP bimodale nella stessa sessione**
   (salda il debito S-103/104).
3. Seconda canna: criterio fusione-op su prop (pila operandi 26,6%,
   S-101) SCRITTO nella finestra, eseguito solo se resta tempo.
4. Contro-prova icache size-only: backlog per NOME (R-GR-106-2).
