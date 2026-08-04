# Verbale sedia 9 — Gregg (metodologia di misura e attribuzione) — WP-97

## VERDETTO

**APPROVATO CON DECLASSAMENTO DI GRADO.** La decisione di prosecuzione
(strada lunga F2→F3→F4) regge; la banda P3 NO, non al grado dichiarato.

1. **Il canale 4,5–6,5% NON regge come moltiplicatore VERDICT.** Deriva da
   prof95-media.out, che si auto-dichiara SCREEN R=1 («serve a ORDINARE le
   leve, non a pinnare cifre»). L'aritmetica è consistente (7,20×0,542 +
   2,85×0,581 ≈ 5,56%, dentro la banda), ma soffre di tre debolezze non
   quantificate: (a) attribuzione a livello di simbolo sotto inlining
   release — i clone/drop INLINED spariscono dai simboli `Zval::clone/drop`
   e finiscono direttamente nel chiamante, quindi la quota è distorta in
   direzione ignota; (b) R=1: nessuno spread del canale stesso; (c) le
   percentuali dei chiamanti vengono da stack invertiti campionati, non da
   conteggi. **La banda P3 va dichiarata SCREEN finché F4 non la misura.**
2. **Grado del prodotto = il fattore più debole.** frazione (VERDICT,
   conteggi esatti, determinismo riprodotto) × canale (SCREEN) = **SCREEN**.
   zvalcensus-f1.out e f2.out marchiano `grade=VERDICT` in testa a file che
   contengono righe `guadagno_cpu_atteso_*` derivate dal canale SCREEN:
   contaminazione di grado nel raw stesso.
3. **Sensibilità che salva la decisione**: canale sovrastimato 2× →
   floor safe 1,91%→0,95% = banda MEDIA, che comunque preferisce la strada
   lunga a parità di conto. La PROSECUZIONE è robusta all'errore del
   canale; la PREDIZIONE no. Distinguere le due cose per iscritto.
4. **Asimmetria del modello di costo**: la predizione conta il lavoro
   RISPARMIATO (coppie inc/dec) ma non il lavoro AGGIUNTO da F3 (guard di
   tipo a runtime, store di `Undef`). Un Δ sotto banda con
   `slot_reads_avoided` al valore predetto = il prezzo del guard, e va
   nominato, non trattato come rumore.

## Emendamenti

- **A-BG-97-1**: annotare nei due raw (o in un companion) che le righe
  `guadagno_cpu_atteso_*` sono grado SCREEN; il `grade=VERDICT` di testa
  copre SOLO i conteggi.
- **A-BG-97-2**: prima di F4, sanity-check ns/evento (regola WP-53/54 «i
  conteggi non sono secondi»): 22,67M coppie rc safe × costo plausibile
  per coppia confrontato con la user CPU del media group — se il canale
  implicito esce fuori da 4,5–6,5%, ri-derivare la banda.
- **A-BG-97-3**: F4 dichiari R e il tetto di spread A/A PRIMA del run; il
  confronto è «Δ vs spread», con esito UNDECIDED se lo spread copre il
  floor della banda.
- **A-BG-97-4**: specificare che `slot_reads_avoided` si legge da un run
  SEPARATO con feature census sullo stesso HEAD: il controllo positivo
  non condivide mai il binario col cronometro.
- **A-BG-97-5**: se F3 sceglie il nucleo stringhe, la P3 si RI-deriva dai
  numeri `_str` (0,84–1,21%, banda MEDIA) prima dell'opcode — non si
  eredita la banda safe.

## Kill-switch

- **KS-BG-97-1**: spread A/A ≥ 1,9% (floor della banda safe) → F4 non può
  emettere verdetto; fermarsi e alzare R o ridurre il rumore.
- **KS-BG-97-2**: `slot_reads_avoided` devia oltre una tolleranza
  dichiarata ex-ante dai 22.674.665 di F2 → qualsiasi Δ è NON attribuito.
- **KS-BG-97-3**: Δ oltre 2× la banda (falsificatore P3 esistente) O sotto
  il floor con controllo positivo verde → profilo di coppia obbligatorio
  prima di qualunque rivendicazione.

## Refutazioni capitali

**Sì, una (di grado, non di merito)**: la banda ALTA è presentata a valle
di file `grade=VERDICT` ma è un prodotto SCREEN — refuto l'etichetta, non
la prosecuzione, che sopravvive perfino a un errore 2× del canale.

OGGETTO: sì, la sessione ha avanzato l'oggetto — oggi sappiamo, con conteggi esatti e deterministici, che il 47,11% delle letture rc di phpr è un ultimo uso, che la prudenza ne lascia il 90,21%, e che i Ref a runtime sfuggono all'analisi statica: fatti nuovi, falsificabili da F4.
