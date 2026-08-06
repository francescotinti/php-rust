# Team-LEVA (fase 2, Concilio WP-106) — relatore: sintesi Bak (v5) + Leijen (v7) + Hejlsberg (v4)

I verbali individuali restano vincolanti; questa nota è un indice di convergenza/conflitto.

## (a) Convergenze

1. **Leva S-105 = inline-storage args di calls, forma SmallVec/array inline 2 slot** (A-BA-106-1, A-LE-106-2). **Pool/freelist utente ESCLUSO senza gara**: conserva l'indirezione (Bak c), re-implementa il fast path TL di mimalloc con guadagno sotto banda-layout (R-LE-106-3) — non si misura nemmeno.
2. **Enumerazione delle FUGHE prima dell'implementazione**: func_get_args, varargs conservati, by-ref sugli arg, catture di closure — con fixture per ciascuno (Bak c-ii "audit di fuga" ≡ A-LE-106-4). Rischio nominato: stabilità d'indirizzo ⇒ parity.
3. **Disasm prima/dopo OBBLIGATORIO** (bl-count + taglia run_loop, Δtesto nominato): Bak c-iii, A-LE-106-2, KS-HE-106-2. La leva non deve gonfiare codice.
4. **«icache-bound» declassato a IPOTESI**: nessun contatore HW; tre meccanismi indistinti (icache / pressione registri / BTB-layout) — R-BA-106-2 ≡ R-HE-106-1; KS-HE-106-1 rende VOID ogni frase futura senza contatore fetch/miss.
5. A/B su calls con **N EMESSO dal run e pavimento SUO** (non ereditato da prop); Δ solo in ns/iter, conversione in rapporto a valle.

## (b) Conflitti

- **Gate d'apertura della leva.** Bak: SiteTag residuo≡0 + istogramma arità + audit-fuga PRIMA di implementare. Leijen: la probe **cap-bump 2→4 (~45′)** sostituisce il SiteTag pieno (attesa: massa (16,32]→(32,64] 1:1); SiteTag → backlog solo se probe ambigua; la garanzia «nessun canale nascosto» migra al gate di promozione (KS-LE-106-1: census **alloc/chiamata 1,0000→0,0000** co-primario col timing). Nota relatore: i due gate sono componibili — Leijen sostituisce solo il TAG, non l'audit-fuga né l'arità.
- **Attesa quantitativa.** Leijen pre-registra **Δ∈[6,14] ns/iter** con soglie schema H-C2 (Δ≥8⇒R=5; [4,8)⇒R≥9). Bak chiede solo segno 5/5 + disasm, senza banda numerica. Da comporre nel criterio, non in conflitto letale; vince la versione più stringente (entrambe).
- **Contatori HW: prerequisito o braccio parallelo?** Hejlsberg: prerequisito ASSOLUTO — «precede ogni leva di codice» (priorità 1, A-HE-106-1, ~2h, binari già in stash). Bak: prerequisito solo per ristrutturazioni run_loop-wide (superistruzioni H-C3 GATED, threaded-dispatch vietato); la leva args passa PRIMA (ritmo). Leijen: non li richiede affatto per la leva args. Lettura del relatore: la leva args è sul canale ALLOCAZIONE, non sul volume di codice ⇒ contatori = braccio parallelo breve in S-105, ma PREREQUISITO per qualunque leva icache (PGO, outlining, H-C3).

## (c) Priorità S-105 (fronte leva, sequenza proposta)

0. **Atto zero**: correzioni a registro (R=7 «indeterminata», A-LE-106-5; «icache-bound»→ipotesi, KS-HE-106-1) + fingerprint v3 con rustc/profilo (A-HE-106-3).
1. **Probe cap-bump** (≤45′) + istogramma arità; SiteTag solo se ambigua.
2. **Audit-fuga**: enumerazione siti + fixture (verdi prima di toccare la VM).
3. **Implementazione SmallVec inline-2** (minima variazione semantica).
4. **A/B su calls**: pavimento suo, N emesso, attesa Δ∈[6,14], segno, disasm prima/dopo.
5. **Gate promozione**: timing E census 0,0000 (KS-LE-106-1) + gate PIN + coppia WP (salda il debito S-103).
6. **Parallelo breve** (se budget): contatori L1I/retired/mispredict su prop (A-HE-106-1 ≡ A-BA-106-2) — sblocca la mappa delle leve di codice per S-106.
