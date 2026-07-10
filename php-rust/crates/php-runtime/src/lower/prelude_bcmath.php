<?php

/*
 * BcMath\Number — the PHP 8.4+ arbitrary-precision decimal value object.
 *
 * Implemented in the prelude as a real PHP class delegating to the bc*
 * builtins (which are byte-identical ports of libbcmath). The scale-defaulting
 * rules mirror ext/bcmath/bcmath.c's bcmath_number_*_internal helpers:
 *   add/sub -> max(a.scale, b.scale)      mul -> a.scale + b.scale
 *   div/sqrt/pow(neg) -> a.scale + 10, then collapse trailing zeros
 *   pow(>0) -> a.scale * exponent         mod -> max(a.scale, b.scale)
 *   powmod  -> given (default 0)          floor/ceil -> 0
 * `value` is the number rendered at `scale`; `scale` equals the number of
 * fractional digits kept in `value`.
 *
 * NOT modelled: operator overloading (+, -, *, /, %, **, <=>, ==, ++/--) —
 * PHP's do_operation/compare_object handlers are an engine feature phpr does
 * not expose. See PHPR_DIVERGENCES_FROM_PHP.md §2.1.
 */

namespace BcMath;

final class Number implements \Stringable
{
    public readonly string $value;
    public readonly int $scale;

    public function __construct(string|int $num)
    {
        $s = \is_int($num) ? (string) $num : $num;
        if (!self::wellFormed($s)) {
            throw new \ValueError('BcMath\\Number::__construct(): Argument #1 ($num) is not well-formed');
        }
        $fs = self::fracLen($s);
        $this->value = \bcadd($s, '0', $fs);
        $this->scale = $fs;
    }

    /* ---- validation / scale helpers ---- */

    private static function wellFormed(string $s): bool
    {
        // [+-]? digits* (. digits*)?  — matches libbcmath's bc_str2num grammar.
        return (bool) \preg_match('/^[+-]?[0-9]*(\.[0-9]*)?$/', $s);
    }

    /** Number of fractional digits as written (no trailing-zero trimming). */
    private static function fracLen(string $s): int
    {
        $dot = \strpos($s, '.');
        return $dot === false ? 0 : \strlen($s) - $dot - 1;
    }

    /** Fractional length after trimming trailing zeros (the bc_num n_scale). */
    private static function trimScale(string $s): int
    {
        $dot = \strpos($s, '.');
        if ($dot === false) {
            return 0;
        }
        return \strlen(\rtrim(\substr($s, $dot + 1), '0'));
    }

    private static function isIntegerValue(string $s): bool
    {
        return self::trimScale($s) === 0;
    }

    private static function checkScale(?int $scale, int $argNum, string $method): void
    {
        if ($scale !== null && ($scale < 0 || $scale > 2147483647)) {
            throw new \ValueError(
                "BcMath\\Number::$method(): Argument #$argNum (\$scale) must be between 0 and 2147483647"
            );
        }
    }

    /** Coerce an argument to [numericString, fullScale]. */
    private static function coerce(mixed $v, string $method, int $argNum, string $pname): array
    {
        if ($v instanceof Number) {
            return [$v->value, $v->scale];
        }
        if (\is_int($v)) {
            return [(string) $v, 0];
        }
        if ($v === null) {
            // PHP deprecates passing null here, then treats it as 0. phpr does
            // not emit that ZPP deprecation (see PHPR_DIVERGENCES §1.2).
            return ['0', 0];
        }
        if (\is_string($v)) {
            if (!self::wellFormed($v)) {
                throw new \ValueError(
                    "BcMath\\Number::$method(): Argument #$argNum (\$$pname) is not well-formed"
                );
            }
            return [$v, self::fracLen($v)];
        }
        throw new \TypeError(
            "BcMath\\Number::$method(): Argument #$argNum (\$$pname) must be of type int, string, or BcMath\\Number, "
            . \get_debug_type($v) . ' given'
        );
    }

    private static function make(string $value): Number
    {
        return new Number($value);
    }

    /** div/sqrt/pow(neg) auto-scale: compute at fs+10, then collapse. */
    private static function collapse(string $full, int $fullScale): string
    {
        $trimmed = \rtrim(\rtrim($full, '0'), '.');
        $actual = self::fracLen($trimmed);
        $diff = $fullScale - $actual;
        $reduced = $fullScale - \min($diff, 10);
        return \bcadd($trimmed, '0', $reduced);
    }

    /* ---- arithmetic methods ---- */

    public function add(mixed $num, ?int $scale = null): Number
    {
        [$bstr, $bfs] = self::coerce($num, 'add', 1, 'num');
        self::checkScale($scale, 2, 'add');
        $sc = $scale ?? \max($this->scale, $bfs);
        return self::make(\bcadd($this->value, $bstr, $sc));
    }

    public function sub(mixed $num, ?int $scale = null): Number
    {
        [$bstr, $bfs] = self::coerce($num, 'sub', 1, 'num');
        self::checkScale($scale, 2, 'sub');
        $sc = $scale ?? \max($this->scale, $bfs);
        return self::make(\bcsub($this->value, $bstr, $sc));
    }

    public function mul(mixed $num, ?int $scale = null): Number
    {
        [$bstr, $bfs] = self::coerce($num, 'mul', 1, 'num');
        self::checkScale($scale, 2, 'mul');
        $sc = $scale ?? $this->scale + $bfs;
        return self::make(\bcmul($this->value, $bstr, $sc));
    }

    public function div(mixed $num, ?int $scale = null): Number
    {
        [$bstr] = self::coerce($num, 'div', 1, 'num');
        self::checkScale($scale, 2, 'div');
        if ($scale !== null) {
            return self::make(\bcdiv($this->value, $bstr, $scale));
        }
        $full = $this->scale + 10;
        return self::make(self::collapse(\bcdiv($this->value, $bstr, $full), $full));
    }

    public function mod(mixed $num, ?int $scale = null): Number
    {
        [$bstr, $bfs] = self::coerce($num, 'mod', 1, 'num');
        self::checkScale($scale, 2, 'mod');
        $sc = $scale ?? \max($this->scale, $bfs);
        return self::make(\bcmod($this->value, $bstr, $sc));
    }

    public function divmod(mixed $num, ?int $scale = null): array
    {
        [$bstr, $bfs] = self::coerce($num, 'divmod', 1, 'num');
        self::checkScale($scale, 2, 'divmod');
        $sc = $scale ?? \max($this->scale, $bfs);
        [$q, $r] = \bcdivmod($this->value, $bstr, $sc);
        return [self::make($q), self::make($r)];
    }

    public function pow(mixed $exponent, ?int $scale = null): Number
    {
        [$estr] = self::coerce($exponent, 'pow', 1, 'exponent');
        self::checkScale($scale, 2, 'pow');
        if (!self::isIntegerValue($estr)) {
            throw new \ValueError('BcMath\\Number::pow(): Argument #1 ($exponent) exponent cannot have a fractional part');
        }
        $exp = \bcadd($estr, '0', 0); // integer-normalized exponent string
        if ($scale !== null) {
            return self::make(\bcpow($this->value, $exp, $scale));
        }
        if ($exp === '0') {
            return self::make(\bcpow($this->value, '0', 0));
        }
        if ($exp[0] !== '-') {
            $sc = $this->scale * (int) $exp;
            if ($sc > 2147483647) {
                throw new \ValueError('BcMath\\Number::pow(): scale of the result is too large');
            }
            return self::make(\bcpow($this->value, $exp, $sc));
        }
        $full = $this->scale + 10;
        return self::make(self::collapse(\bcpow($this->value, $exp, $full), $full));
    }

    public function powmod(mixed $exponent, mixed $modulus, ?int $scale = null): Number
    {
        [$estr] = self::coerce($exponent, 'powmod', 1, 'exponent');
        [$mstr] = self::coerce($modulus, 'powmod', 2, 'modulus');
        self::checkScale($scale, 3, 'powmod');
        if (!self::isIntegerValue($this->value)) {
            // zend_value_error: no method-name prefix.
            throw new \ValueError('Base number cannot have a fractional part');
        }
        if (!self::isIntegerValue($estr)) {
            throw new \ValueError('BcMath\\Number::powmod(): Argument #1 ($exponent) cannot have a fractional part');
        }
        if ($estr[0] === '-') {
            throw new \ValueError('BcMath\\Number::powmod(): Argument #1 ($exponent) must be greater than or equal to 0');
        }
        if (!self::isIntegerValue($mstr)) {
            throw new \ValueError('BcMath\\Number::powmod(): Argument #2 ($modulus) cannot have a fractional part');
        }
        if (\bccomp($mstr, '0', self::trimScale($mstr)) === 0) {
            throw new \DivisionByZeroError('Modulo by zero');
        }
        $sc = $scale ?? 0;
        return self::make(\bcpowmod($this->value, $estr, $mstr, $sc));
    }

    public function sqrt(?int $scale = null): Number
    {
        self::checkScale($scale, 1, 'sqrt');
        if ($this->value[0] === '-' && !self::isZero($this->value)) {
            // zend_value_error: no method-name prefix.
            throw new \ValueError('Base number must be greater than or equal to 0');
        }
        if ($scale !== null) {
            return self::make(\bcsqrt($this->value, $scale));
        }
        $full = $this->scale + 10;
        return self::make(self::collapse(\bcsqrt($this->value, $full), $full));
    }

    public function floor(): Number
    {
        return self::make(\bcfloor($this->value));
    }

    public function ceil(): Number
    {
        return self::make(\bcceil($this->value));
    }

    public function round(int $precision = 0, ?\RoundingMode $mode = null): Number
    {
        return self::make(\bcround($this->value, $precision, $mode));
    }

    public function compare(mixed $num, ?int $scale = null): int
    {
        [$bstr] = self::coerce($num, 'compare', 1, 'num');
        self::checkScale($scale, 2, 'compare');
        $sc = $scale ?? \max(self::trimScale($this->value), self::trimScale($bstr));
        return \bccomp($this->value, $bstr, $sc);
    }

    private static function isZero(string $s): bool
    {
        return \bccomp($s, '0', self::trimScale($s)) === 0;
    }

    /* ---- conversion / serialization ---- */

    public function __toString(): string
    {
        return $this->value;
    }

    public function __serialize(): array
    {
        return ['value' => $this->value];
    }

    public function __unserialize(array $data): void
    {
        $v = $data['value'] ?? null;
        if (!\is_string($v) || $v === '' || !self::wellFormed($v)) {
            throw new \Exception('Invalid serialization data for BcMath\\Number object');
        }
        $fs = self::fracLen($v);
        $this->value = \bcadd($v, '0', $fs);
        $this->scale = $fs;
    }
}
