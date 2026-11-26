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
- **Esito N1b (26/11 02:50)**: carico intero contro C4d **ORM −1,88 %** (attesa ≤ −1,0 %: raggiunta), **WordPress −0,45 %**; accettazione `s194N1b` rc=0; commit `fab50dd2`.

## 4. N2 — l'impronta delle classi registrate, incrementale (scritto il 26/11 03:25, prima della misura)
- **Cosa succede oggi** (profilo dell'ORM sul braccio B2, `s194-profilo-orm-B2.out` + `s194-chiamanti-orm.out`): dopo N1 resta lo 0,8 % dei campioni in SipHash, tutto sotto `Op::Include`: `Vm::unit_fp`, calcolata a ogni include, fa un SipHash per OGNI classe registrata e li combina in XOR (apertura per nome «`unit_fp` O(classi) per include», S-192).
- **Cura**: lo XOR è indipendente dall'ordine, quindi si mantiene dove l'indice cambia. Campo `class_index_fp`; ogni scrittura di `Vm::class_index` passa da `class_index_put` (file nuovo `vm/class_fp`), che aggiunge il termine nuovo e toglie quello sostituito. I tre siti di scrittura (dichiarazione a run-time, `class_alias`, classe coniata da un'unità) sono gli unici: verificato per ricerca. Il VALORE dell'impronta è lo stesso di prima (stesso hasher, stessi termini): il comportamento della cache delle unità non si muove.
- **Prova** (`out/n2-copertura.sh`, braccio di sola prova da una COPIA del tree): in `unit_fp` l'asserzione «incrementale == ricalcolo intero» attiva; batteria intera, corpus Zend, una gamba ORM e una WordPress: zero scostamenti prima dell'accettazione.
- **Attese** (N2 misurato insieme a B1+B2, un solo passo «N2» contro il braccio N1b): ORM ≤ −1,0 % (B1 −0,49 % e B2 −0,11 % già letti con la prova rapida; N2 da solo atteso ≥ −0,5 %); WordPress mai sopra; sonda: nessuna fixture oltre +1,0 % rispetto a N1b.
- **Esito N2 (+B1+B2), 26/11 05:10**: prova dell'impronta incrementale = zero scostamenti (batteria, corpus, ORM, WordPress; `out/n2-copertura.done`); accettazione `s194N2` rc=0 (batteria 1789/0/2); carico intero contro N1b **ORM −1,44 %** (attesa ≤ −1,0 %: raggiunta); commit `047bbee5`.

## 5. T1 — memoria dei controlli di tipo di classe riusciti (scritto il 26/11 05:15, prima della misura)
- **Cosa succede oggi** (`s194-profilo-orm-B2.out`): `value_satisfies_class` — l'`instanceof` dietro ogni parametro, proprietà e ritorno tipizzato con una classe — vale l'1,2 % dei campioni dell'ORM: confronta i nomi senza distinzione di maiuscole e, per una sottoclasse o un'interfaccia, minuscolizza, cerca nell'indice delle classi e interroga la memoria degli antenati.
- **Cura**: tabella a indirizzamento diretto (1024 voci, file nuovo `vm/hint_memo`) delle coppie (id di classe, nome dichiarato) GIÀ soddisfatte. Si ricordano solo i successi: un successo resta vero per tutta la corsa (id, antenati e legame nome→classe non cambiano), mentre un nome non ancora dichiarato può esserlo dopo. `static` non passa dalla tabella (dipende dal frame). Si svuota al confine di richiesta, con `iof_cache`.
- **Attese**: prova rapida contro il braccio N2: ORM ≤ −0,3 %; poi catena intera (WordPress mai sopra). Dente `s194-t1-comportamento.php` == oracle: classe esatta, sottoclasse, interfaccia, classe sbagliata, nullable, nomi con maiuscole diverse, `static`/`self` di ritorno, classe dichiarata DOPO un primo controllo fallito, `class_alias`, quaranta classi sorelle sugli stessi siti.
- **Divergenza preesistente trovata scrivendo il dente** (pin compreso, non curata, apertura per nome): `self` come tipo di PARAMETRO non è verificato (`$a->same(new C)` passa; PHP lancia TypeError).
- **Esito T1 (26/11 06:40)**: prova rapida contro N2 ORM −1,25 %; catena: accettazione `s194T1` rc=0 (batteria 1790/0/2, corpus ×2 == congelato); carico intero contro N2 **ORM −1,41 %** (attesa ≤ −0,3 %: superata), **WordPress −0,08 %** (non sopra); sonda `call_typed_iface` −8,3 %; commit `a0a74e9c`.

## 6. Somma della notte 25→26/11 (istruzioni sul carico intero, bracci con la ricetta del pin)
| passo | commit | ORM | WordPress |
|---|---|---|---|
| C4 + C4b + C4d | `29b6d962`, `9b9b5f11` | −0,59 % · −0,02 % · −0,07 % | −0,70 % · −0,06 % · −0,11 % |
| N1b | `fab50dd2` | −1,88 % | −0,45 % |
| N2 (+B1+B2) | `047bbee5` | −1,44 % | −0,87 % |
| T1 | `a0a74e9c` | −1,41 % | −0,08 % |
| **contro il pin s189** | | **225,350 → 213,463 G = −5,27 %** | **3534,325 → 3455,316 G = −2,24 %** |
| contro il pin s188 | | 240,009 → 213,463 G = −11,06 % | 3731,438 → 3455,316 G = −7,40 % |

Sono istruzioni (lavoro eseguito), non tempo: il verdetto di tempo è la notte di conferma.

## 7. N3 — un solo buffer per passata (scritto il 26/11 06:50, prima della misura)
- **Cosa succede oggi** (`s194-profilo-orm-T1.out`, chiamanti di `libsystem_platform`): le tre passate curate in N1b (`seed_stub_mask` 0,5 %, `unit_remap_elided` 0,4 %, `get_declared_classes` 0,6 %) azzerano un buffer di 128 byte PER CLASSE del seed (`[0u8; 128]` dichiarato dentro il ciclo): l'allocazione è sparita, è rimasto un `bzero` per voce.
- **Cura**: il buffer si dichiara una volta fuori dal ciclo. Nessun cambio di comportamento.
- **Attese**: carico intero contro il braccio T1: ORM ≤ −0,5 %; WordPress mai sopra. Errore mio in N1b: il buffer andava fuori dal ciclo dall'inizio.
- **Esito N3 (26/11 07:55)**: accettazione `s194N3` rc=0 (batteria 1790/0/2); carico intero contro T1 **ORM −0,02 %, WordPress −0,02 %**: attesa (≤ −0,5 %) **MANCATA**. L'ipotesi era sbagliata: il tempo attribuito a quelle righe è la copia del nome nel buffer (inevitabile finché si minuscolizza per voce), non l'azzeramento. Il passo è innocuo e resta (commit `6b5c0095`); la cura vera delle passate O(seed) per include resta l'apertura per nome già registrata (nome minuscolo precalcolato sulla dichiarazione, o contratto WP-63/64).
- **Somma aggiornata contro il pin s189**: ORM 225,350 → 213,450 G = **−5,28 %**; WordPress 3534,325 → 3454,954 G = **−2,25 %** (contro s188: −11,07 % e −7,41 %).

