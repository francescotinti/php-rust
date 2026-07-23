# Analisi Post-WP-41: I-Cache Bloat e il Tetto dell'Architettura (23 Luglio 2026)

Ho letto attentamente i file `REPORT_GAP_41.md` e `WP_SESSION_41.md`. Questa sessione certifica ufficialmente che abbiamo raggiunto il **tetto strutturale dell'interprete attuale**. L'analisi condotta da Claude sui fallimenti e sulle attribuzioni è tecnicamente ineccepibile.

Ecco le mie osservazioni e le direttive per l'apertura della prossima sessione.

## 1. Il fallimento della Leva C: La vendetta della I-Cache
L'inlining del frontend di `gc_note` ha causato un rallentamento dello 0.62%. È un esito apparentemente controintuitivo, ma la diagnosi di Claude centra in pieno il problema: **I-Cache Bloat** (gonfiamento della cache istruzioni). 

Inserendo forzatamente (`#[inline(always)]`) il controllo del tag in ~60 opcode diversi all'interno del gigantesco `match` del `run_loop`, il volume del codice macchina generato è esploso. Questo ha saturato la L1 Instruction Cache della CPU (che tipicamente è limitata a 32KB). Il costo di ricaricare le istruzioni dalla L2/RAM ha spazzato via i minuscoli guadagni derivanti dall'evitare l'overhead della chiamata a funzione per gli scalari.
**Verdetto:** Eccellente intuizione, eccellente A/B testing e perfetto tempismo nel *revert*. Questo chiude la Leva C.

## 2. Attribuzione del Churn: La sentenza sul Bytecode a Stack
L'attribuzione per-chiamante (Punto 2) è la pistola fumante che cercavamo per decidere sul futuro del progetto.
Avete dimostrato che i continui `clone` e `drop` dei `Zval` non hanno un singolo colpevole aggredibile. Sono distribuiti organicamente lungo tutto il `run_loop` come conseguenza inevitabile dell'architettura a **Stack**. Spingere e rimuovere continuamente temporanei dallo stack genera questo rumore di fondo che nessun micro-refactoring può spegnere.

## 3. Direttive per la Prossima Fase (WP-42+)

Il 2.68x attuale è il massimo splendore che questa architettura può offrire. Ora bisogna cambiare le regole del gioco.

### Warm-up: Mini-Leva `silent_get_path` (WP-42)
L'unica inefficienza locale emersa è il cloning intermedio durante l'esplorazione dei percorsi degli array (`dim_is_walk`).
- **Azione:** Implementare l'esplorazione *by-borrow* (passando `&Zval` o `Ref<Zval>`) e riservare il clone **solo** alla foglia finale (e solo quando strettamente necessario, ad es. coalescing `??`).
- **Target:** Recuperare quel 0.5-1% rimasto sul tavolo. Ottimo task per scaldare i motori nella nuova sessione.

### Il Grande Cantiere: Leva B (Bytecode a Registri)
Tutti i dati convergono qui. È arrivato il momento di aprire formalmente l'arco multi-sessione per la transizione ai registri.
- **Strategia:** Bisognerà istruire `mago` (o il layer intermedio del compilatore) ad allocare slot persistenti (registri) per-funzione, sostituendo l'attuale logica Push/Pop.
- **Aspettative:** Preparatevi a un periodo turbolento in cui il codice non compilerà o le performance peggioreranno temporaneamente finché l'architettura non sarà a regime e il `run_loop` riscritto. Nessun'altra ottimizzazione locale deve distrarvi mentre questo cantiere è aperto.

Chiudete la mini-leva in WP-42, blindate la parità nei gate, e preparatevi a demolire lo stack. Siete a un passo dal riscrivere la storia di questa VM!
