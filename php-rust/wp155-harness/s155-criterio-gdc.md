# Criterio S-155 p.3 — conteggio get_declared_classes (PRE-REGISTRATO; CONTEGGI mai tempo, NEXT p.3: «conteggio CHIAMATE dovuto prima del criterio»)

1. **Domanda**: il census s154 dà get_declared_classes = 4.563.808 ALLOC
   nell'ORM; le CHIAMATE sono ignote. Ipotesi s152 da falsificare: l'array
   intero delle ~2.393 classi è ricostruito per chiamata (⇒ per-classe ≈ 1
   alloc, chiamate ≈ 4,56M/(fisso+2393)).
2. **Strumento**: `gdc-count.php` (driver NUOVO dichiarato, modello
   ce-count.php) col probe census s155 GIÀ COLLAUDATO dalla sonda p.2 (hash
   verificato contro il verdetto sonda); 4 run: N∈{100000, 300000} ×
   GDX∈{0, 200} (GDX = classi extra via eval, setup eliso da Δ/ΔN).
3. **Aritmetica meccanica**: k(GDX) = Δn/ΔN per name=get_declared_classes;
   per_classe = (k200−k0)/200 · fisso = k0 − per_classe×C0 (C0 = classi
   stampate dal driver) · k_ORM = fisso + per_classe×2393 · chiamate_ORM =
   4.563.808 / k_ORM.
4. **Attese PRE-REGISTRATE**: k0 e k200 INTERI ESATTI (Δ divisibile, pena
   dichiarazione); acc_ok=1 e classi coerenti (C200 = C0+200) pena STOP;
   ipotesi s152 CONFERMATA se per_classe ∈ [0,9; 1,1]; qualunque esito va a
   verbale coi numeri — il criterio di LEVA (se mai) è atto separato.
5. **Gate a rischio morso PRE-DICHIARATI (az.rev. S-154 #3)**: eval di 200
   classi nel probe (cliff evalcls noto: solo setup, non nel Δ) · probe
   mancante/hash difforme = STOP.
6. **Igiene**: lock presente, niente cargo/rustc in volo (parte SOLO a sonda
   p.2 conclusa), rc autoritativo da gdc-out/gdc.rc.
