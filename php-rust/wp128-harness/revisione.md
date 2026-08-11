# Revisione S-128 — lente MISURA

## Reperto principale
Il bordo alto del «nuovo riferimento» poggia su una gamba con regime di scheduling anomalo e NON sorvegliato. Nei `.time` di leg1-off gli involuntary context switches sono 502.715 (phpr) e 1.405.510 (oracle) contro ~170–196k e ~686–690k delle altre tre gambe: 2–3× su ENTRAMBI i lati. È esattamente la gamba che produce sia 1,909 (full) sia 2,539 (media). La ricetta pair109 non ha il gate contesa in ictx/s che compoff invece ha: gate_void=0 è quindi cieco su questo. L'intervallo 1,758–1,909 non è confrontabile con 1,815–1,896 di S-125 (spread oracle 1,2%): il bordo alto misura la contesa, non il motore. Le coppie proprie dicono 1,765–1,805 su tre gambe pulite.

## Reperti secondari
- **Doppia «media» nei verdetti**: `cross-ratios.out` pubblica media_ratio 2,494/2,412 (user+sys, non etichettato); il canonico user-only 2,539 sta solo nei `ratios.out` per gamba. Due cifre «media» meccaniche divergenti nello stesso harness.
- **Collaudo F1 incoerente col criterio**: il criterio p.2 ordinava «−1,00 ESATTO» su 5 categorie con rif. pre-F1 13−9=4; −1 uniforme lascia Δins_alloc=4, ma il verdetto misura 12−7=5 (objalloc −2) e dichiara [F1-OK] con «atteso 5,00». O il riferimento del criterio è errato o l'indagine ordinata («scarto ⇒ indagine») non è a verbale.
- **Compoff: metrica non omogenea col full**: canonica NET user-only 1,863–1,891, ma sys≈2,3 s ≥ user oracle su entrambi i lati; col criterio del full (user+sys) la cifra sarebbe ~1,32. Le due headline di sessione non sono la stessa specie di numero.
- `rustc=` vuoto nelle identity (già S-125): identità del toolchain non chiusa.

## Vagliate e respinte
- «Early-stop R=2 inconcludente»: respinta — segno B-più-lento replicato in 4 coppie ABAB su 2 smoke indipendenti (D=−6,7 e −16,7); R=5 raffinava la magnitudine, non il segno.
- «objalloc identico al centesimo tra smoke1/smoke2 = dati riusati»: respinta — mtime dei `.out` distano ~400 s; rerun reale, categoria stabilissima.
- «Compoff sotto-scala come coll»: respinta — denominatore netto 1,60 s con floor med3 e spread inter-gamba 1,5%, due ordini sopra la risoluzione.

## Azioni S-129
1. Aggiungere il gate contesa ictx/s per gamba alla ricetta pair; gamba segnalata ⇒ NULLA/rimisura.
2. Rimisurare una gamba off pulita e ripubblicare il riferimento citando coppie proprie + matrice.
3. Etichettare user+sys/user-only in `cross-ratios.out`; una sola «media» canonica.
4. Riconciliare a verbale il collaudo F1 (objalloc 9→7 vs «−1 esatto») coi raw s127.
5. Dichiarare in PERF_MAP la non-confrontabilità compoff NET user-only ↔ full user+sys.
