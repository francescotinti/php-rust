# Verbale Sedia 9 — Gregg (metodologia di misura, attribuzione) — Concilio WP-100, MANDATO INVERSO

**Oggetto giudicato**: S-98.0. Domanda: che cosa sappiamo di phpr oggi che ieri non sapevamo?

## VERDETTO

**APPROVATO CON EMENDAMENTI.** Dal punto di vista dell'OGGETTO è la sessione più densa da settimane: tre numeri nuovi e veri su phpr, di cui uno negativo (il più raro). Ma due refutazioni capitali sul modo in cui quei numeri vengono RACCONTATI.

## (a) Inventario: conoscenza dell'oggetto vs apparato

Conoscenza NUOVA dell'oggetto: (1) **il preambolo di dispatch costa ~0 su questo core OoO** (P=0%, banda [0–6,7%]; la split-borrow perde 0,53 ns/op sotto pressione) — conoscenza negativa solida, chiude un intero asse senza codice; (2) **anche gli op economici costano 6,27 ns contro 1,07** (noop 200M): i corpi dominano ovunque; (3) **D=6,07 ns/occ è il prezzo del plumbing generico di un Binary** — prima attribuzione POSITIVA del fattore ~8; (4) l'ASM del loop head decomposto 11/17. Apparato (utile ma non-oggetto): reg_lower_funnel, bin_op_of, M5, ricostruzione census. Rapporto oggetto/apparato: il migliore da WP-94.

## (b) Igiene

La serie bimodale (build concorrente) è dichiarata nel .out con la ⚠️ e scartata. La coppia add È rimisurata da zero su ENTRAMBI i binari nella stessa finestra: il .out lo dice esplicitamente («COPPIA rimisurata da zero su ENTRAMBI i binari nella stessa finestra», R=5+R=5, baseline dallo stash verificato per hash, deriva inter-build 0,5% via noop, misure riprodotte sul pin finale). La baseline early-window (5,64) e quella rimisurata (5,63) coincidono allo 0,2%: omogeneità DIMOSTRATA, non assunta. Su (b) non refuto nulla.

## (c) Bande — refutazione capitale n.1

`arith` in banda [−3,−5]% per KS-GR-99-2 è TROPPO prudente nel caso specifico: con R=5 c'è **separazione di rango completa** (tutti i 7,50–7,66 sotto tutti i 7,88–7,94; p≈1/252). Il 3×spread sulla gamba peggiore, gonfiato da UN outlier, butta informazione che il rango conserva. Prudenza in direzione sicura, ma un criterio futuro che pescasse da quella banda erediterebbe l'imprecisione.

**La refutazione capitale è l'altra metà**: il titolo «add 14,5→12,1» ha la **gamba oracle STANTIA** (0,39 misurato nella finestra baseline, MAI rimisurato in coppia post-build) — e «arith 18,1→17,1» lo ammette solo in commento. Il progetto ha istituito la regola della coppia stessa-finestra e la viola nel proprio titolo. Il claim pulito è D=6,07 (phpr-vs-phpr); i RAPPORTI sono nominali e vanno etichettati tali. Inoltre str/re/prop/calls flag-off NON rimisurati dopo il cambio di emissione: i criteri di attivazione di H-C/H-D («prop resta sopra 5×») galleggiano su baseline morte.

## (d) Attribuzione — refutazione capitale n.2

D=6,07 è una differenza pulita; la scomposizione call/marshalling/pop-push NON serve prima del rollout: il criterio pre-registrato per occorrenza basta come go/no-go. MA il testo dice «le forme registro flag-on hanno lo stesso plumbing da togliere — è lì che compone col −30,7%»: **ipotesi spacciata per fatto**. BinaryDst/SS/SC non fanno pop/push: la quota rimovibile flag-on è per costruzione DIVERSA, e la lezione di M1 dice che il controfattuale statico può essere nascosto dall'OoO. D=6,07 NON è trasferibile.

## (e) FONDAMENTALI

NEXT_SESSION punto 1 dice «DOVUTO, non rinviabile»: nessuna scusa lì. La crepa è in WP_SESSION_98: «alla prossima occasione utile» — linguaggio che ha già fatto slittare collaudi quattro volte (WP-94 è 4 sessioni fa). E il punto 3 (rollout) è il più attraente: senza un dente, la tentazione di invertire l'ordine è strutturale.

## Emendamenti

- **A-GR-100-1**: ogni rapporto phpr/oracle pubblicato senza gamba oracle stessa-finestra porta l'etichetta obbligatoria «nominale, gamba oracle stantia»; il 14,5→12,1 la riceve retroattivamente.
- **A-GR-100-2**: i criteri di caduta del rollout flag-on si derivano da una misura FLAG-ON del controfattuale; D=6,07 non si ricicla.
- **A-GR-100-3**: KS-GR-99-2 si integra col criterio di rango (separazione completa a R=5 ⇒ stima puntuale con banda, non banda sola).
- **A-GR-100-4**: al collaudo S-99, ri-baseline delle sei categorie flag-off su entrambi i motori stessa-finestra (rianima i criteri H-C/H-D).

## Kill-switch

- **KS-GR-100-1**: in S-99 nessuna spedizione del rollout H-B2 né alcun claim CPU nuovo PRIMA che il collaudo di parità WordPress (punto 1) sia chiuso per NOME; violazione ⇒ VOID.
- **KS-GR-100-2**: VOID ogni occorrenza del rollout il cui criterio è derivato dal D flag-off invece che da misura flag-on.
