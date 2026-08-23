# s156-criterio-census.md — census ORM post-CE1 col probe s155 (PRE-REGISTRATO, prima del run)
1. Oggetto: testa alloc per-NOME post-CE1 su doctrine/orm; probe s155 `3e6b5008482c32d0` (stash ×2 verificato); copione = COPIA DICHIARATA di `s154-census-orm.sh` (manifest `s156-census-copia.diff`; adattamenti: path s156, probe s155, §5-bis sostituito dalle attese s156).
2. CONTEGGI, mai cifre di tempo; identità §3 (Σsiti==tot per canale · overlap=clsovf=0 · cons b+c==d+l per classe · s149 sum==hostcall_n, unnamed=0, overflow=0) pena census NULLO; R=2 repliche, s151tot r1==r2 o differenze DICHIARATE.
3. Attese pre-registrate (r1, plumbing args-Vec INCLUSO nel conteggio per-nome — veto S-155):
   a. debug_backtrace ∈ 6,15M ±10% (invariata da s154);
   b. class_exists ∈ [3.250.000; 4.870.000] (post-CE1 ≈ chiamate×1: il valore esatto SCIOGLIE il mix di forme [3,25M;4,87M]);
   c. get_declared_classes ∈ 4.563.808 ±2% (CE1 non tocca gdc);
   d. hostcall_n: DIREZIONE GIÙ vs 67.023.784 (attesa minima = caduta class_exists [4,87M;6,49M]; il funnel CE1(b) su nomi terzi si RIPORTA per-NOME vs testa s154, magnitudine apporzionata post-hoc, MAI attesa per-nome — veto S-155 funnel).
4. Giudice: awk del copione (identico a s154, N emesso dal sorgente); rc: 0=valido · 5=identità violate (NULLO) · 6=guardia · 7=setup · 8=probe muto · 9=lock.
5. Un'attesa FUORI ⇒ dichiarare e tornare al sorgente prima d'ogni conclusione (clausola s155); il census alloca la leva p.2 (denominatore N hostcall per args-Vec), non emette cifre di tempo.
