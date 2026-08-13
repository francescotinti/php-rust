# Revisione S-134 — lente MISURA (revisore singolo)

## Reperto principale
**La «riconciliazione parte-modellata» è infalsificabile dall'alto e copre il 74% dell'effetto promosso.** Ricomputato: D objalloc = 943,3−806,7 = **+136,7**; la parte modellata è 35,4 → eccedenza **+101,3 = 74%** di D (objdatains: +97,9 = 73%). Il criterio p.2.3 la pre-dichiara «attesa se D>modellata» con componenti nominati ma **non prezzati**: qualunque D sopra 35,4 passa per costruzione. In S-133 lo stesso 35,4 era **UB** e l'esito D>UB fu marcato FUORI BANDA; S-134 lo degrada a bordo inferiore. La sonda a conteggi (p.2.6) era prevista solo «se ambiguo» e **non è stata eseguita**: l'attribuzione meccanicistica di ~3/4 del guadagno resta non misurata (precedente WP-104: artefatti layout della stessa scala). Non invalida il D (reale, riprodotto dal submicro) né il pin; invalida la pretesa che la riconciliazione col modello abbia «verificato» la leva.

## Reperti secondari
1. **objdatains submicro↔A/B fuori dalla banda del giudice**: 1183,3−1066,7 = −116,6 vs D=+133,3 → scarto **16,7 > 13,3** (spread-batch dichiarato). Nessuna banda submicro↔A/B è pre-registrata da nessuna parte; rientra solo nello spread submicro (0,09 s→30 ns), mai invocato.
2. **Coppia WP avversa al riferimento**: on-only 1,769 vs s133 1,754 (**+0,015**). Le gambe off dello stesso run mostrano spread 1,748–1,789 (0,041): chiamare «prima banda propria» due valori che concordano al terzo decimale (1,7694/1,7692) con quella variabilità sotto gli occhi è ottimista; N=2 non è una banda.
3. **Smoke1 rc=1** (quiescenza fallita): dichiarato con STOP e rieseguito (smoke2 rc=0) — gestito, ma la riconciliazione citata nel verdetto usa solo smoke2; floor_B smoke2 = 0,03 vs 0,02 nel R5 (minore, 3,3 ns).

## Vagliate e respinte
- Soglie: gambe B s133 objalloc 2,88−2,80 = 0,08 s→**26,7**; objdatains 0,04→**13,3**; objchurn 0,03→10,0; objmap 0→max(4)=4. Tutte corrette.
- D dai raw tsv: objalloc med 2,85/2,44→943,3/806,7; objdatains 3,55/3,15→1176,7/1043,3; drop-1 (chiave (|x−m|,x)) A'=10,0/13,3, B'=3,3/6,7 — verdetto esatto.
- ABAB reale (tsv e `i%2` nello script); stdout A/B byte-identici 10/10 con gate STOP; N=3000000 dal sorgente.
- Ordine: criterio 12:58 → codice 13:05 → run 13:28:44 (mtime tsv) → verdetto 13:29.
- rc citati: tutti esistono e valgono 0 (A/B, quiesce, 5× pair, done).
- Rapporti propri: 784,74/443,50=1,769; 781,14/441,53=1,769 ✓; submicro 6,6=2,44/0,37 ✓; objalloc −133,4 vs +136,7 → 3,3 in banda 26,7 ✓.

## Azioni S-135
1. Sonda a conteggi (apparato s133) su pin s134 vs stash s133: verificare resolve 2→0/iter e ripartire l'eccedenza +101,3.
2. Prossimo criterio: banda superiore falsificabile obbligatoria — niente componenti «nominati senza prezzo».
3. Pre-registrare la banda submicro↔A/B; ricondurre lo scarto objdatains 16,7.
4. Coppia WP on-only a N≥3; confronto formale 1,769 vs 1,754 con la variabilità off (0,041) come banda.
5. Disasm bl-count prima/dopo su prop_set_entry per escludere il canale layout nell'eccedenza. [Nota di sessione: il bl-count run_loop s133↔s134 è stato eseguito (resolve 21=21, magic 10+2, +681 istr/+32 bl dichiarati) — l'azione resta valida per la parte NON coperta: escludere che il layout (+681 istr) contribuisca all'eccedenza, cosa che il conteggio statico da solo non esclude.]
