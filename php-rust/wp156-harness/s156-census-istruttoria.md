# S-156 istruttoria census — attese (b) e (d) FUORI, spiegate al sorgente (clausola criterio p.5)

## Esito in una riga
Il census è VALIDO (identità §3 OK ×2, repliche identiche); l'attesa
«class_exists ≈ chiamate×1» era DI NUOVO mal posta: mancava la componente
MISS/AUTOLOAD. Lo scope s149 è RAII con nesting (`memcensus.rs:1968`:
un hostcall annidato ri-attribuisce a sé, ma il run_loop dell'AUTOLOADER
dentro class_exists resta attribuito al nome). Nessun sospetto sulla cura
CE1: il suo Δ è esatto e nella direzione giusta.

## Contabilità (INTERA ESATTA dai due census s154→s156)
- Δ class_exists = 9.740.728 − 7.280.781 = **2.459.947** = chiamate×1 +
  chiamate_true×1 (CE1: −1 to_vec su tutti i rami, −1 lowercase sul ramo
  true) ⇒ **chiamate ∈ [1,23M; 2,46M]** (mix forme non sciolto — il census
  conta ALLOC, non chiamate; lo scioglimento richiederebbe un contatore di
  chiamate per nome, NON in questo probe).
- Residuo E = 7.280.781 − chiamate ∈ **[4,82M; 6,05M]** = allocazioni del
  cammino miss/autoload (loader utente in run_loop annidato) attribuite al
  nome. Meccanismo nominato al sorgente, magnitudine NON ripartita oltre
  l'intervallo. Fetta candidata futura per NOME (miss-path class_exists).
- (d) coerente: la «caduta minima 4,87M» derivava dall'attesa mal posta;
  caduta vera hostcall_n = **4.539.071** = Δ class_exists 2.459.947 +
  funnel CE1(b) su nomi terzi ≈ 2,08M.

## Apporzionamento funnel CE1(b) (Δ per-NOME s154→s156, testa)
__reflect_class_real_name **−1.283.775** · __reflect_class_loc **−452.259** ·
__reflect_prop_attr_new **−97.886** · array_map −46.089 · array_filter
−16.154 · sotto-testa ≈ −182.954 · invariati: debug_backtrace, gdc,
str_replace, array_diff, __reflect_method_{info,names}, assert, implode.
L'oltre-attesa ORM di S-155 (Δ +0,41/+0,50) è ora APPORZIONATA nel canale
alloc: ~4,54M alloc in meno/run, ~45% fuori da class_exists (funnel).

## Conseguenza sul denominatore della leva p.2 (args-Vec CallHostBuiltin)
Il census dà alloc, non chiamate: N_chiamate noto solo per nomi con k
provato (bt 473k · gdc 636 · class_exists [1,23M;2,46M]). A scala ORM
l'effetto args-Vec ≈ 7 ns × N ≈ 0,01–0,05 s < risoluzione 0,293 s ⇒ la
leva si DICHIARA micro-judged (veto S-155 rispettato); il giudice è il
driver micro dedicato, non la coppia.
