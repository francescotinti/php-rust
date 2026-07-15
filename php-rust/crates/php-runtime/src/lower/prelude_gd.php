<?php
// ext/gd: the GdImage opaque handle class and the image* procedural API,
// delegating to the __gd_* host builtins (system libgd FFI) in vm/gd.rs.
// GdImage is engine-opaque (vm: is_opaque_handle_class): var_dump/var_export/
// json show no props, clone/serialize throw, reflection reports no members —
// the hidden $__h prop and the helper methods below stay invisible.
// I/O deliberately happens PHP-side (file_get_contents / file_put_contents /
// echo of the encoded bytes), so stream wrappers and output buffering apply
// exactly as in ext/gd's php_stream path.

final class GdImage
{
    public $__h = 0;

    public function __construct(...$a)
    {
        if (($a[0] ?? null) !== "\0gd\0") {
            throw new Error('You cannot initialize a GdImage object except through helper functions');
        }
    }

    public static function __wrap(int $h): GdImage
    {
        $o = new GdImage("\0gd\0");
        $o->__h = $h;
        return $o;
    }

    public function __destruct()
    {
        if ($this->__h) {
            __gd_destroy($this->__h);
            $this->__h = 0;
        }
    }
}

// ---- creation ---------------------------------------------------------

function imagecreate(int $width, int $height)
{
    if ($width < 1) {
        throw new ValueError('imagecreate(): Argument #1 ($width) must be greater than 0');
    }
    if ($height < 1) {
        throw new ValueError('imagecreate(): Argument #2 ($height) must be greater than 0');
    }
    $h = __gd_create($width, $height, false);
    return $h === false ? false : GdImage::__wrap($h);
}

function imagecreatetruecolor(int $width, int $height)
{
    if ($width < 1) {
        throw new ValueError('imagecreatetruecolor(): Argument #1 ($width) must be greater than 0');
    }
    if ($height < 1) {
        throw new ValueError('imagecreatetruecolor(): Argument #2 ($height) must be greater than 0');
    }
    $h = __gd_create($width, $height, true);
    return $h === false ? false : GdImage::__wrap($h);
}

function imagedestroy(GdImage $image): bool
{
    __deprecated_from_caller('Function imagedestroy() is deprecated since 8.5, as it has no effect since PHP 8.0');
    return true;
}

// Shared reader for imagecreatefrom*(path): the "Failed to open stream"
// Warning carries the *outer* function's name (docref style).
function __gd_read_for(string $fn, string $filename)
{
    $d = @file_get_contents($filename);
    if ($d === false) {
        $e = error_get_last();
        $reason = 'No such file or directory';
        if ($e !== null && ($p = strpos($e['message'], 'Failed to open stream: ')) !== false) {
            $reason = substr($e['message'], $p + strlen('Failed to open stream: '));
        }
        __warning_from_caller($fn . '(' . $filename . '): Failed to open stream: ' . $reason);
        return false;
    }
    return $d;
}

// Shared decode for imagecreatefrom*(path): libgd messages become Warnings,
// then ext/gd's own '"%s" is not a valid %s file'.
function __gd_load(string $fn, string $filename, string $kind, string $tn)
{
    $d = __gd_read_for($fn, $filename);
    if ($d === false) {
        return false;
    }
    $r = __gd_decode($kind, $d);
    if (isset($r['h'])) {
        return GdImage::__wrap($r['h']);
    }
    foreach ($r['errs'] ?? array() as $e) {
        __warning_from_caller($fn . '(): ' . $e);
    }
    __warning_from_caller($fn . '(): "' . $filename . '" is not a valid ' . $tn . ' file');
    return false;
}

function imagecreatefromjpeg(string $filename)
{
    return __gd_load('imagecreatefromjpeg', $filename, 'jpeg', 'JPEG');
}
function imagecreatefrompng(string $filename)
{
    return __gd_load('imagecreatefrompng', $filename, 'png', 'PNG');
}
function imagecreatefromgif(string $filename)
{
    return __gd_load('imagecreatefromgif', $filename, 'gif', 'GIF');
}
function imagecreatefromwebp(string $filename)
{
    return __gd_load('imagecreatefromwebp', $filename, 'webp', 'WEBP');
}
function imagecreatefromavif(string $filename)
{
    return __gd_load('imagecreatefromavif', $filename, 'avif', 'AVIF');
}
function imagecreatefrombmp(string $filename)
{
    return __gd_load('imagecreatefrombmp', $filename, 'bmp', 'BMP');
}
function imagecreatefromwbmp(string $filename)
{
    return __gd_load('imagecreatefromwbmp', $filename, 'wbmp', 'WBMP');
}
function imagecreatefromtga(string $filename)
{
    return __gd_load('imagecreatefromtga', $filename, 'tga', 'TGA');
}

function imagecreatefromstring(string $data)
{
    $r = __gd_decode_auto($data);
    if (isset($r['h'])) {
        return GdImage::__wrap($r['h']);
    }
    if (isset($r['unknown'])) {
        __warning_from_caller('imagecreatefromstring(): Data is not in a recognized format');
        return false;
    }
    foreach ($r['errs'] ?? array() as $e) {
        __warning_from_caller('imagecreatefromstring(): ' . $e);
    }
    __warning_from_caller("imagecreatefromstring(): Couldn't create GD Image Stream out of Data");
    return false;
}

// ---- geometry / state --------------------------------------------------

function imagesx(GdImage $image): int
{
    $s = __gd_stat($image->__h);
    return $s['sx'];
}
function imagesy(GdImage $image): int
{
    $s = __gd_stat($image->__h);
    return $s['sy'];
}
function imageistruecolor(GdImage $image): bool
{
    $s = __gd_stat($image->__h);
    return $s['tc'];
}
function imagecolorstotal(GdImage $image): int
{
    $s = __gd_stat($image->__h);
    return $s['colors'];
}
function imagecolortransparent(GdImage $image, ?int $color = null): int
{
    return __gd_colortransparent($image->__h, $color === null ? -2 : $color);
}
function imagealphablending(GdImage $image, bool $enable): bool
{
    return __gd_flag($image->__h, 'blend', $enable);
}
function imagesavealpha(GdImage $image, bool $enable): bool
{
    return __gd_flag($image->__h, 'savealpha', $enable);
}
function imageinterlace(GdImage $image, ?bool $enable = null): bool
{
    if ($enable !== null) {
        __gd_flag($image->__h, 'interlace', $enable);
    }
    $s = __gd_stat($image->__h);
    return $s['interlace'];
}
function imageantialias(GdImage $image, bool $enable): bool
{
    return __gd_flag($image->__h, 'aa', $enable);
}
function imagesetinterpolation(GdImage $image, int $method = IMG_BILINEAR_FIXED): bool
{
    return __gd_setinterpolation($image->__h, $method);
}

// ---- colors ------------------------------------------------------------

function __gd_check_rgb(string $fn, int $red, int $green, int $blue, ?int $alpha = null): void
{
    if ($red < 0 || $red > 255) {
        throw new ValueError($fn . '(): Argument #2 ($red) must be between 0 and 255 (inclusive)');
    }
    if ($green < 0 || $green > 255) {
        throw new ValueError($fn . '(): Argument #3 ($green) must be between 0 and 255 (inclusive)');
    }
    if ($blue < 0 || $blue > 255) {
        throw new ValueError($fn . '(): Argument #4 ($blue) must be between 0 and 255 (inclusive)');
    }
    if ($alpha !== null && ($alpha < 0 || $alpha > 127)) {
        throw new ValueError($fn . '(): Argument #5 ($alpha) must be between 0 and 127 (inclusive)');
    }
}

function imagecolorallocate(GdImage $image, int $red, int $green, int $blue)
{
    __gd_check_rgb('imagecolorallocate', $red, $green, $blue);
    return __gd_color($image->__h, 'allocate', $red, $green, $blue, -1);
}
function imagecolorallocatealpha(GdImage $image, int $red, int $green, int $blue, int $alpha)
{
    __gd_check_rgb('imagecolorallocatealpha', $red, $green, $blue, $alpha);
    return __gd_color($image->__h, 'allocate', $red, $green, $blue, $alpha);
}
function imagecolorclosest(GdImage $image, int $red, int $green, int $blue): int
{
    __gd_check_rgb('imagecolorclosest', $red, $green, $blue);
    return __gd_color($image->__h, 'closest', $red, $green, $blue, -1);
}
function imagecolorclosestalpha(GdImage $image, int $red, int $green, int $blue, int $alpha): int
{
    __gd_check_rgb('imagecolorclosestalpha', $red, $green, $blue, $alpha);
    return __gd_color($image->__h, 'closest', $red, $green, $blue, $alpha);
}
function imagecolorexact(GdImage $image, int $red, int $green, int $blue): int
{
    __gd_check_rgb('imagecolorexact', $red, $green, $blue);
    return __gd_color($image->__h, 'exact', $red, $green, $blue, -1);
}
function imagecolorexactalpha(GdImage $image, int $red, int $green, int $blue, int $alpha): int
{
    __gd_check_rgb('imagecolorexactalpha', $red, $green, $blue, $alpha);
    return __gd_color($image->__h, 'exact', $red, $green, $blue, $alpha);
}
function imagecolorresolve(GdImage $image, int $red, int $green, int $blue): int
{
    __gd_check_rgb('imagecolorresolve', $red, $green, $blue);
    return __gd_color($image->__h, 'resolve', $red, $green, $blue, -1);
}
function imagecolorresolvealpha(GdImage $image, int $red, int $green, int $blue, int $alpha): int
{
    __gd_check_rgb('imagecolorresolvealpha', $red, $green, $blue, $alpha);
    return __gd_color($image->__h, 'resolve', $red, $green, $blue, $alpha);
}
function imagecolorsforindex(GdImage $image, int $color): array
{
    $r = __gd_colorsforindex($image->__h, $color);
    if ($r === false) {
        throw new ValueError('imagecolorsforindex(): Argument #2 ($color) is out of range');
    }
    return $r;
}
function imagecolorat(GdImage $image, int $x, int $y)
{
    $r = __gd_colorat($image->__h, $x, $y);
    if ($r === false) {
        __notice_from_caller('imagecolorat(): ' . $x . ',' . $y . ' is out of bounds');
        return false;
    }
    return $r;
}

// ---- pixels / drawing ---------------------------------------------------

function imagesetpixel(GdImage $image, int $x, int $y, int $color): bool
{
    return __gd_setpixel($image->__h, $x, $y, $color);
}
function imageline(GdImage $image, int $x1, int $y1, int $x2, int $y2, int $color): bool
{
    return __gd_draw($image->__h, 'line', $x1, $y1, $x2, $y2, $color);
}
function imagerectangle(GdImage $image, int $x1, int $y1, int $x2, int $y2, int $color): bool
{
    return __gd_draw($image->__h, 'rect', $x1, $y1, $x2, $y2, $color);
}
function imagefilledrectangle(GdImage $image, int $x1, int $y1, int $x2, int $y2, int $color): bool
{
    return __gd_draw($image->__h, 'filledrect', $x1, $y1, $x2, $y2, $color);
}
function imageellipse(GdImage $image, int $center_x, int $center_y, int $width, int $height, int $color): bool
{
    return __gd_draw($image->__h, 'ellipse', $center_x, $center_y, $width, $height, $color);
}
function imagefilledellipse(GdImage $image, int $center_x, int $center_y, int $width, int $height, int $color): bool
{
    return __gd_draw($image->__h, 'filledellipse', $center_x, $center_y, $width, $height, $color);
}
function imagefill(GdImage $image, int $x, int $y, int $color): bool
{
    return __gd_draw($image->__h, 'fill', $x, $y, $color, 0, 0);
}
function imagefilltoborder(GdImage $image, int $x, int $y, int $border_color, int $color): bool
{
    return __gd_draw($image->__h, 'filltoborder', $x, $y, $border_color, $color, 0);
}

// ---- copy / transform ----------------------------------------------------

function imagecopy(GdImage $dst_image, GdImage $src_image, int $dst_x, int $dst_y, int $src_x, int $src_y, int $src_width, int $src_height): bool
{
    return __gd_copy($dst_image->__h, $src_image->__h, 'copy', $dst_x, $dst_y, $src_x, $src_y, $src_width, $src_height, 0, 0);
}
function imagecopyresampled(GdImage $dst_image, GdImage $src_image, int $dst_x, int $dst_y, int $src_x, int $src_y, int $dst_width, int $dst_height, int $src_width, int $src_height): bool
{
    return __gd_copy($dst_image->__h, $src_image->__h, 'resampled', $dst_x, $dst_y, $src_x, $src_y, $dst_width, $dst_height, $src_width, $src_height);
}
function imagecopyresized(GdImage $dst_image, GdImage $src_image, int $dst_x, int $dst_y, int $src_x, int $src_y, int $dst_width, int $dst_height, int $src_width, int $src_height): bool
{
    return __gd_copy($dst_image->__h, $src_image->__h, 'resized', $dst_x, $dst_y, $src_x, $src_y, $dst_width, $dst_height, $src_width, $src_height);
}
function imagerotate(GdImage $image, float $angle, int $background_color)
{
    $h = __gd_rotate($image->__h, $angle, $background_color);
    return $h === false ? false : GdImage::__wrap($h);
}
function imageflip(GdImage $image, int $mode): bool
{
    if ($mode < IMG_FLIP_HORIZONTAL || $mode > IMG_FLIP_BOTH) {
        throw new ValueError('imageflip(): Argument #2 ($mode) must be one of IMG_FLIP_VERTICAL, IMG_FLIP_HORIZONTAL, or IMG_FLIP_BOTH');
    }
    return __gd_flip($image->__h, $mode);
}
function imagecrop(GdImage $image, array $rectangle)
{
    foreach (array('x', 'y', 'width', 'height') as $k) {
        if (!isset($rectangle[$k])) {
            throw new ValueError('imagecrop(): Argument #2 ($rectangle) must have keys "x", "y", "width" and "height"');
        }
    }
    $h = __gd_crop($image->__h, (int)$rectangle['x'], (int)$rectangle['y'], (int)$rectangle['width'], (int)$rectangle['height']);
    return $h === false ? false : GdImage::__wrap($h);
}
function imagescale(GdImage $image, int $width, int $height = -1, int $mode = IMG_BILINEAR_FIXED)
{
    if ($height < 0) {
        $s = __gd_stat($image->__h);
        $height = (int)round($width * $s['sy'] / $s['sx']);
    }
    $h = __gd_scale($image->__h, $width, $height, $mode);
    return $h === false ? false : GdImage::__wrap($h);
}
function imagetruecolortopalette(GdImage $image, bool $dither, int $num_colors): bool
{
    if ($num_colors < 1) {
        throw new ValueError('imagetruecolortopalette(): Argument #3 ($num_colors) must be greater than 0 and less than 2147483648');
    }
    return __gd_t2p($image->__h, $dither, $num_colors);
}
function imagepalettetotruecolor(GdImage $image): bool
{
    return __gd_p2t($image->__h);
}

// ---- bitmap-font text -----------------------------------------------------

function imagestring(GdImage $image, int $font, int $x, int $y, string $string, int $color): bool
{
    return __gd_string($image->__h, $font, $x, $y, $string, $color, false);
}
function imagestringup(GdImage $image, int $font, int $x, int $y, string $string, int $color): bool
{
    return __gd_string($image->__h, $font, $x, $y, $string, $color, true);
}
function imagechar(GdImage $image, int $font, int $x, int $y, string $char, int $color): bool
{
    return __gd_char($image->__h, $font, $x, $y, $char, $color, false);
}
function imagecharup(GdImage $image, int $font, int $x, int $y, string $char, int $color): bool
{
    return __gd_char($image->__h, $font, $x, $y, $char, $color, true);
}
function imagefontwidth(int $font): int
{
    $s = __gd_fontsize($font);
    return $s[0];
}
function imagefontheight(int $font): int
{
    $s = __gd_fontsize($font);
    return $s[1];
}

// ---- output ---------------------------------------------------------------

// Encode + route the bytes: null → the output buffer; resource → fwrite;
// string path → file_put_contents (streams, so file:// & co. work). The
// "Failed to open stream" Warning carries the outer function's name.
function __gd_output(GdImage $image, string $kind, string $fn, $file, int $q1, int $q2): bool
{
    $r = __gd_encode($image->__h, $kind, $q1, $q2);
    if (!is_string($r)) {
        if (is_array($r)) {
            foreach ($r['errs'] ?? array() as $e) {
                __warning_from_caller($fn . '(): ' . $e);
            }
        }
        return false;
    }
    if ($file === null || $file === false) {
        echo $r;
        return true;
    }
    if (is_resource($file)) {
        return @fwrite($file, $r) !== false;
    }
    $ok = @file_put_contents((string)$file, $r);
    if ($ok === false) {
        $e = error_get_last();
        $reason = 'No such file or directory';
        if ($e !== null && ($p = strpos($e['message'], 'Failed to open stream: ')) !== false) {
            $reason = substr($e['message'], $p + strlen('Failed to open stream: '));
        }
        __warning_from_caller($fn . '(' . $file . '): Failed to open stream: ' . $reason);
        return false;
    }
    return true;
}

function imagejpeg(GdImage $image, $file = null, int $quality = -1): bool
{
    if ($quality < -1 || $quality > 100) {
        throw new ValueError('imagejpeg(): Argument #3 ($quality) must be at between -1 and 100');
    }
    return __gd_output($image, 'jpeg', 'imagejpeg', $file, $quality, 0);
}
function imagepng(GdImage $image, $file = null, int $quality = -1, int $filters = -1): bool
{
    if ($quality < -1 || $quality > 9) {
        throw new ValueError('imagepng(): Argument #3 ($quality) must be at between -1 and 9');
    }
    return __gd_output($image, 'png', 'imagepng', $file, $quality, 0);
}
function imagegif(GdImage $image, $file = null): bool
{
    return __gd_output($image, 'gif', 'imagegif', $file, 0, 0);
}
function imagewebp(GdImage $image, $file = null, int $quality = -1): bool
{
    if ($quality < -1 || $quality > 101) {
        throw new ValueError('imagewebp(): Argument #3 ($quality) must be at between -1 and 101 (IMG_WEBP_LOSSLESS)');
    }
    return __gd_output($image, 'webp', 'imagewebp', $file, $quality, 0);
}
function imageavif(GdImage $image, $file = null, int $quality = -1, int $speed = -1): bool
{
    if ($quality < -1 || $quality > 100) {
        throw new ValueError('imageavif(): Argument #3 ($quality) must be at between -1 and 100');
    }
    if ($speed < -1 || $speed > 10) {
        throw new ValueError('imageavif(): Argument #4 ($speed) must be at between -1 and 10');
    }
    // ext/gd maps the default speed itself (libgd would read -1 as 0).
    if ($speed === -1) {
        $speed = 6;
    }
    return __gd_output($image, 'avif', 'imageavif', $file, $quality, $speed);
}

// ---- environment ------------------------------------------------------------

function imagetypes(): int
{
    // IMG_GIF | IMG_JPG | IMG_PNG | IMG_WBMP | IMG_WEBP | IMG_BMP | IMG_TGA |
    // IMG_AVIF — what the linked libgd supports (no XPM), as the oracle reports.
    return 495;
}

function gd_info(): array
{
    return array(
        'GD Version' => __gd_version(),
        'FreeType Support' => true,
        'FreeType Linkage' => 'with freetype',
        'GIF Read Support' => true,
        'GIF Create Support' => true,
        'JPEG Support' => true,
        'PNG Support' => true,
        'WBMP Support' => true,
        'XPM Support' => false,
        'XBM Support' => true,
        'WebP Support' => true,
        'BMP Support' => true,
        'AVIF Support' => true,
        'TGA Read Support' => true,
        'JIS-mapped Japanese Font Support' => false,
    );
}
