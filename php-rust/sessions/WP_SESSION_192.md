# WP_SESSION_192 — leva di costo K0: dove stava davvero il costo della cura O(1) (2026-11-19 01:30 → 03:55, sessione APERTA) — Generato con assistenza AI (Claude Fable 5.2)
**In una frase**: il rincaro di ogni ritorno di metodo introdotto dalla cura O(1) è stato tolto dal tree con tre righe (e qualcosa in più: ora quel caso è più veloce di prima della cura), ma è una cifra di direzione: il verdetto di notte sul binario canonico e la promozione sono ancora da fare.
**SCOREBOARD** (PIN phpr s184 `60b10d857eb0343a`, server s190 `721b7802e2d12058`, INVARIATI): arith 2,2 → · prop 2,3 → · calls 3,5 → · str 4,3 → · arr 2,9 → · re 2,5 → (nessuna rimisura) · WP t26 1,751 · ORM t28 [7,045; 7,136] · **leve spedite: 0** (1 nel tree, 0ac0b35b, non promossa) · batteria non rieseguita (catena di giorno ×2 verde) · corpus 1407 ×2.
## Esiti secchi (branch `s186-firma`, 1ea1c6f1 → 0ac0b35b + registro; main INTATTO)
- **Pre-flight**: pin per hash ✓, dischi 2411G/11G ✓, bundle montata ✓, lock assente ✓, modo sviluppatore ✓; **MySQL wp8 GIÙ** (non serve a questa gamba: da rialzare prima di ogni gate WP).
- **C0** criterio `wp192-harness/s192-criterio-k0.md` PRIMA del codice (5de4229b).
- **Leva 1** (8dbf88fb, salto in linea della sonda al Ret): catena di giorno verde, guadagno NULLO (gemelli L−Z +0,46 ns/Ret) ⇒ **ritirata**, patch agli atti. Il modello «il costo è la chiamata alla sonda» era falso.
- **Bisezione a gemelli** (`s192-bisezione-k0.py`): 22/10 +36,7 · v1 +7,3 · v2 +7,6 · s184 +3,9/+4,1 ns/Ret vs sorgente s183; stesso sorgente ⇒ stesso hash, ripetibilità 0,2 ns. Lo strumento riproduce le guardie di notte b/b2/b4 in ~6 minuti per punto.
- **Passo 2** (0ac0b35b, `gc_note_this_tail`): nota di `$this` corta quando il ricevente è vivo e già in coda al buffer; esatta per costruzione; catena di giorno `s192k0p2` rc=0; **gemelli g3: K0 −5,08 · K4 −4,31 · K16 −6,00 ns/Ret, mcalls/calls invariati**.
- **Diagnostica**: togliere la sonda dal Ret fuso vale ~3,4 ns anche se non viene mai chiamata ⇒ resta un costo di FORMA nel loop caldo (apertura per nome).
- **Censimento h2** (`s192-censimento-h2-c1.out`, una gamba ORM, soli conteggi; i totali riproducono al numero la fetta 0 di S-186): dei 17,05M di Ret lenti **14,14M (83 %) hanno come UNICO motivo il tipo di ritorno scalare** (91 % con le combinazioni), 1,10M `ext`, 0,45M `ret_cell`; delle 7,67M chiamate di metodo generali **3,88M (51 %) per parametri tipizzati**, 2,84M (37 %) IC miss, 0,19M meno argomenti dei parametri; `$this->m()` generale 0,69M tutte IC miss. Lettura onesta: ammettere il tipo di ritorno al Ret fuso sposta il 28 % dei Ret ma vale ~1-2 % del tempo ORM (la fetta 0 dava call_ret 1,76 s self contro 12 s di self del loop): h2 è una leva piccola, il grosso resta il dispatch.
## Incidenti (contati)
1. Corsa b1 della bisezione caduta: `git archive` porta la data del commit, cargo tiene rlib stantie nella stessa target (cura: `tar -m`).
2. Script `s192-gemelli.sh` modificato mentre era in esecuzione (corsa g2): rifatta pulita come g3, stessa cifra.
3. Direzione misurata con Data a 8G (< 10G) e CI/language server attivi a tratti: per questo è direzione e non record.
## Passi
1. **Decisione utente**: verdetto di notte (guardia `s186-guardia.sh` sul binario canonico) e promozione del passo 2; restano aperte le decisioni di S-191 (REGOLE guardia-su-canonica, anti-flare `photolibraryd`, merge su `main`).
2. mcalls +3 ns/iter: un solo Ret di metodo per istruzione ⇒ push + sonda + Sweep per un ricevente vivo; leva a parte con criterio proprio.
3. Rotta (da firmare): il censimento h2 è FATTO e dice che l'ammissione vale poco sull'ORM; la domanda vera diventa il self del loop caldo (12 s nella fetta 0). Proposta: anatomia per op del dispatch con lo strumento a gemelli (ogni leva giudicabile di giorno in minuti), e l'ammissione del tipo di ritorno al Ret fuso solo come leva piccola e sicura.
## Lezioni
- Un modello di costo si prova con un gemello PRIMA di scriverci una leva: la bisezione costa 15 minuti, la leva 1 è costata una catena intera.
- Il costo in un loop caldo gigante può stare nella forma del codice e non nella sua esecuzione: una chiamata mai eseguita vale 3,4 ns.
- Mai toccare uno script in esecuzione; mai riusare una target con sorgenti d'archivio senza `tar -m`.
