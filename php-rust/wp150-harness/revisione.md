# Revisione S-150 — revisore singolo, lente SEMANTICA (verbale ≤400 parole)

VERDETTO: **REGGE CON RETTIFICA** — promozione e misure valide, ma il claim
«anche cura di FEDELTÀ» è sovradimensionato e due divergenze residue non sono
a catalogo per NOME.

RILIEVI

1. **Fedeltà = solo perimetro fixture.** fx-backtrace.php (righe 16–22) copre
   6 combinazioni da UN solo call-site (metodo) e stampa solo le CHIAVI, mai i
   valori di args. Non copre: options=true/false (non-int),
   PROVIDE_OBJECT|IGNORE_ARGS insieme, limit>profondità, closure/include —
   esattamente le superfici di debug_backtrace_options.phpt, che resta FAIL a
   contenuto mutato. «La cura chiude la divergenza» vale per la fixture, non
   per debug_backtrace in generale.
2. **Residui impliciti, non a catalogo.** PHPR_DIVERGENCES_FROM_PHP.md non ha
   righe per il residuo di debug_backtrace_options né per debug_print_backtrace:
   il perimetro s149 dichiara `collect_backtrace()` INVARIATO ⇒ il suo limit
   resta IGNORATO e debug_print_backtrace_limit.phpt resta nel fail-set. La
   cura crea un'asimmetria (una funzione onora limit, la gemella no) mai
   dichiarata per nome.
3. **FR1: «meccanismo NOMINATO» > del provato.** L'esito (b) è
   un'ELIMINAZIONE: i due bracci M1 condividono per costruzione lo stesso
   run_loop — l'A/B non distingue «prezzo del dispatcher +3180B/+26bl» da
   alignment/microarch del commit s145. Declassare a «localizzato per
   eliminazione, indiziato il delta strutturale».
4. **Scommessa ORM: attribuzione regge, modello no.** Δmin=6,07 ≈ 2× l'attesa
   ALTA 3,1: il «tetto census tranche-4» come TETTO è smentito — da spiegare
   in S-151, non solo dichiarare.
5. **Identità: regge, con nota.** cbbe↔ac26 provata transitivamente via 2dd3;
   coperta perché la catena RI-GIUDICA il candidato (fixture + m-backtrace
   R=5). Le «5 violazioni parità» del promo-verdetto sono il braccio A
   senza cura: coerenti, non spiegate nel verbale.

AZIONI S-151
1. Due righe a catalogo per NOME: residuo debug_backtrace_options
   (true/false/both, valori args) e debug_print_backtrace limit ignorato.
2. Estendere fx-backtrace: options non-int, both, limit>profondità,
   closure/include; ri-gate byte-id.
3. Leva di fedeltà: options/limit su debug_print_backtrace (bersaglio: flip
   debug_print_backtrace_limit.phpt).
4. Riformulare la voce FR1 chiusa: «localizzato per eliminazione; indiziato
   +3180B/+26bl».
5. Ogni atto A/B registri la ricetta ESATTA (env incluso) del braccio B.

## Recepimento (stessa sessione)
- Azione 1 ESEGUITA SUBITO (REGOLE §9): due righe a catalogo in
  PHPR_DIVERGENCES_FROM_PHP.md (§3.23 debug_backtrace residui options;
  §3.24 debug_print_backtrace limit ignorato).
- Azione 4 applicata a WP_SESSION_150/NEXT_SESSION (wording «localizzato per
  eliminazione»); i verdetti .out restano atti immutati.
- Azioni 2, 3, 5 e il rilievo 4 (modello del tetto) in NEXT_SESSION §S-151/
  aperture.
- Incidente 18 già contato in autonomia (flip-handler non collaudato nei rami).
