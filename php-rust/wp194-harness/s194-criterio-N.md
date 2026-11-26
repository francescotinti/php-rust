# S-194 — pacchetto N («new», hash, builtin costosi): criterio

Generato con assistenza AI (Claude Fable 5.2). Scritto il 26/11 01:15, PRIMA della misura.

## 0. Da dove viene (registro fresco e profilo dell'ORM sul commit 9b9b5f11)
- `s194-registro-op-C4d.out` (istruzioni per op, gamba ORM intera) e `s194-profilo-orm-C4d.out` + `s194-chiamanti-orm.out` (samply, direzione).
- Rettifiche al mandato del 25/11 sera: **(a)** i builtin NON sono più cercati per nome a ogni chiamata (curato dal pacchetto D di questa sessione); **(b)** il ritorno con tipo esatto è già fuso (C1b): il tag del registro «Ret: generale, tipo di ritorno dichiarato» conta TUTTI i ritorni tipizzati, fusi compresi (15,6 M, ~230 istruzioni in più di un ritorno non tipizzato).
- Trovato: **SipHash** (hasher di default della libreria standard) su cammini caldi — 1,8 % dei campioni dell'ORM: `next_id` (due mappe `std` consultate a ogni `new` che riusa un id), l'include (`seed_conditional`, `conditional_fns/classes`, `attempted`), la cache di `__reflect_method_info`. `new C` costa 3031 istruzioni nette contro 392 dell'oracle (7,7×); `strpos` 1203 istruzioni per chiamata (2,35 M chiamate) perché `memmem::find` costruisce un cercatore a ogni chiamata.

## 1. N1 — cura
1. `next_id`: ogni `remove` delle dodici mappe per-id solo se la mappa non è vuota (nessun hash nel caso comune).
2. Insiemi dell'include e cache di reflection: da `std::collections` a `FxHash` (l'ordine di iterazione di una mappa `std` è casuale per processo: nessun comportamento può dipenderne).
3. `StampThrowable`: il controllo «è un Throwable» senza clonare l'handle; il clone solo per i Throwable.
4. `find_sub` (strpos, str_contains, strstr…): ago di un byte → `memchr`; pagliaio ≤ 256 byte → `memchr` sul primo byte + confronto; il resto come prima.

## 2. Attese
- Sonda (contro la lettura del pin s189, `s194-primitive-C4base.out`): `new_obj` ≤ −8 %, `new_ctor` ≤ −4 %, `builtin_strcont` ≤ −30 %; nessuna fixture oltre +1,0 % rispetto a C4d.
- Carico intero contro il braccio C4d: ORM ≤ −1,0 %; WordPress mai sopra.
- Dente: `s194-n1-comportamento.php` (strpos/str_contains/strstr/stripos/substr_count con ago vuoto, di un byte, più lungo del pagliaio, a cavallo della soglia 256, offset negativi; Throwable e non-Throwable creati in ciclo con riuso degli id), atteso = oracle.

## 3. N1 → N1b (26/11 01:52)
- **N1, sonda** (braccio `c3a164e9cbcc774c`): `new_obj` −11,3 %, `new_ctor` −5,9 %, `p_ro_ctor` −4,3 % (attese raggiunte); `builtin_strcont` −2,1 % contro −30 % atteso: **MANCATA**, la fixture usa un ago di un byte, che `memmem` serviva già con `memchr`; il guadagno della cura 4 è sugli aghi di più byte e si legge solo sul carico intero.
- **N1, accettazione ROSSA**: solo il dente LOC (`php-builtins/src/string.rs` 2878 righe > cap 2865; corpus ×2 verde, batteria per il resto verde). Errore mio: il dente LOC (0,05 s) va girato PRIMA della catena. Cura: la funzione vive nel file nuovo `php-builtins/src/search.rs`.
- **N1b aggiunge (scritto prima della misura)**: 5. le tre ricerche per nome di classe che ogni include fa su TUTTE le classi del seed (`seed_stub_mask`, `unit_remap_elided`, più `get_declared_classes`) minuscolizzano in un buffer di pila invece di allocare un vettore per classe (`lc_name`). Attesa invariata sul carico intero (ORM ≤ −1,0 %).

