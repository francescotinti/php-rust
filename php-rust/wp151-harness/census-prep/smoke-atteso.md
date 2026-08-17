# smoke151 — ATTESI dichiarati PRIMA di leggere l'output (S-151, criterio §3)

Scritto DOPO la build del probe e PRIMA di qualunque esecuzione/lettura del
suo output census. Ogni voce è una guardia fail-closed del verificatore
meccanico `smoke151-check.sh`. Voci marcate **[bounded]** = disuguaglianza
dichiarata col MECCANISMO che impedisce l'esattezza (ammesse dal criterio solo
dove l'esatto dipende dal lowering interno, non dal canale sotto collaudo).

## Run A — smoke151.php (probe, PHPR_MEM_CENSUS attivo)
1. rc==0; stdout ESATTAMENTE `SMOKE151 r=42\n`.
2. **Identità per canale (KS-G1)**: per OGNI canale c1..c5,
   Σ righe `s151site channel=<c> ... n=` == `s151tot channel=<c> n=` (esatto,
   anche quando 0==0).
3. `s151overlap n=0` · `s151clsovf n=0` · `s151snap taken=1` (esatti).
4. `s151cons class=K`: births==9 · conservazione births+clones==drops+live_end
   ESATTA · live_end==3 (handle forti a fine request: slot $keep + slot $alias
   + store `created`).
5. `s151n6 class=K`: objects==1 · deaths==9.
6. Distribuzione props (N2): `s151props` n==9, sum==27, p50==3, p90==3,
   p99==3, le4_pct==100.00, le8_pct==100.00, dyn_objs==0, dyn_entries==0;
   UNICA riga `s151propshist bucket=3 n=9`; UNICA riga `s151dynhist bucket=0 n=9`.
7. C3: riga `s151site channel=c3 site=alloc_instance` presente con n>=18
   **[bounded: 2 malloc/oggetto (header Rc + Vec slots) × 9; il PRIMO `new K`
   paga in più la costruzione dello stampo props della classe]**.
8. C5: Σ su tutti i siti di `*.scalar.drop` == 1 (esatto: l'unico valore
   spiazzato è il Long(1) di `$keep->a = 42`); Σ `*.scalar.clone` >= 1
   **[bounded: il lowering può clonare il valore anche in transito]**.
9. C1: Σ `*.mint` == 0 (esatto: nello smoke nessun weak-upgrade né carrier
   sintetico).
10. C2: `s151tot channel=c2` >= 1 **[bounded: i borrow della macchineria
    (store insert, gc_track, prop access) non sono enumerabili a priori]**.
11. C4: `s151tot channel=c4` >= 1 **[bounded: `$drop = null` spiazza un
    OGGETTO ⇒ almeno una gc_note(Object); il teardown può aggiungerne]**.
12. Emissioni EREDITATE vive nello stesso run: identità s148
    `galloc_n==sum_n` (esatta) e s149 `hostcall_n==sum_name_n+unnamed_n`,
    `unnamed_n==0`, `overflow==0`.

## Run B — wp149-harness/smoke149.php (eredità s148/s149, stesso probe)
13. `s148tag` name=frame/hostcall/arrgrow tutti con n>=1; identità s148 esatta.
14. `s149name` str_repeat e sprintf con n>=1; identità s149 esatta
    (hostcall_n==sum_name_n+unnamed_n, unnamed==0, overflow==0).

Nessun altro numero verrà usato come gate. Se una guardia ESATTA fallisce:
STOP fail-closed, meccanismo da nominare in PREP.out; una ri-dichiarazione
(dopo fix di un difetto del probe o di un errore di derivazione) va REGISTRATA
in PREP.out con la causa — mai aggiustata in silenzio.
