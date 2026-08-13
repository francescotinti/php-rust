# Criterio S-135 p.3a — bisezione objmap (ISTRUTTORIA, non leva) — commit PRIMA del run

1. Oggetto: decomporre il **+163,3 ns/iter** di objmap (173,3 phpr vs 10,0
   oracle, s134-submicro) in canali NOMINATI, bilateralmente (mai un lato
   solo). Scelta dai numeri di p.1-2: le suite non si muovono con le leve
   typed-set; il profilo ORM indica insert/lookup + clone/drop; objmap 17,3
   è la voce peggiore ed è un `$arr[int] = $obj` in overwrite.
2. Varianti (`micro-bisez/`, stesso preambolo di objmap, N=3e6, stessa
   statement-shape): **m0_obj2048** = objmap (chiave `$i&2047`, valore
   oggetto) · **m1_int2048** (stessa chiave, valore int) · **m2_obj0**
   (chiave fissa 0, valore oggetto) · **m3_int0** (chiave fissa, int).
3. Canali (delta di DELTA bilaterali, phpr−oracle per variante):
   **valore-oggetto** = Δ(m0)−Δ(m1) [Rc clone + drop del vecchio + gc_note]
   · **chiave/lookup** = Δ(m1)−Δ(m3) [KeyIndex/hash su chiave variabile]
   · **pavimento dim-set** = Δ(m3). Companion: Δ(m2)−Δ(m3) (oggetto a
   chiave monomorfa).
4. Metodo = s126-orm-micro p.2 (arbitro `s135-objmap-bisez.sh` = copia
   DICHIARATA di s126-orm-micro.sh collaudata con copia-gate, manifest
   committato): R=5 alternato, user CPU `/usr/bin/time -p`, pavimento
   per-binario su empty.php, N dal sorgente, parità stdout pena cifra NULLA.
   Finestra: lock misura GIÀ in campo, LSP giù, CI in attesa.
5. Lettura pre-registrata: la leva S-135 si NOMINA sul canale DOMINANTE
   (>50% del delta objmap) SE il meccanismo è nominabile dal sorgente
   (Serena, post-misura); la sua **UB falsificabile = il delta bilaterale
   del canale misurato QUI + spread** (az.rev. S-134 #2: niente componenti
   senza prezzo). Se nessun canale supera il 50% ⇒ istruttoria estesa a
   verbale, NESSUNA leva di ripiego. Nessuna predizione di magnitudine
   (prima decomposizione).
