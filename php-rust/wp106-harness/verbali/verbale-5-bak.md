# Verbale sedia 5 — BAK (VM di produzione: alloc-rate, code-cache, path caldi) — Concilio WP-106 su S-104

## VERDETTO

S-104 è metodologicamente onesta (A/B ×2, disasm prima/dopo, revert al byte)
e il RITIRO della leva H-C2 è corretto. Ma due claim vanno declassati:
«icache-bound» da FATTO a IPOTESI NOMINATA (nessun contatore hardware), e
«le 1101 chiamate erano quasi gratis» da firma a NON-FIRMATO. La mia R1
(«senza fusione il Δ può essere ~0 o negativo») è confermata nel segno ma
la sessione ha comprato il NETTO, non i componenti.

## R-BA-106-1 — L'A/B ha misurato una SOMMA confusa, non la leva

B cambia due cose insieme: il fast-out (9 siti) E il modo codegen globale
(flip inliner, bl 1101→0, +8KB). B>A 5/5 prova solo che (guadagno
chiamate) < (costo +8KB). Con costo-chiamate 5 ns e costo-testo 16 ns il
verdetto sarebbe identico: «quasi gratis» NON è firmata. Una terza build a
inlining PINNATO (shim noinline che conserva bl≈1101 coi fast-out attivi)
avrebbe separato i due addendi; il divieto di «iterazioni cieche» non copre
un braccio di CONTROLLO pre-registrato. Operativamente la CADUTA regge
(la leva non è realizzabile sotto l'inliner reale): refutazione MAGGIORE,
non capitale.

## R-BA-106-2 — «icache-bound» inferito senza contatori

+8KB con dispatch duplicato in 1101 siti carica anche BTB/predittore
(1101 alberi ldrb+cmp invece di 1 corpo caldo condiviso). Il rimedio
diverge: se L1i (run_loop 257KB > 192KB L1i M-serie) → taglia handler; se
predittore → fusione/conteggio rami. Una run con contatori (L1I-miss,
branch-mispredict) arbitra in minuti.

## Risposte al mandato

**(a)** La conclusione «ridurre volume di codice/op» è giusta nella
DIREZIONE, generica nella FORMA. Threaded/computed-goto è CONTROINDICATO:
duplica il dispatch in coda a ogni handler (+testo — la cura peggiora la
malattia se davvero icache-bound) e sui predittori Apple moderni il
guadagno classico è ridotto. I rimedi compatibili: (1) handler più piccoli
— cold-split sistematico dei rami rari (già dimostrato quasi-free: il glue
outlined lo era); (2) SUPERISTRUZIONI sulla meccanica di pila (26,6%): il
census S-103 mostra coppie fondibili (BinaryAdd consuma rhs+sovrascrive
top; CmpJmpSC; IncDecSlot) — la fusione riduce op E dispatch E morti Zval
intermedie: è l'unica leva che paga in TUTTE le ipotesi di collo.

**(b)** L'ha misurata l'euristica dell'inliner insieme alla leva (R1
sopra). Sì, la terza build pinnata avrebbe separato; si compra solo se
gate di una leva futura, non per curiosità.

**(c)** Inline storage vince sul pool SENZA gara: il free-hist dice taglia
UNIFORME 32 B, 1/1 per chiamata — il pool conserva l'indirezione e
aggiunge free-list; le VM di produzione non allocano args sull'heap affatto
(args nello stack di valori del frame). Criterio pre-registrato: (i)
SiteTag residuo≡0 conferma args-Vec = 100% del canale; (ii) istogramma
ARITÀ dal census + audit di FUGA dell'indirizzo (by-ref sugli arg,
func_get_args, closure che catturano) — se un indirizzo di arg sopravvive
alla chiamata, l'inline storage cambia stabilità d'indirizzo = rischio
parity, fixture dedicata; (iii) A/B su calls (N EMESSO), pavimento SUO
(non ereditare il 4 di prop), segno 5/5, disasm prima/dopo OBBLIGATORIO
(bl-count + taglia run_loop, Δtesto nominato); (iv) gate PIN + coppia WP
(salda il debito).

## Emendamenti

- **A-BA-106-1**: leva S-105 = H-D inline-storage (SmallVec-like 2 slot)
  col criterio (i)-(iv); il pool si nomina SOLO se l'audit di fuga boccia
  l'inline.
- **A-BA-106-2**: braccio CONTATORI (una run: L1I-miss + mispredict su
  prop) PRIMA di ogni ristrutturazione run_loop-wide; se L1i non domina,
  la mappa «icache-bound» si riscrive.
- **A-BA-106-3**: design superistruzioni pila (coppie dal census) =
  candidato H-C3, GATED sul braccio contatori; threaded-dispatch VIETATO
  come leva finché la valuta è testo.

## KS-BA-106-1

Nessuna affermazione futura su COMPONENTI di costo (chiamata vs icache vs
predittore) da un A/B che cambia il modo codegen globale: o braccio a
inlining pinnato o contatori hardware — altrimenti si firma solo il NETTO.

## Priorità S-105

1) SiteTag H-D → leva inline-storage con A/B ESEGUITO (regola di ritmo);
2) braccio contatori (breve); 3) superistruzioni = solo design; 4) 21,2%
senza nome resta in coda, per NOME.

**Refutazioni capitali: NO** (due maggiori; l'azione di S-104 resta valida).
