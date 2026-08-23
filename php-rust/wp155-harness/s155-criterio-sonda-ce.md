# Criterio S-155 p.2 — sonda ce-count post-CE1 (DOVUTA: criterio ce1 p.4, FUORI-UB D=22,0 > 13,8) — PRE-REGISTRATO prima del run

1. **Scopo**: CONTEGGI mai tempo. k alloc/chiamata di class_exists sul probe
   census RICOSTRUITO dal tree CON CE1 (il probe s154 b2889909 è PRE-CE1:
   inutilizzabile per questa domanda). Strumento EREDITATO INVARIATO:
   `wp152-harness/ce-count.php` (2 chiamate/iter: hit 'CeProbe' + miss
   'Nope\Missing', entrambe ≤64 B, autoload=false), due N (100000/300000),
   k = Δn/Δchiamate (Δchiamate = 2·ΔN); baseline s152: k=2 ESATTE.
2. **Ricetta**: `s155-sonda-ce.sh` = COPIA DICHIARATA di s154-sonda.sh
   (copia-gate rc=0, manifest s155-sonda-copia.diff): S0 guardie (lock, no
   cargo/rustc altrui, pin s154 bddc050320a6af4c + b3cf348f69739edc, no HOLD)
   · S1 copia-fedele · S2 identità a CONTENUTO vs pin (cluster nominati
   LC_UUID/firma/banner, emenda §1-bis) · S3 diff census s151 con offset ·
   S4 build probe mem-census (target APFS dedicato, stash ×2) · S5 smoke
   checker s151 attesi INVARIATI · S6 ce-count due N stdout ESATTO
   (`CE-COUNT-OK n=$N hit=$N`) · S7 k + riconciliazione.
3. **Attesa k PRE-REGISTRATA**: k 2→**0** (hit ≤64 B; il ramo miss
   autoload=false condivide il lookup LcKey — k>0 ⇒ si torna al sorgente
   prima d'ogni conclusione, lezione s154-sonda). k INTERO ESATTO atteso.
4. **Riconciliazione FUORI-UB (meccanica)**: D_ce1=+22,0 (ab-ce1b);
   D_alloc=(2−k_post)×miheap [6,7;6,9]; residuo R=D−D_alloc: canali candidati
   da NOMINARE (memcpy del to_vec evitato · byte-loop to_ascii_lowercase
   evitato · hash su chiave stack) — NESSUNA cifra per canale senza mock
   dedicato.
5. **Gate a rischio morso PRE-DICHIARATI (az.rev. S-154 #3)**: S2 identità
   (il pin s154 è nato da build di promozione: cluster attesi LC_UUID+firma
   ±banner) · S3 apply con offset su tree avanzato (CE1 tocca mod.rs) — un
   hunk respinto = STOP rc=7, niente fix-a-mano non dichiarato.
6. **Sequenza**: parte SOLO a coppia p.1 conclusa (build in finestra di
   misura = veto); rc autoritativo da sonda-out/sonda.rc.
