# WP_SESSION_195 — corpo di Sweep, valore all'invio, tetto del modello a registri (2026-11-28, APERTA) — Generato con assistenza AI (Claude Fable 5.2)
**In una frase**: il raccoglitore lavora un po' meno a ogni istruzione, una chiamata come `$o->m($i, $i = 5)` ora dà lo stesso risultato del PHP ufficiale, e una misura dice che passare a un bytecode «a registri» può rendere al massimo il 6–7 %: abbastanza per provarne una fetta, non per riscrivere la macchina.
**SCOREBOARD** (PIN phpr **s190** `97e000aa00fe802e` INVARIATO, server **s191** `c413398a44f9b0a4`): arith 2,2 → · prop 2,4 → · calls 3,6 → · str 3,3 → · arr 2,9 → · re 2,6 → (non rimisurati) · WP t29 1,555 · ORM t31 [5,184; 5,316] (riferimenti del 27/11) · **leve spedite: 0 pin** (due leve nel tree, promozione pronta e ferma sul disco) · batteria 1792/0/2 (+1 dente `a_variable_argument_is_read_at_the_send`) · corpus 1399 congelati per modo.
## Esiti secchi (branch `s186-firma` → b70ba82f e seguenti; `main` non toccato)
- **Disamina dell'architettura (richiesta dell'utente)**: nessuna riscrittura a registri consigliata; il divario strutturale è l'ABI delle chiamate. Ordine dell'utente: corpo di `Sweep`, poi `$o->m($i, $i = 5)`, poi il tetto misurato del modello a registri.
- **Pacchetto S** (`b9365351`, `s195-criterio-sweep.md`): rilascio a cascata senza hash su due mappe quasi sempre vuote + nota di un oggetto in una funzione propria. Istruzioni contro il sorgente del pin: **ORM −0,93 %, WordPress −1,53 %** (attesa −0,3 %); accettazione `s195S` rc=0. Le due cure non sono ripartite.
- **Valore all'invio** (`e04ce6e4`, forma J `abcbfd90`, `s195-criterio-invio.md`): il dente nuovo sbagliava 27 righe su 40 sul pin; ora == oracle. Nessuna op nuova (`ArgPlaceBase::LocalSnap`, analisi a compile-time in `compile/arg_hazard`). Prima forma +0,10 % ORM / +0,08 % WP; forma J **+0,05 % / −0,01 %**; accettazione `s195J` rc=0, corpus ×2 zero flip. Tre residui dichiarati.
- **Tetto dei registri** (`s195-criterio-tetto-registri.md`, pin nei due modi): le forme esistenti rendono −2,78 % ORM / −2,73 % WP; 86,5 / 73,2 istruzioni per op tolta; movimento di pila residuo 30,5 % / 33,0 % degli op; **tetto per eccesso 6,5 % / 6,9 %** ⇒ per la regola scritta prima, una fetta dietro flag, non una riscrittura.
- **Promozione → s191** (delibera 28/11, `s195-criterio-promozione.md`): catena generata, prova a secco 41 gate zero morsi, **cancello in cicli di REGOLE §6 applicato per la prima volta con soglia dal giro A-A (T = 3,46): candidato − s190 = −8,71 cicli, −30,8 istruzioni per chiamata**, sotto anche a s189. **NON lanciata: Data 7G < 10G.**
- **Fetta a registri aperta** (`s195-criterio-fetta-registri.md`): ritorno diretto, risultato di chiamata nello slot, ultimo argomento dallo slot; flag `PHPR_REG_SLICE`; tre bracci; kill a −0,3 % sui due carichi. Codice non iniziato (aspetta il pin).
## Incidenti (contati: 3)
1. Prima forma della cura dell'invio con quattro allocazioni per invio a rischio: +0,10 % sull'ORM, alla soglia del kill; rifatta (forma J) prima di tenerla.
2. Il conto in istruzioni dell'ORM deriva fra corse (212,12 → 211,21 G sullo stesso binario in un'ora): scoperto a misura fatta; da qui solo confronti dentro la stessa corsa. Causa non cercata.
3. Data a 7–8G per tutta la sessione: la promozione deliberata non è partita.
## Passi
1. Liberare Data ≥10G → `wp195-harness/s195-lancio-promo.sh` col daemonizer → cancello in cicli ripetuto sul pin → notte s191 contro s190 (criterio di notte da scrivere prima).
2. Fetta a registri: denti di p.7, poi forma A (`RetConst`/`RetSlot`), col conto a tre bracci.
3. Catalogo: le due divergenze trovate per caso (ordine dei distruttori degli elementi di un array, id di oggetto riusato).
## Lezioni
- Un tetto misurato con gli strumenti che ci sono già costa un'ora e toglie di mezzo una discussione di molte sessioni.
- Una soglia in cicli si scrive solo dopo un giro A-A nella stessa finestra: qui il rumore fra due copie dello stesso binario era 1,7 cicli.
