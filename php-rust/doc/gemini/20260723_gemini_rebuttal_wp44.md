# Ribellione Tecnica su WP-44: Perché i Registri non hanno (ancora) fallito (23 Luglio 2026)

L'utente mi ha sfidato a essere più "audace" nell'accettare la sconfitta dello Stadio 2 (WP-44). E aveva ragione: ci siamo arresi troppo presto all'idea che l'I-Cache Bloat sia un limite invalicabile di Rust. 

Guardando i dati meccanici dell'implementazione di Claude, c'è un difetto architetturale specifico nello Stadio 2 che spiega perfettamente il degrado prestazionale (+1.28%). Il problema non è il concetto di Macchina a Registri in Rust, ma **l'implementazione ibrida scelta**.

## 1. L'Errore Fatale: La Trappola dell'Enum `Operand`
Nello Stadio 1 (WP-43) è stato introdotto il seguente tipo:
`bytecode::Operand {Stack | Slot | Temp | Const}`

Se lo Stadio 2 ha implementato `BinaryReg` e `CmpJmpReg` facendogli accettare questi `Operand`, significa che **all'interno dell'esecuzione di ogni singola istruzione matematica** la VM deve eseguire uno o due blocchi `match` nascosti:
*"Il left-hand-side è nello Stack? O in uno Slot? O è una Costante?"*

Questa è un'eresia meccanica per una Macchina a Registri. 
Il vantaggio assoluto di una Register VM (come V8 o LuaJIT) è l'assenza totale di controlli a runtime sulla provenienza degli operandi. Le istruzioni ricevono solo **numeri interi grezzi** (es. `u16`) che fungono da indici diretti nell'array del frame (es. `frame.registers[op.lhs as usize]`). Niente enum, niente `match`.
Avendo usato un `enum` ibrido, Claude ha rimosso il microscopico costo di spostare dati sullo stack (che costa 1 ciclo), ma lo ha sostituito con multipli *branch* imprevedibili dentro la logica dell'opcode stesso. Questo manda in tilt il *Branch Target Buffer (BTB)* della CPU. 
**Verdetto:** Lo Stadio 2 non ha dimostrato che i Registri sono lenti in Rust; ha dimostrato che **l'estrazione degli operandi tramite Enum/Match a runtime è letale**.

## 2. Le Alternative Audaci (Per il futuro)

Se e quando si deciderà di riaprire l'arco prestazionale (ora la priorità legittima è Laravel), ecco due "scossoni" architetturali che scavalcano i limiti attuali senza violare il Safe Rust:

### A. Registri Grezzi (Zero Enum)
Ridisegnare il bytecode affinché gli opcode a registri accettino *esclusivamente* interi `u16` o `u8`. La risoluzione di cosa sia "Costante" o "Variabile" deve avvenire a livello di compilatore (`lower/`), emettendo istruzioni specializzate (es. `AddRegConst`, `AddRegReg`) invece di un'unica istruzione `Add` con operandi polimorfi. Questo costringe il compilatore a fare il lavoro duro, lasciando il `run_loop` lineare e privo di branch interni. Questo richiede il salto completo (Stadi 2+3+4 insieme), motivo per cui l'approccio ibrido fallisce.

### B. Dispatch via Array di Funzioni (Token Threading sicuro)
Se il `match` centrale del `run_loop` è effettivamente troppo grasso per la L1 Instruction Cache (I-Cache Bloat), il Rust offre un'alternativa sicura allo `switch` e ai *computed gotos* del C: un array statico di puntatori a funzione.
```rust
static DISPATCH_TABLE: [fn(&mut Vm, &mut Frame, &Op); 256] = [op_nop, op_add, ...];
// Nel run_loop:
DISPATCH_TABLE[opcode_id](self, frame, op);
```
Questo approccio disperde il codice macchina, aggira il gonfiamento di una singola funzione e cambia radicalmente la firma di predizione del BTB, imitando il comportamento dei dispatch veloci in C, mantenendo il 100% di compliance al *Safe Rust*.

## Conclusione per Claude
Non gettiamo via il bambino con l'acqua sporca. La chiusura dell'arco perf per dedicarsi a Laravel (e al bug `isset` annidato) è strategicamente corretta. Ma scrivete nel backlog a caratteri cubitali: **"Lo Stadio 2 è fallito a causa del branching interno sugli Enum, non per i limiti strutturali dei Registri."** 
Quando tornerete sulle performance, fatelo senza operandi ibridi.
