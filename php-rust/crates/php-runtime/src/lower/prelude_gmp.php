<?php

/*
 * GMP — arbitrary-precision integers (ext/gmp).
 *
 * The `GMP` value object and the gmp_* functions live here (global namespace),
 * delegating to the `_gmp_*` builtins (num-bigint) in php-builtins/src/gmp.rs.
 * A GMP holds its value as a canonical base-10 string in the public `num`
 * property (what var_dump / get_object_vars show, matching ext/gmp).
 *
 * NOT modelled: gmp_random_* (non-deterministic). Operator overloading is wired
 * in the VM (like BcMath\Number). See PHPR_DIVERGENCES_FROM_PHP.md.
 */

class GMP
{
    public string $num;

    public function __construct(string|int $num = 0, int $base = 0)
    {
        if ($base !== 0 && ($base < 2 || $base > 62)) {
            throw new \ValueError('GMP::__construct(): Argument #2 ($base) must be 0 or between 2 and 62');
        }
        $this->num = self::_parse($num, $base, 'GMP::__construct', 1);
    }

    /** PHP type name for error messages ("resource", not "resource (stream)"). */
    private static function _typename(mixed $v): string
    {
        $t = \get_debug_type($v);
        return \str_starts_with($t, 'resource') ? 'resource' : $t;
    }

    /** Validate a GMP_ROUND_* rounding mode (argument #3 of the division fns). */
    public static function _round(int $mode, string $fn): void
    {
        if ($mode < 0 || $mode > 2) {
            throw new \ValueError(
                "$fn(): Argument #3 (\$rounding_mode) must be one of GMP_ROUND_ZERO, GMP_ROUND_PLUSINF, or GMP_ROUND_MINUSINF"
            );
        }
    }

    /** Validate an int-typed argument without the userland "called in" suffix. */
    public static function _int(mixed $v, string $fn, int $argnum, string $pname): int
    {
        if (\is_int($v)) {
            return $v;
        }
        if (\is_bool($v) || \is_float($v) || (\is_string($v) && \is_numeric($v))) {
            return (int) $v;
        }
        throw new \TypeError(
            "$fn(): Argument #$argnum (\$$pname) must be of type int, " . self::_typename($v) . ' given'
        );
    }

    public function __toString(): string
    {
        return $this->num;
    }

    public function __serialize(): array
    {
        return [\_gmp_strval($this->num, 16)];
    }

    public function __unserialize(array $data): void
    {
        $this->num = self::_parse($data[0] ?? '0', 16, 'GMP::__unserialize', 1);
    }

    /* ---- internal helpers (called by gmp_* wrappers and the VM) ---- */

    /** Parse a constructor/init operand (honours $base), or throw a branded ValueError. */
    public static function _parse(mixed $v, int $base, string $fn, int $argnum): string
    {
        if ($v instanceof GMP) {
            return $v->num;
        }
        if (\is_int($v)) {
            return (string) $v;
        }
        if (\is_string($v)) {
            $d = \_gmp_parse($v, $base);
            if ($d === false) {
                throw new \ValueError("$fn(): Argument #$argnum (\$num) is not an integer string");
            }
            return $d;
        }
        throw new \TypeError(
            "$fn(): Argument #$argnum (\$num) must be of type string|int, " . \get_debug_type($v) . ' given'
        );
    }

    /** Coerce a GMP|int|string operand to its canonical decimal (base auto-detect). */
    public static function _arg(mixed $v, string $fn, int $argnum, string $pname): string
    {
        if ($v instanceof GMP) {
            return $v->num;
        }
        if (\is_int($v)) {
            return (string) $v;
        }
        if (\is_float($v)) {
            return (string) (int) $v; // PHP deprecates; phpr omits the notice (§1.2)
        }
        if (\is_string($v)) {
            $d = \_gmp_parse($v, 0);
            if ($d === false) {
                throw new \ValueError("$fn(): Argument #$argnum (\$$pname) is not an integer string");
            }
            return $d;
        }
        throw new \TypeError(
            "$fn(): Argument #$argnum (\$$pname) must be of type GMP|string|int, " . self::_typename($v) . ' given'
        );
    }

    /** Wrap a canonical decimal string in a fresh GMP without re-parsing. */
    public static function _g(string $decimal): GMP
    {
        $o = new GMP();
        $o->num = $decimal;
        return $o;
    }

    /** Coerce an operator operand (do_operation semantics). */
    private static function _oparg(mixed $v): string
    {
        if ($v instanceof GMP) {
            return $v->num;
        }
        if (\is_int($v)) {
            return (string) $v;
        }
        if (\is_float($v)) {
            return (string) (int) $v; // PHP deprecates; phpr omits the notice (§1.2)
        }
        if (\is_string($v)) {
            $d = \_gmp_parse($v, 0);
            if ($d === false) {
                throw new \ValueError('Number is not an integer string');
            }
            return $d;
        }
        throw new \TypeError('Number must be of type GMP|string|int, ' . self::_typename($v) . ' given');
    }

    /**
     * do_operation for GMP, invoked by the VM's operator dispatch.
     * $op: 0 add 1 sub 2 mul 3 div 4 mod 5 pow 6 and 7 or 8 xor.
     */
    public function __op(int $op, mixed $a, mixed $b): GMP
    {
        $x = self::_oparg($a);
        $y = self::_oparg($b);
        switch ($op) {
            case 0:
                return self::_g(\_gmp_bin(0, $x, $y));
            case 1:
                return self::_g(\_gmp_bin(1, $x, $y));
            case 2:
                return self::_g(\_gmp_bin(2, $x, $y));
            case 3:
                if ($y === '0') {
                    throw new \DivisionByZeroError('Division by zero');
                }
                return self::_g(\_gmp_divq($x, $y, 0));
            case 4:
                if ($y === '0') {
                    throw new \DivisionByZeroError('Modulo by zero');
                }
                return self::_g(\_gmp_mod($x, $y));
            case 5:
                return self::_g(\_gmp_bin(12, $x, $y));
            case 6:
                return self::_g(\_gmp_bin(9, $x, $y));
            case 7:
                return self::_g(\_gmp_bin(10, $x, $y));
            case 8:
                return self::_g(\_gmp_bin(11, $x, $y));
            case 9: // shift left: x * 2^y
                return self::_g(\_gmp_bin(2, $x, \_gmp_bin(12, '2', $y)));
            default: // 10 = shift right: floor(x / 2^y)
                return self::_g(\_gmp_divq($x, \_gmp_bin(12, '2', $y), 2));
        }
    }

    /** do_compare for GMP. Returns -1/0/1; throws for invalid operands. */
    public function __cmp(mixed $a, mixed $b): int
    {
        return \_gmp_cmp(self::_oparg($a), self::_oparg($b));
    }
}

/* ---- conversion ---- */

function gmp_init(mixed $num, int $base = 0): GMP
{
    if ($base !== 0 && ($base < 2 || $base > 62)) {
        throw new \ValueError('gmp_init(): Argument #2 ($base) must be 0 or between 2 and 62');
    }
    return GMP::_g(GMP::_parse($num, $base, 'gmp_init', 1));
}

function gmp_strval(mixed $num, int $base = 10): string
{
    $a = GMP::_arg($num, 'gmp_strval', 1, 'num');
    if (($base < 2 || $base > 62) && ($base < -36 || $base > -2)) {
        throw new \ValueError('gmp_strval(): Argument #2 ($base) must be between 2 and 62, or -2 and -36');
    }
    return \_gmp_strval($a, $base);
}

function gmp_intval(mixed $num): int
{
    return \_gmp_intval(GMP::_arg($num, 'gmp_intval', 1, 'num'));
}

/* ---- arithmetic ---- */

function gmp_add(mixed $num1, mixed $num2): GMP
{
    return GMP::_g(\_gmp_bin(0, GMP::_arg($num1, 'gmp_add', 1, 'num1'), GMP::_arg($num2, 'gmp_add', 2, 'num2')));
}

function gmp_sub(mixed $num1, mixed $num2): GMP
{
    return GMP::_g(\_gmp_bin(1, GMP::_arg($num1, 'gmp_sub', 1, 'num1'), GMP::_arg($num2, 'gmp_sub', 2, 'num2')));
}

function gmp_mul(mixed $num1, mixed $num2): GMP
{
    return GMP::_g(\_gmp_bin(2, GMP::_arg($num1, 'gmp_mul', 1, 'num1'), GMP::_arg($num2, 'gmp_mul', 2, 'num2')));
}

function gmp_neg(mixed $num): GMP
{
    return GMP::_g(\_gmp_un(0, GMP::_arg($num, 'gmp_neg', 1, 'num')));
}

function gmp_abs(mixed $num): GMP
{
    return GMP::_g(\_gmp_un(1, GMP::_arg($num, 'gmp_abs', 1, 'num')));
}

/* ---- division ---- */

function gmp_div_q(mixed $num1, mixed $num2, int $rounding_mode = 0): GMP
{
    $a = GMP::_arg($num1, 'gmp_div_q', 1, 'num1');
    $b = GMP::_arg($num2, 'gmp_div_q', 2, 'num2');
    GMP::_round($rounding_mode, 'gmp_div_q');
    if ($b === '0') {
        throw new \DivisionByZeroError('gmp_div_q(): Argument #2 ($num2) Division by zero');
    }
    return GMP::_g(\_gmp_divq($a, $b, $rounding_mode));
}

function gmp_div_r(mixed $num1, mixed $num2, int $rounding_mode = 0): GMP
{
    $a = GMP::_arg($num1, 'gmp_div_r', 1, 'num1');
    $b = GMP::_arg($num2, 'gmp_div_r', 2, 'num2');
    GMP::_round($rounding_mode, 'gmp_div_r');
    if ($b === '0') {
        throw new \DivisionByZeroError('gmp_div_r(): Argument #2 ($num2) Division by zero');
    }
    return GMP::_g(\_gmp_divr($a, $b, $rounding_mode));
}

function gmp_div_qr(mixed $num1, mixed $num2, int $rounding_mode = 0): array
{
    $a = GMP::_arg($num1, 'gmp_div_qr', 1, 'num1');
    $b = GMP::_arg($num2, 'gmp_div_qr', 2, 'num2');
    GMP::_round($rounding_mode, 'gmp_div_qr');
    if ($b === '0') {
        throw new \DivisionByZeroError('gmp_div_qr(): Argument #2 ($num2) Division by zero');
    }
    $qr = \explode(' ', \_gmp_divqr($a, $b, $rounding_mode));
    return [GMP::_g($qr[0]), GMP::_g($qr[1])];
}

function gmp_div(mixed $num1, mixed $num2, int $rounding_mode = 0): GMP
{
    $a = GMP::_arg($num1, 'gmp_div', 1, 'num1');
    $b = GMP::_arg($num2, 'gmp_div', 2, 'num2');
    GMP::_round($rounding_mode, 'gmp_div');
    if ($b === '0') {
        throw new \DivisionByZeroError('gmp_div(): Argument #2 ($num2) Division by zero');
    }
    return GMP::_g(\_gmp_divq($a, $b, $rounding_mode));
}

function gmp_mod(mixed $num1, mixed $num2): GMP
{
    $a = GMP::_arg($num1, 'gmp_mod', 1, 'num1');
    $b = GMP::_arg($num2, 'gmp_mod', 2, 'num2');
    if ($b === '0') {
        throw new \DivisionByZeroError('gmp_mod(): Argument #2 ($num2) Modulo by zero');
    }
    return GMP::_g(\_gmp_mod($a, $b));
}

function gmp_divexact(mixed $num1, mixed $num2): GMP
{
    $a = GMP::_arg($num1, 'gmp_divexact', 1, 'num1');
    $b = GMP::_arg($num2, 'gmp_divexact', 2, 'num2');
    if ($b === '0') {
        throw new \DivisionByZeroError('gmp_divexact(): Argument #2 ($num2) Division by zero');
    }
    return GMP::_g(\_gmp_bin(6, $a, $b));
}

/* ---- powers / roots ---- */

function gmp_pow(mixed $num, mixed $exponent): GMP
{
    $a = GMP::_arg($num, 'gmp_pow', 1, 'num');
    $e = GMP::_int($exponent, 'gmp_pow', 2, 'exponent');
    if ($e < 0) {
        throw new \ValueError('gmp_pow(): Argument #2 ($exponent) must be greater than or equal to 0');
    }
    return GMP::_g(\_gmp_bin(12, $a, (string) $e));
}

function gmp_sqrt(mixed $num): GMP
{
    $a = GMP::_arg($num, 'gmp_sqrt', 1, 'num');
    if ($a[0] === '-') {
        throw new \ValueError('gmp_sqrt(): Argument #1 ($num) must be greater than or equal to 0');
    }
    return GMP::_g(\_gmp_un(3, $a));
}

/* ---- gcd / lcm ---- */

function gmp_gcd(mixed $num1, mixed $num2): GMP
{
    return GMP::_g(\_gmp_bin(7, GMP::_arg($num1, 'gmp_gcd', 1, 'num1'), GMP::_arg($num2, 'gmp_gcd', 2, 'num2')));
}

function gmp_lcm(mixed $num1, mixed $num2): GMP
{
    return GMP::_g(\_gmp_bin(8, GMP::_arg($num1, 'gmp_lcm', 1, 'num1'), GMP::_arg($num2, 'gmp_lcm', 2, 'num2')));
}

/* ---- comparison ---- */

function gmp_cmp(mixed $num1, mixed $num2): int
{
    return \_gmp_cmp(GMP::_arg($num1, 'gmp_cmp', 1, 'num1'), GMP::_arg($num2, 'gmp_cmp', 2, 'num2'));
}

function gmp_sign(mixed $num): int
{
    return \_gmp_sign(GMP::_arg($num, 'gmp_sign', 1, 'num'));
}

/* ---- number theory ---- */

function gmp_powm(mixed $num, mixed $exponent, mixed $modulus): GMP
{
    $e = GMP::_arg($exponent, 'gmp_powm', 2, 'exponent');
    if ($e[0] === '-') {
        throw new \ValueError('gmp_powm(): Argument #2 ($exponent) must be greater than or equal to 0');
    }
    $m = GMP::_arg($modulus, 'gmp_powm', 3, 'modulus');
    if ($m === '0') {
        throw new \DivisionByZeroError('Modulo by zero');
    }
    return GMP::_g(\_gmp_powm(GMP::_arg($num, 'gmp_powm', 1, 'num'), $e, $m));
}

function gmp_gcdext(mixed $num1, mixed $num2): array
{
    $r = \explode(' ', \_gmp_gcdext(GMP::_arg($num1, 'gmp_gcdext', 1, 'num1'), GMP::_arg($num2, 'gmp_gcdext', 2, 'num2')));
    return ['g' => GMP::_g($r[0]), 's' => GMP::_g($r[1]), 't' => GMP::_g($r[2])];
}

function gmp_invert(mixed $num1, mixed $num2): GMP|false
{
    $m = GMP::_arg($num2, 'gmp_invert', 2, 'num2');
    if ($m === '0') {
        throw new \DivisionByZeroError('Division by zero');
    }
    $r = \_gmp_invert(GMP::_arg($num1, 'gmp_invert', 1, 'num1'), $m);
    return $r === false ? false : GMP::_g($r);
}

function gmp_root(mixed $num, int $nth): GMP
{
    if ($nth < 1) {
        throw new \ValueError('gmp_root(): Argument #2 ($nth) must be greater than 0');
    }
    $a = GMP::_arg($num, 'gmp_root', 1, 'num');
    if ($a[0] === '-' && $nth % 2 === 0) {
        throw new \ValueError('gmp_root(): Argument #1 ($num) must be greater than or equal to 0 when argument #2 ($nth) is even');
    }
    return GMP::_g(\_gmp_root($a, $nth));
}

function gmp_rootrem(mixed $num, int $nth): array
{
    if ($nth < 1) {
        throw new \ValueError('gmp_rootrem(): Argument #2 ($nth) must be greater than 0');
    }
    $a = GMP::_arg($num, 'gmp_rootrem', 1, 'num');
    if ($a[0] === '-' && $nth % 2 === 0) {
        throw new \ValueError('gmp_rootrem(): Argument #1 ($num) must be greater than or equal to 0 when argument #2 ($nth) is even');
    }
    $r = \explode(' ', \_gmp_rootrem($a, $nth));
    return [GMP::_g($r[0]), GMP::_g($r[1])];
}

function gmp_sqrtrem(mixed $num): array
{
    $a = GMP::_arg($num, 'gmp_sqrtrem', 1, 'num');
    if ($a[0] === '-') {
        throw new \ValueError('gmp_sqrtrem(): Argument #1 ($num) must be greater than or equal to 0');
    }
    $r = \explode(' ', \_gmp_sqrtrem($a));
    return [GMP::_g($r[0]), GMP::_g($r[1])];
}

function gmp_fact(mixed $num): GMP
{
    if ($num instanceof GMP) {
        $num = $num->num;
    }
    $n = (int) $num;
    if ($n < 0) {
        throw new \ValueError('gmp_fact(): Argument #1 ($num) must be greater than or equal to 0');
    }
    return GMP::_g(\_gmp_fact($n));
}

function gmp_binomial(mixed $n, int $k): GMP
{
    if ($k < 0) {
        throw new \ValueError('gmp_binomial(): Argument #2 ($k) must be greater than or equal to 0');
    }
    return GMP::_g(\_gmp_binomial(GMP::_arg($n, 'gmp_binomial', 1, 'n'), $k));
}

function gmp_prob_prime(mixed $num, int $repetitions = 10): int
{
    return \_gmp_probprime(GMP::_arg($num, 'gmp_prob_prime', 1, 'num'), $repetitions);
}

function gmp_nextprime(mixed $num): GMP
{
    return GMP::_g(\_gmp_nextprime(GMP::_arg($num, 'gmp_nextprime', 1, 'num')));
}

function gmp_kronecker(mixed $num1, mixed $num2): int
{
    return \_gmp_kronecker(GMP::_arg($num1, 'gmp_kronecker', 1, 'num1'), GMP::_arg($num2, 'gmp_kronecker', 2, 'num2'));
}

function gmp_jacobi(mixed $num1, mixed $num2): int
{
    return \_gmp_kronecker(GMP::_arg($num1, 'gmp_jacobi', 1, 'num1'), GMP::_arg($num2, 'gmp_jacobi', 2, 'num2'));
}

function gmp_legendre(mixed $num1, mixed $num2): int
{
    return \_gmp_kronecker(GMP::_arg($num1, 'gmp_legendre', 1, 'num1'), GMP::_arg($num2, 'gmp_legendre', 2, 'num2'));
}

function gmp_perfect_square(mixed $num): bool
{
    return \_gmp_perfsquare(GMP::_arg($num, 'gmp_perfect_square', 1, 'num'));
}

function gmp_perfect_power(mixed $num): bool
{
    return \_gmp_perfpower(GMP::_arg($num, 'gmp_perfect_power', 1, 'num'));
}

/* ---- bitwise ---- */

function gmp_and(mixed $num1, mixed $num2): GMP
{
    return GMP::_g(\_gmp_bin(9, GMP::_arg($num1, 'gmp_and', 1, 'num1'), GMP::_arg($num2, 'gmp_and', 2, 'num2')));
}

function gmp_or(mixed $num1, mixed $num2): GMP
{
    return GMP::_g(\_gmp_bin(10, GMP::_arg($num1, 'gmp_or', 1, 'num1'), GMP::_arg($num2, 'gmp_or', 2, 'num2')));
}

function gmp_xor(mixed $num1, mixed $num2): GMP
{
    return GMP::_g(\_gmp_bin(11, GMP::_arg($num1, 'gmp_xor', 1, 'num1'), GMP::_arg($num2, 'gmp_xor', 2, 'num2')));
}

function gmp_com(mixed $num): GMP
{
    return GMP::_g(\_gmp_un(2, GMP::_arg($num, 'gmp_com', 1, 'num')));
}

function gmp_setbit(GMP $num, int $index, bool $value = true): void
{
    if ($index < 0) {
        throw new \ValueError('gmp_setbit(): Argument #2 ($index) must be greater than or equal to 0');
    }
    $num->num = \_gmp_setbit($num->num, $index, $value ? 1 : 0);
}

function gmp_clrbit(GMP $num, int $index): void
{
    if ($index < 0) {
        throw new \ValueError('gmp_clrbit(): Argument #2 ($index) must be greater than or equal to 0');
    }
    $num->num = \_gmp_setbit($num->num, $index, 0);
}

function gmp_testbit(mixed $num, int $index): bool
{
    if ($index < 0) {
        throw new \ValueError('gmp_testbit(): Argument #2 ($index) must be greater than or equal to 0');
    }
    return \_gmp_testbit(GMP::_arg($num, 'gmp_testbit', 1, 'num'), $index);
}

function gmp_scan0(mixed $num1, int $start): int
{
    if ($start < 0) {
        throw new \ValueError('gmp_scan0(): Argument #2 ($start) must be greater than or equal to 0');
    }
    return \_gmp_scan0(GMP::_arg($num1, 'gmp_scan0', 1, 'num1'), $start);
}

function gmp_scan1(mixed $num1, int $start): int
{
    if ($start < 0) {
        throw new \ValueError('gmp_scan1(): Argument #2 ($start) must be greater than or equal to 0');
    }
    return \_gmp_scan1(GMP::_arg($num1, 'gmp_scan1', 1, 'num1'), $start);
}

function gmp_popcount(mixed $num): int
{
    return \_gmp_popcount(GMP::_arg($num, 'gmp_popcount', 1, 'num'));
}

function gmp_hamdist(mixed $num1, mixed $num2): int
{
    return \_gmp_hamdist(GMP::_arg($num1, 'gmp_hamdist', 1, 'num1'), GMP::_arg($num2, 'gmp_hamdist', 2, 'num2'));
}
