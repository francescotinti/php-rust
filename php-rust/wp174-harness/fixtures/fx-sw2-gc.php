<?php
// fx-sw2-gc — S-175 az.rev. S-174 (a): gamba «pressione GC» del predicato sweep_idle
// (3ª clausola: `!gc_enabled || radici < gc_sweep_bound`, bound base 50 000 radici oggetto+container).
// Giudice: INVARIANZA pin == stash s173 (NON bilaterale: l'oracle raccoglie i cicli SOLO all'inserimento
// di una radice a buffer pieno, phpr allo Sweep di fine statement — divergenza a catalogo, ho_gc_status).
// Meccanismo: con GC disabilitato le radici (cicli a OGGETTI: `$o->self = $o`; un ciclo via `$a[] = &$a`
// NON lascia radici perché `$a = []` scrive attraverso il riferimento — sondato S-175) si accumulano senza
// raccolta; `gc_enable()` è una host-call SENZA Sweep proprio, quindi eseguita DENTRO il rhs di un op fuso
// in place su Long il PRIMO Sweep che vede «gc on ∧ radici ≥ bound» è quello che segue l'op fuso: sul pin
// gira e raccoglie (runs+1) prima della lettura di gc_status(); sotto MS (predicato sempre vero) o M3
// (3ª clausola forzata vera) viene scavalcato e la lettura vede il conteggio precedente ⇒ blocco ROTTO.
class N { public $self; }
function po(int $k): int { $s = 0; for ($i = 0; $i < $k; $i++) { $o = new N; $o->self = $o; $s += 1; } return $s; }

// gcp-fused: op fuso (BinarySTDst in place su Long) con gc_enable() INLINE nel rhs; lettura subito dopo
gc_disable(); po(60000);
$r0 = gc_status()['runs']; $s = 0; $s += (int)(gc_enable() === null); $r1 = gc_status()['runs'];
echo "gcp-fused: d=" . ($r1 - $r0) . " s=$s\n";
gc_collect_cycles();

// gcp-ctl: stessa forma NON fusa (`$s = $s + …` non è BinarySTDst: S-174) ⇒ lo Sweep è un op vero, gira sempre
gc_disable(); po(60000);
$r0 = gc_status()['runs']; $s = 0; $s = $s + (int)(gc_enable() === null); $r1 = gc_status()['runs'];
echo "gcp-ctl: d=" . ($r1 - $r0) . " s=$s\n";
gc_collect_cycles();

// gcp-loop: lettura DENTRO un loop con radici ≥ bound a ogni iterazione (revisione S-174, azione 3)
$d = []; $s = 0;
for ($i = 0; $i < 2; $i++) {
    gc_disable(); po(60000);
    $r0 = gc_status()['runs']; $s += (int)(gc_enable() === null); $d[] = gc_status()['runs'] - $r0;
}
echo "gcp-loop: " . implode(',', $d) . " s=$s\n";
gc_collect_cycles();

// gcp-scsc: BinarySCSCDst ha operandi slot/const puri ⇒ l'abilitazione sta in uno statement NON fuso subito
// prima: il PRIMO Sweep dopo «gc on» è quello dell'abilitazione (op vero) ⇒ raccolta lì; a verdetto (INTATTO atteso)
gc_disable(); po(60000); $a = 5; $b = 9; $l = 0;
$r0 = gc_status()['runs']; $z = (int)(gc_enable() === null); $l += $a * 3 - ($b >> 1); $r1 = gc_status()['runs'];
echo "gcp-scsc: d=" . ($r1 - $r0) . " l=$l\n";
gc_collect_cycles();

// gcp-under: radici SOTTO il bound con gc on: nessuna raccolta in nessun binario (3ª clausola vera per «radici < bound»)
gc_enable(); po(100);
$r0 = gc_status()['runs']; $s = 0; $s += (int)(gc_enable() === null); $r1 = gc_status()['runs'];
echo "gcp-under: d=" . ($r1 - $r0) . " s=$s\n";
gc_collect_cycles();

// gcp-off: gc DISABILITATO durante l'op fuso: 3ª clausola vera per `!gc_enabled` ⇒ Sweep inerte, nessuna raccolta
gc_disable(); po(60000);
$r0 = gc_status()['runs']; $s = 0; $s += 1; $r1 = gc_status()['runs'];
echo "gcp-off: d=" . ($r1 - $r0) . " s=$s\n";
gc_enable(); gc_collect_cycles();
echo "FX-SW2 DONE\n";
