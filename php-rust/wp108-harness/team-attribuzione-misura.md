# Team ATTRIBUZIONE-MISURA — Concilio WP-108, fase 2

**Membri**: Gregg (relatore), Bak, Hejlsberg · **Data**: 2026-08-07
**Fonti**: verbale-9-gregg, verbale-5-bak, verbale-4-hejlsberg, NEXT_SESSION §S-107.

## Convergenze (ID canonico unico)

- **C-AM-1 (prop −0,9)**: Gregg (R-GR-108-2/A-GR-108-1) e Bak (R-BA-108-1) convergono
  indipendentemente sulla stessa diagnosi: il dump firma direzione+meccanismo
  (BinarySTDst vive in `$s += $o->x`), la magnitudine NO — il Δ per-iter implicito
  ~12,7 ns/iter supera i 7,0 misurati su arith; l'eccesso ~5,7 è ORFANO
  (igiene D-12, reindex N_OPS=187, churn/relink, deriva oracle, tra-sere).
  → formula unica in T-AM-108-1; KS-BA-108-1 ≡ KS-GR-108-2 (stessa legge, due voci).
- **C-AM-2 (clausola ±0,4)**: Bak la refuta come attraversata senza emendamento
  (R-BA-108-2), Gregg fornisce la forma canonica sostitutiva — non è conflitto,
  è sanatoria (scritta) + norma (canonica): T-AM-108-3.
- **C-AM-3 (budget forme)**: Bak A-BA-108-3 e Hejlsberg A-HE-108-3 chiedono la
  stessa cosa da due lati (ledger a eventi / tetto combinatorio): T-AM-108-7.
- **C-AM-4 (proxy e adempimenti)**: KS-HE-108-2 (proxy col perimetro) e la linea
  Gregg su cifre attribuibili sono la stessa dottrina: T-AM-108-6.

## Conflitti risolti

- **CF-AM-1 — posizione census D-5**: Bak ratifica la sequenza 1-5 com'è (census
  nei denti del punto 1); Gregg la emenda (census DOPO §3.15). Risolto PRO-GREGG:
  il fix D-13 riscrive push_call_args, il sito che il census strumenta — censirlo
  prima significa misurare un binder che muore col fix o pagare il rerun. Il
  vincolo sostanziale di Bak («mai tornare a calls senza contatore») resta
  soddisfatto: il census precede comunque ogni lavoro calls. Bak concorda.
- **CF-AM-2 — Sweep per-iter**: Bak lo ammette «DOPO istruttoria sul perché sta
  nel corpo»; Hejlsberg lo VIETA come peephole (semantica GC, criterio proprio +
  gate d'ordine-free). Risolto PRO-HEJLSBERG (forma più forte, che assorbe la
  cautela di Bak): Sweep ESCE dalla finestra fusioni; l'istruttoria chiesta da
  Bak diventa il prerequisito del criterio GC dedicato. Candidata fusione unica
  per S-107: IncDecSlot+Pop.

## Direttive T-AM-108-n

1. **T-AM-108-1 (VINCOLANTE)** — Formula unica prop: «direzione+meccanismo firmati
   dal dump; magnitudine NON ripartita (~12,7 vs 7,0 ns/iter, eccesso ~5,7 orfano)».
   Vietato citare «H-A1 vale −0,9 su prop»; cifra ufficiale = micro sul pin.
   [Assorbe R-GR-108-2, A-GR-108-1, R-BA-108-1, KS-BA-108-1, KS-GR-108-2]
2. **T-AM-108-2 (VINCOLANTE)** — Retro-A/B prop in S-107 (~10′): ABAB R=5 sul
   giudice prop coi due stash phpr-s105 (d4d0fa52) vs phpr-s106 (eb555106);
   converte la direzione in magnitudine attribuita o smaschera il co-fattore.
   [Assorbe A-BA-108-2 prima parte]
3. **T-AM-108-3 (VINCOLANTE)** — Clausola ±0,4 emendata DICHIARATAMENTE: «fermo
   entro ±0,4 OPPURE movimento migliorativo con beneficiario nominato dal dump,
   registrato SOLO nella forma T-AM-108-1»; sanatoria S-106 scritta, non sottintesa.
   [Assorbe R-BA-108-2, A-BA-108-1]
4. **T-AM-108-4 (VINCOLANTE)** — Se l'istruttoria dichiara la finestra TRASVERSALE,
   l'A/B misura ANCHE le categorie beneficiarie nominate, nello stesso ABAB.
   Entra nel criterio della leva S-107 PRIMA del run. [Assorbe A-BA-108-2 seconda parte]
5. **T-AM-108-5 (VINCOLANTE)** — «L'inliner ripaga i due dispatch» declassata a
   osservazione; il −128 B resta narrato finché manca il diff taglie per-simbolo;
   conto per-target COMPLETO obbligatorio al prossimo admission. [Assorbe R-HE-108-2]
6. **T-AM-108-6 (VINCOLANTE)** — «Micro su hash₁» dichiarato proxy arith-only,
   NON R-1 pieno; le cinque categorie restanti su hash₁ in voci aperte. Ogni proxy
   futuro si dichiara col perimetro che NON copre. [Assorbe R-HE-108-3, KS-HE-108-2]
7. **T-AM-108-7 (VINCOLANTE)** — Ledger forme monomorfe al primo pin S-107:
   hit-count census per-forma (giudici+corpus), forme ~0 flaggate fredde; sveglia
   (+8 forme da S-104 O run_loop +4 KB) ⇒ istruttoria cold-partition/outlining o
   pivot superistruzioni-da-census; ogni istruttoria pre-registra il budget forme.
   [Assorbe A-BA-108-3, KS-BA-108-2, A-HE-108-3]
8. **T-AM-108-8 (VINCOLANTE)** — Coppia WP in S-107 DOVUTA: il «se» del punto 4
   si cancella (condizione già vera). Istituita KS-GR-108-1: voce fuori banda
   senza rerun per 2 sessioni = debito FONDAMENTALI con data; alla terza blocca
   la promozione della leva successiva. [Assorbe A-GR-108-2, R-GR-108-3, KS-GR-108-1]
9. **T-AM-108-9 (VINCOLANTE)** — Leva S-107: candidata fusione = IncDecSlot+Pop
   (stessi helper, +1 braccio, budget text regge). Sweep per-iter VIETATO come
   peephole: ammesso solo come istruttoria propria con criterio GC dedicato +
   gate d'ordine-free. [Assorbe A-HE-108-2; risolve CF-AM-2; ordine Bak assorbito]
10. **T-AM-108-10 (RACCOMANDAZIONE)** — arr: registrare il SEGNO oltre alla banda;
    drift monotono stesso verso su N≥3 sere oltre metà banda = trend ⇒ istruttoria.
    Una banda difende dal rumore, non dalla deriva. [Assorbe punto minore Bak]
11. **T-AM-108-11 (RACCOMANDAZIONE)** — Banda-layout: il contrasto 4,99/4,97 vale
    come punto N≥3 solo se ripreso stesso-protocollo; un contrasto = UN punto;
    disambiguare «due punti colti» in NEXT_SESSION. [Assorbe A-HE-108-1]
12. **T-AM-108-12 (VINCOLANTE)** — Doc di modulo reg_lower.rs riscritto nei tre
    siti (righe 20-21, r.190, lista v3 + eccezione ST) nell'igiene del punto 1/2:
    un emendamento è completo solo quando il testo normativo smette di
    contraddire il codice. [Assorbe R-HE-108-1, KS-HE-108-1]

**Bilancio**: 10 vincolanti, 2 raccomandazioni.

## Modifiche richieste all'ordine §S-107

- **M-1**: census hit/miss D-5 ESCE dal punto 1 e diventa passo autonomo DOPO il
  punto 2, sul binario post-fix §3.15 (CF-AM-1). Sequenza emendata:
  mutanti → §3.15 → census D-5 post-fix → leva → coppia WP → igiene.
- **M-2**: nel punto 1 entrano il retro-A/B prop (T-AM-108-2, ~10′) e la
  riscrittura doc reg_lower.rs (T-AM-108-12).
- **M-3**: punto 3 — candidata SOLO IncDecSlot+Pop; Sweep rimosso dalla lista
  peephole (T-AM-108-9); il criterio recepisce T-AM-108-4 (beneficiari
  trasversali) e T-AM-108-7 (budget forme); ledger forme al primo pin.
- **M-4**: punto 4 — «se» cancellato: coppia WP DOVUTA, con data di rerun
  registrata in FONDAMENTALI (T-AM-108-8).
- **M-5**: punto 5 — banda-layout vincolata a stesso-protocollo, un contrasto =
  un punto (T-AM-108-11); registrare il segno di arr (T-AM-108-10).
