# Revisione S-194 (lente: misura) — Generato con assistenza AI (Claude Fable 5.2)

## Verdetti
1. **s190 confermato in tempo sui carichi: REGGE CON RISERVE.** WordPress è solido (N=6, [1,535;1,560] contro [1,579;1,587]). L'ORM ha due sole gambe per binario e gambe segnalate tenute nell'intervallo (t32 phpr1: 2504 cambi di contesto; t33 phpr2: 452): lo scarto s190−s189 (0,204) è minore della dispersione interna di s186 (0,246). Lo reggono i secondi assoluti (≈26,1 · 27,9 · 30,6) e il conto in istruzioni.
2. **Costo della guardia: «reale» REGGE** (b7 e b7b 137,00 ns; segno costante in tre rotazioni). **«Non curabile limando istruzioni» e l'attribuzione a N1: NON REGGONO.**
3. **REGOLE §3 (allarme, non verdetto): REGGE. §6 (cancello in cicli): NON REGGE com'è scritta.**

## Il reperto che invalida
Lo strumento in cicli ha una dispersione più grande di ogni soglia con cui ha deciso; il suo stimatore di rumore (drop-1 della mediana, ≤1,00) non la vede.
- Stesso binario N3: 286,61 alle 10:00, 281,52 alle 15:10. Scarto N3−s189: +10,34 poi +3,73. Soglie usate: +4 (attribuzione), −3 (C5a). N1 (+4,47, zero istruzioni) sta dentro l'oscillazione.
- `s194g-forme-c5a.out`: `var1` e `var1x`, stesse 1865,2 istruzioni, stesso binario: 293,7 contro 282,4 cicli. C5a dà −11,9 su `var1` e +2,7 su `var1x`: corse singole.
- Cancello di C5a irraggiungibile: 19 istruzioni × 0,150 = 2,85 cicli < 3. E C4 è spiegato dalle istruzioni: 43,03 × 0,1503 = 6,47 su 6,58 cicli.
- Confronto non omogeneo: `s194g-forme.sh` oppone il braccio c5a (albero, target `s192-armD`, 16.178.760 byte) al braccio N3 della bisezione (archivio, altra target, 15.370.000 byte). Se la disposizione vale +4,5 cicli, due ricette diverse non isolano C5a.

Conseguenza: C5a è «non misurata», non «inefficace»; e §6 promuove a cancello lo strumento la cui instabilità motiva §3.

## Azioni
1. NEXT: «C4 = costo di istruzioni; C5a non decisa per potenza insufficiente; N1 non attribuito».
2. Prima di usare §6: giro A-A (stesso binario, due nomi) in `s194g-cicli.py`; soglia = scarto A-A fra rotazioni distanti ore, non il drop-1.
3. `s194g-cicli.py` scriva percorso e hash dei binari e una riga d'igiene: oggi «stesso paio di binari» non è verificabile dal `.out`.
4. Ogni cura contro un controllo della STESSA ricetta e target, R≥9 anche sulle forme.
5. Prossima notte ORM: tre gambe per binario, ordine ruotato.
