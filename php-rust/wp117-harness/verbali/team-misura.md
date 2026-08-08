# Verbale team-misura (relatore) — sedie: Gregg, Bak, Leijen

## CONVERGENZE
1. Verdetto unanime: CONCORDO CON EMENDAMENTI; ordine A → B (regime) con D come cava/alimentatore.
2. **BOLT non esiste su Mach-O ARM64**: rotta A reale = PGO rustc (`-Cprofile-generate/use`) + LTO fat + cgu=1 + `ld64 -order_file`. Verifica tooling in pre-flight.
3. **Profilo = artefatto pinnato**: workload pre-registrato (sei micro + WP), profdata versionato/congelato, stesso profdata per ENTRAMBI i bracci di ogni A/B; rigenerazione solo via scripts/pin-*.sh emendato.
4. **«A ripara il metro» è ipotesi, non cura**: si giudica ri-misurando la banda leva-nulla (N≥2, patch s114/s115 riusate) sul binario PGO. Reset dichiarato: A invalida TUTTE le bande/baseline; nuovo pin, nulla si eredita.
5. Gate parità PIENO sul binario PGO (batteria, corpus per NOME ×2, output) prima di qualunque micro.
6. **C non è riserva**: l'aritmetica (~107 ns/iter prop vs ~42 target; L-A ~27 + A 5-15% non chiudono) è pre-registrabile oggi; istruttoria C (censimento alloc/op e RC/op per categoria, entrambi i motori, senza codice sul path caldo) entro S-118/119.
7. Treno B: le tasse sistematiche si sommano come i guadagni; giudizio sul treno INTERO, vagoni con ammissione/parità/direzione individuali su binari conservati.

## CONFLITTI
- **Resa attesa A**: Bak impone 5-10% dichiarato; Gregg e Leijen citano 5-15%.
- **Logica kill A**: Bak chiude con congiunzione (PGO<+3% mediano **E** banda invariata); Leijen declassa con disgiunzione (banda>5 **O** geomean<3%). Non riconciliato.
- **Cosa può diventare A se la banda non scende**: Gregg «sola leva velocità»; Bak «guadagno una-tantum, B obbligatorio»; Leijen «solo-layout-freeze».
- **Giudice del treno B**: Gregg netto pesato + espulsione leave-one-out max 1 giro; Bak somma-bersaglio ≥2× banda vigente; Leijen pretende un treno-NULLO (3-5 commit vuoti) preliminare.
- **Variante C**: Leijen presume colpevole l'arena sopra mimalloc (gate peak ≤1842 MiB+2%) e privilegia l'eliminazione delle alloc (scalari inline/NaN-box); Bak decide coi numeri tra scalari-non-contati/deferred-RC/arena; Gregg si limita al design doc entro S-119.
- **Tassa calls**: solo Gregg la eleva a primo vagone (cold/outlining del probe, dente disasm bl-count) se non assolta gratis da A.

## PRIORITÀ PER L'ORDINE S-117
1. **Spike PGO+LTO+order_file** (timebox ½ sessione). Misura: due build a sorgente invariato con hash identico + gate parità pieno (batteria 1742/0, corpus per NOME ×2, output).
2. **Ri-misura banda nulla sul binario PGO**: micro R=5 + ≥2 nulle. Misura: banda_new ≤5 ns/iter (vs 10) = claim ripara-metro; altrimenti solo Δ velocità sopra banda_new.
3. **L-A ricompilata sotto PGO (tassa calls)**. Misura: Δ calls dentro banda nulla nuova ⇒ ipotesi probe/layout firmata; dente disasm bl-count.

## KILL-SWITCH consolidati
- Parità fallita su binario PGO e fix >½ sessione ⇒ A revertata/rinviata in sessione (KS1/KS-A2).
- Banda invariata e velocità sotto soglia ⇒ A cade o declassa (KS2/KS-A1/Leijen-A; **divergenza E/O registrata**).
- Toolchain non in piedi in ½ sessione ⇒ ripiego PGO-solo; 2 sessioni solo-apparato senza A/B ⇒ anomalia, stop rotta (KS4).
- Treno B sotto bersaglio dopo 1 giro espulsione ⇒ sciolto, vagoni in coda singola, C primaria (KS3/KS-B1).
- Censimento C: <1 alloc/op e RC<30% budget, o RC-traffic ≈ Zend ⇒ C si ridisegna (KS-C1/C-tutta); arena con peak WP >1842+2% ⇒ variante morta.
