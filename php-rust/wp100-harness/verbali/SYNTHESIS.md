# Concilio WP-100 — SINTESI DI CONVERGENZA (su S-98.0 e programma S-99.0)

## §FONDAMENTALI (prima di tutto, per regola 2026-08-03)

**(a) Avanzamento dell'OGGETTO in questa sessione**: NON zero — Gregg
(mandato inverso) la certifica «la sessione più densa di conoscenza-oggetto
da WP-94». Nuovo per NOME: H-B1 chiusa a COSTO ZERO con la sola misura
(P=0%: il preambolo di dispatch è gratis sul core OoO, la split-borrow può
perfino perdere); H-B2 spedita con verdetto numerico (D=6,07 ns/occorrenza
sul giudice `add`, −16,2%); il fattore ~8 del costo per opcode ATTRIBUITO
al plumbing dei corpi; DUE corpus per NOME (flag-off e flag-on) sullo
stesso albero. Il metodo (criterio scritto prima) ha morso per la terza
sessione consecutiva.

**(b) Contatore sessioni-senza-misura**: full/media = WP-94, 4 sessioni fa
— e ora il collaudo è DOVUTO senza scuse (l'emissione flag-off è cambiata).
Peak footprint: nessuna misura da m90 e NESSUNO strumento nominato
nell'ordine (refutazione di Leijen). La gamba ORACLE delle sei categorie
non è mai stata rimisurata dalla baseline S-97.0 (Gregg).

**(c) Rischio d'oggetto più trascurato**: il pin php-server è alla SECONDA
rotazione consecutiva senza collaudo (832568a7 mai collaudato → 365f4d40
mai collaudato) e l'ordine S-99 in bozza gli faceva servire il collaudo
WordPress del punto 1: VOID per costruzione (Pedersen, KS-PE-99-1).

## Verdetti di fase 1 (9/9 CON EMENDAMENTI; Klabnik «rifiutato nella FORMA,
misure valide»; Bak «esiti sopravvivono ma non per le ragioni dichiarate»)

Verbali integrali VINCOLANTI in `verbali/verbale-*.md`; note di team in
`verbali/team-*.md`. Refutazioni capitali, per convergenza:

1. **D=6,07 NON è ereditabile come criterio del rollout** (SEI sedie
   indipendenti: Hoare, Matsakis, Bak, Stogov, Hejlsberg, Gregg — la
   convergenza più larga mai registrata). È il costo del plumbing del
   percorso PILA (call + marshalling per valore + pop/push); le forme
   registro flag-on NON hanno quel plumbing (BinarySS inlinea già
   `binary_fast` su prestiti: il residuo è il match del payload BinOp).
   Il criterio del punto 3 si deriva da controfattuale/misura DEL percorso
   registro (A-ST-100-1, A-BA-100-1, A-MA-100-1, A-GR-100-2); ogni
   spedizione col criterio ereditato è VOID (KS-MA-100-1, KS-HE-100-3,
   KS-BA-100-1, KS-GR-100-2).
2. **L'ordine S-99 in bozza era VOID al punto 1** (Pedersen): il collaudo
   WordPress passa da php-server, che è il binario NON collaudato.
   Riconciliazione team-ordine (Gregg concorda: il collaudo NON slitta):
   parità server = PRIMA GAMBA del punto 1, stessa sessione, poi il
   collaudo. Sigillo eager di `enabled()` + dente anti-putenv promossi a
   GATE DI PROMOZIONE (KS-HE-100-1, KS-PE-100-2), non residuo.
3. **Evidenza sovradichiarata** (Klabnik+Hejlsberg+Leijen, team-evidenza):
   il controllo positivo del funnel pinna il DUMP ma non stdout/exit; il
   gate corpus per NOME è UN bit per fail (la promozione esige il diff
   riga-per-riga, KS-KL-100-1); dump e `lowered()` sono ciechi sugli hook
   riscritti flag-on e su prop_init (RC di Hejlsberg); la coppia peak
   promessa era SENZA strumento (KS-LE-100-1: solo `/usr/bin/time -l` +
   env mimalloc di WP-94); il byte-tag è a 186/256 senza gate
   (A-LE-100-3); due rialzi di budget in una sessione — il terzo senza
   delibera esplicita = gate non-mordente (KS-KL-100-3).
4. **La sonda M1 va annotata, non ritrattata** (Bak): misura
   Δ(A_sonda, B) con una A più ottimizzata della A reale (i caldi non
   spillano) e il «tetto anti-hiding» usa il metodo count-derived che la
   sessione stessa ha refutato; il verdetto H-B1 REGGE (P sotto soglia su
   tutta la banda) ma A-BA-100-2 impone l'annotazione nel .out e
   KS-BA-100-2 declassa a banda ogni claim della sonda sotto 2× il suo
   pavimento di risoluzione.
5. **«Compone col −30,7%» è un'ipotesi, non un fatto** (Gregg): flag-off e
   flag-on non sono mai stati misurati INSIEME dopo H-B2; i rapporti con
   gamba oracle stantia (14,5→12,1) portano l'etichetta «nominale»
   (A-GR-100-1).

## Ordine DEFINITIVO S-99.0 (regola di ammissione applicata)

1. **Parità server** (prima gamba, precondizione): restapi+option per NOME
   sotto `env -i` sul pin 365f4d4069513de3 + sentinella output-capture.
2. **Collaudo WordPress full+media stessa-sera** (parità per NOME sui due
   lati) **+ coppia peak** con `/usr/bin/time -l` + env mimalloc pinnate da
   WP-94 (A-LE-100-1). Nessun claim CPU nuovo e nessun rollout prima della
   chiusura per NOME (KS-GR-100-1).
3. **Ri-baseline delle sei categorie** su ENTRAMBI i motori, stessa
   finestra (A-GR-100-4): rianima i criteri di H-C/H-D e sana la gamba
   oracle stantia.
4. **Pre-misura del rollout, SOLO misura** (il codice del rollout NON è in
   quest'ordine): controfattuale statico del percorso registro
   (A-ST-100-1) + build intermedia che decompone D in call/marshalling vs
   pop/push (A-BA-100-1) + baseline flag-on; il criterio del rollout nasce
   QUI. Se il timebox regge: sigillo eager + test anti-putenv
   (A-PE-100-2 — gate di promozione).

**Precondizioni per NOME dei passi FUTURI** (non slot di sessione):
promozione flag-on ⇐ diff riga-per-riga (A-KL-100-2) + sigillo
eager/anti-putenv + sanatoria dump/lowered (A-HE-100-4) + stdout pinnato
nel funnel (A-KL-100-1); rollout ⇐ gate N_OPS≤255 (A-LE-100-3) + matrice
fixture Add (A-KL-100-3) + tripwire zero-BinaryAdd flag-on (A-HE-100-1) +
trappole A-ST-99-3 per ogni fusione AssignOp (KS-ST-100-2); shape nuove ⇐
visit_addrs esaustivo (A-HE-100-2). BACKLOG per NOME: test differenziale
BinaryAdd≡Binary(Add) (A-HE-100-3), registro pin `collaudato:`
(A-PE-100-3), braccio flag-OFF nel funnel (A-PE-100-4), fixture in-tree
MIN/const-lhs/by-ref (A-ST-100-3), offset_of nella sonda (A-LE-100-4),
breakdown budget (A-KL-100-4), etichette e bande .out (A-BA-100-2,
A-LE-100-2, A-GR-100-1/3), census Binary(Add) residui (A-HO-100-4),
debito B2 batteria (A-KL-100-5).

L'ordine NON è composto di solo apparato: i punti 1-3 sono misure
dell'oggetto; il punto 4 è la misura che fonda la prossima leva.

## Conflitti registrati

- team-ordine: server-prima (Pedersen) vs collaudo-non-slitti (Gregg) —
  RICONCILIATO: stessa sessione, server come prima gamba del punto 1.
- team-criterio-rollout: build di decomposizione (Bak, misura) vs argomento
  statico (Matsakis, codice) — COMPATIBILI: il luogo del costo si legge dal
  codice, la grandezza si misura; entrambi entrano nel punto 4.
- team-evidenza: matrice-prima (Klabnik) vs controfattuale-prima
  (Hejlsberg) — componibili nelle precondizioni per NOME del rollout.
