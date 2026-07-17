// (segmento del prelude — concatenato via include_str! in lower/mod.rs;
//  NIENTE <?php qui: il tag di apertura vive solo in core.php)
interface DateTimeInterface {
    const ATOM = 'Y-m-d\TH:i:sP';
    const COOKIE = 'l, d-M-Y H:i:s T';
    const ISO8601 = 'Y-m-d\TH:i:sO';
    const ISO8601_EXPANDED = 'X-m-d\TH:i:sP';
    const RFC822 = 'D, d M y H:i:s O';
    const RFC850 = 'l, d-M-y H:i:s T';
    const RFC1036 = 'D, d M y H:i:s O';
    const RFC1123 = 'D, d M Y H:i:s O';
    const RFC7231 = 'D, d M Y H:i:s \G\M\T';
    const RFC2822 = 'D, d M Y H:i:s O';
    const RFC3339 = 'Y-m-d\TH:i:sP';
    const RFC3339_EXTENDED = 'Y-m-d\TH:i:s.vP';
    const RSS = 'D, d M Y H:i:s O';
    const W3C = 'Y-m-d\TH:i:sP';
}
// Date exception hierarchy (PHP 8.3+): only the members phpr actually throws.
class DateException extends Exception {}
class DateInvalidTimeZoneException extends DateException {}
// phpr models instants as UTC unix timestamps; the carried zone label (IANA
// name or "±HH:MM" offset) resolves through the __tz_* host builtins (TZif
// reader, D-DT3) for construction, formatting and calendar arithmetic.
class DateTimeZone {
    const AFRICA = 1;
    const AMERICA = 2;
    const ANTARCTICA = 4;
    const ARCTIC = 8;
    const ASIA = 16;
    const ATLANTIC = 32;
    const AUSTRALIA = 64;
    const EUROPE = 128;
    const INDIAN = 256;
    const PACIFIC = 512;
    const UTC = 1024;
    const ALL = 2047;
    const ALL_WITH_BC = 4095;
    const PER_COUNTRY = 4096;
    private $__name = "UTC";
    public function __construct($timezone = "UTC") {
        $tz = (string)$timezone;
        // PHP normalizes offset identifiers: "+0500"/"+05"/"GMT+2" → "+05:00"
        // and a lone "z" → "Z"; anything else must be a known identifier or
        // abbreviation, else DateInvalidTimeZoneException (oracle-pinned).
        if (preg_match('/^(?:GMT)?([+-])(\d{1,2}):?(\d{2})?$/', $tz, $m) === 1) {
            $tz = sprintf('%s%02d:%s', $m[1], (int)$m[2], isset($m[3]) && $m[3] !== '' ? $m[3] : '00');
        } elseif ($tz === 'z') {
            $tz = 'Z';
        } elseif (__tz_offset($tz, 0) === false) {
            throw new DateInvalidTimeZoneException("DateTimeZone::__construct(): Unknown or bad timezone ($tz)");
        }
        $this->__name = $tz;
    }
    public function getName() { return $this->__name; }
    public function getOffset($datetime) {
        $zi = __tz_offset($this->__name, $datetime->getTimestamp());
        return $zi === false ? false : $zi[0];
    }
    // var_dump shows the engine's debug pair (timezone_open_basic1):
    // type 1 = UTC offset, 2 = abbreviation, 3 = identifier.
    public function __debugInfo() {
        $n = $this->__name;
        $type = ($n !== '' && ($n[0] === '+' || $n[0] === '-')) ? 1
              : ((strpos($n, '/') !== false || $n === 'UTC') ? 3 : 2);
        return ['timezone_type' => $type, 'timezone' => $n];
    }
    // Transitions exist only for identifier zones (type 3): offset and
    // abbreviation zones return false, oracle-pinned. Row 0 is the state AT
    // $timestampBegin; wp-admin/options-general.php reads row 1 for the next
    // DST change.
    public function getTransitions($timestampBegin = PHP_INT_MIN, $timestampEnd = PHP_INT_MAX) {
        $n = $this->__name;
        $type = ($n !== '' && ($n[0] === '+' || $n[0] === '-')) ? 1
              : ((strpos($n, '/') !== false || $n === 'UTC') ? 3 : 2);
        if ($type !== 3) { return false; }
        $raw = __tz_transitions($n, $timestampBegin, $timestampEnd);
        if ($raw === false) { return false; }
        $out = [];
        foreach ($raw as $r) {
            $out[] = ['ts' => $r[0], 'time' => gmdate('Y-m-d\TH:i:sP', $r[0]),
                      'offset' => $r[1], 'isdst' => $r[2], 'abbr' => $r[3]];
        }
        return $out;
    }
    public function __toString() { return $this->__name; }
    // The oracle's 419 identifiers (macOS tzdata), comma-packed to keep the
    // prelude compact. ALL_WITH_BC (WP-10: WP's sanitize_option validates
    // timezone_string against it) appends the 179 backward-compat zones and
    // re-sorts case-insensitively, exactly like the oracle; PHP includes the
    // BC block only for the exact ALL_WITH_BC group (4095), not for the bare
    // 2048 bit. Country filtering is still not modelled.
    public static function listIdentifiers($timezoneGroup = DateTimeZone::ALL, $countryCode = null) {
        $ids = self::__identifiersAll();
        if ($timezoneGroup === DateTimeZone::ALL_WITH_BC) {
            $ids = array_merge($ids, explode(',', 'Africa/Asmera,Africa/Timbuktu,America/Argentina/ComodRivadavia,America/Atka,America/Buenos_Aires,America/Catamarca,America/Coral_Harbour,America/Cordoba,America/Ensenada,America/Fort_Wayne,America/Godthab,America/Indianapolis,America/Jujuy,America/Knox_IN,America/Louisville,America/Mendoza,America/Montreal,America/Nipigon,America/Pangnirtung,America/Porto_Acre,America/Rainy_River,America/Rosario,America/Santa_Isabel,America/Shiprock,America/Thunder_Bay,America/Virgin,America/Yellowknife,Antarctica/South_Pole,Asia/Ashkhabad,Asia/Calcutta,Asia/Choibalsan,Asia/Chongqing,Asia/Chungking,Asia/Dacca,Asia/Harbin,Asia/Istanbul,Asia/Kashgar,Asia/Katmandu,Asia/Macao,Asia/Rangoon,Asia/Saigon,Asia/Tel_Aviv,Asia/Thimbu,Asia/Ujung_Pandang,Asia/Ulan_Bator,Atlantic/Faeroe,Atlantic/Jan_Mayen,Australia/ACT,Australia/Canberra,Australia/Currie,Australia/LHI,Australia/North,Australia/NSW,Australia/Queensland,Australia/South,Australia/Tasmania,Australia/Victoria,Australia/West,Australia/Yancowinna,Brazil/Acre,Brazil/DeNoronha,Brazil/East,Brazil/West,Canada/Atlantic,Canada/Central,Canada/Eastern,Canada/Mountain,Canada/Newfoundland,Canada/Pacific,Canada/Saskatchewan,Canada/Yukon,CET,Chile/Continental,Chile/EasterIsland,CST6CDT,Cuba,EET,Egypt,Eire,EST,EST5EDT,Etc/GMT,Etc/GMT+0,Etc/GMT+1,Etc/GMT+10,Etc/GMT+11,Etc/GMT+12,Etc/GMT+2,Etc/GMT+3,Etc/GMT+4,Etc/GMT+5,Etc/GMT+6,Etc/GMT+7,Etc/GMT+8,Etc/GMT+9,Etc/GMT-0,Etc/GMT-1,Etc/GMT-10,Etc/GMT-11,Etc/GMT-12,Etc/GMT-13,Etc/GMT-14,Etc/GMT-2,Etc/GMT-3,Etc/GMT-4,Etc/GMT-5,Etc/GMT-6,Etc/GMT-7,Etc/GMT-8,Etc/GMT-9,Etc/GMT0,Etc/Greenwich,Etc/UCT,Etc/Universal,Etc/UTC,Etc/Zulu,Europe/Belfast,Europe/Kiev,Europe/Nicosia,Europe/Tiraspol,Europe/Uzhgorod,Europe/Zaporozhye,Factory,GB,GB-Eire,GMT,GMT+0,GMT-0,GMT0,Greenwich,Hongkong,HST,Iceland,Iran,Israel,Jamaica,Japan,Kwajalein,Libya,MET,Mexico/BajaNorte,Mexico/BajaSur,Mexico/General,MST,MST7MDT,Navajo,NZ,NZ-CHAT,Pacific/Enderbury,Pacific/Johnston,Pacific/Ponape,Pacific/Samoa,Pacific/Truk,Pacific/Yap,Poland,Portugal,PRC,PST8PDT,ROC,ROK,Singapore,Turkey,UCT,Universal,US/Alaska,US/Aleutian,US/Arizona,US/Central,US/East-Indiana,US/Eastern,US/Hawaii,US/Indiana-Starke,US/Michigan,US/Mountain,US/Pacific,US/Samoa,W-SU,WET,Zulu'));
            usort($ids, 'strcasecmp');
        }
        return $ids;
    }
    private static function __identifiersAll() {
        return explode(',', 'Africa/Abidjan,Africa/Accra,Africa/Addis_Ababa,Africa/Algiers,Africa/Asmara,Africa/Bamako,Africa/Bangui,Africa/Banjul,Africa/Bissau,Africa/Blantyre,Africa/Brazzaville,Africa/Bujumbura,Africa/Cairo,Africa/Casablanca,Africa/Ceuta,Africa/Conakry,Africa/Dakar,Africa/Dar_es_Salaam,Africa/Djibouti,Africa/Douala,Africa/El_Aaiun,Africa/Freetown,Africa/Gaborone,Africa/Harare,Africa/Johannesburg,Africa/Juba,Africa/Kampala,Africa/Khartoum,Africa/Kigali,Africa/Kinshasa,Africa/Lagos,Africa/Libreville,Africa/Lome,Africa/Luanda,Africa/Lubumbashi,Africa/Lusaka,Africa/Malabo,Africa/Maputo,Africa/Maseru,Africa/Mbabane,Africa/Mogadishu,Africa/Monrovia,Africa/Nairobi,Africa/Ndjamena,Africa/Niamey,Africa/Nouakchott,Africa/Ouagadougou,Africa/Porto-Novo,Africa/Sao_Tome,Africa/Tripoli,Africa/Tunis,Africa/Windhoek,America/Adak,America/Anchorage,America/Anguilla,America/Antigua,America/Araguaina,America/Argentina/Buenos_Aires,America/Argentina/Catamarca,America/Argentina/Cordoba,America/Argentina/Jujuy,America/Argentina/La_Rioja,America/Argentina/Mendoza,America/Argentina/Rio_Gallegos,America/Argentina/Salta,America/Argentina/San_Juan,America/Argentina/San_Luis,America/Argentina/Tucuman,America/Argentina/Ushuaia,America/Aruba,America/Asuncion,America/Atikokan,America/Bahia,America/Bahia_Banderas,America/Barbados,America/Belem,America/Belize,America/Blanc-Sablon,America/Boa_Vista,America/Bogota,America/Boise,America/Cambridge_Bay,America/Campo_Grande,America/Cancun,America/Caracas,America/Cayenne,America/Cayman,America/Chicago,America/Chihuahua,America/Ciudad_Juarez,America/Costa_Rica,America/Coyhaique,America/Creston,America/Cuiaba,America/Curacao,America/Danmarkshavn,America/Dawson,America/Dawson_Creek,America/Denver,America/Detroit,America/Dominica,America/Edmonton,America/Eirunepe,America/El_Salvador,America/Fort_Nelson,America/Fortaleza,America/Glace_Bay,America/Goose_Bay,America/Grand_Turk,America/Grenada,America/Guadeloupe,America/Guatemala,America/Guayaquil,America/Guyana,America/Halifax,America/Havana,America/Hermosillo,America/Indiana/Indianapolis,America/Indiana/Knox,America/Indiana/Marengo,America/Indiana/Petersburg,America/Indiana/Tell_City,America/Indiana/Vevay,America/Indiana/Vincennes,America/Indiana/Winamac,America/Inuvik,America/Iqaluit,America/Jamaica,America/Juneau,America/Kentucky/Louisville,America/Kentucky/Monticello,America/Kralendijk,America/La_Paz,America/Lima,America/Los_Angeles,America/Lower_Princes,America/Maceio,America/Managua,America/Manaus,America/Marigot,America/Martinique,America/Matamoros,America/Mazatlan,America/Menominee,America/Merida,America/Metlakatla,America/Mexico_City,America/Miquelon,America/Moncton,America/Monterrey,America/Montevideo,America/Montserrat,America/Nassau,America/New_York,America/Nome,America/Noronha,America/North_Dakota/Beulah,America/North_Dakota/Center,America/North_Dakota/New_Salem,America/Nuuk,America/Ojinaga,America/Panama,America/Paramaribo,America/Phoenix,America/Port-au-Prince,America/Port_of_Spain,America/Porto_Velho,America/Puerto_Rico,America/Punta_Arenas,America/Rankin_Inlet,America/Recife,America/Regina,America/Resolute,America/Rio_Branco,America/Santarem,America/Santiago,America/Santo_Domingo,America/Sao_Paulo,America/Scoresbysund,America/Sitka,America/St_Barthelemy,America/St_Johns,America/St_Kitts,America/St_Lucia,America/St_Thomas,America/St_Vincent,America/Swift_Current,America/Tegucigalpa,America/Thule,America/Tijuana,America/Toronto,America/Tortola,America/Vancouver,America/Whitehorse,America/Winnipeg,America/Yakutat,Antarctica/Casey,Antarctica/Davis,Antarctica/DumontDUrville,Antarctica/Macquarie,Antarctica/Mawson,Antarctica/McMurdo,Antarctica/Palmer,Antarctica/Rothera,Antarctica/Syowa,Antarctica/Troll,Antarctica/Vostok,Arctic/Longyearbyen,Asia/Aden,Asia/Almaty,Asia/Amman,Asia/Anadyr,Asia/Aqtau,Asia/Aqtobe,Asia/Ashgabat,Asia/Atyrau,Asia/Baghdad,Asia/Bahrain,Asia/Baku,Asia/Bangkok,Asia/Barnaul,Asia/Beirut,Asia/Bishkek,Asia/Brunei,Asia/Chita,Asia/Colombo,Asia/Damascus,Asia/Dhaka,Asia/Dili,Asia/Dubai,Asia/Dushanbe,Asia/Famagusta,Asia/Gaza,Asia/Hebron,Asia/Ho_Chi_Minh,Asia/Hong_Kong,Asia/Hovd,Asia/Irkutsk,Asia/Jakarta,Asia/Jayapura,Asia/Jerusalem,Asia/Kabul,Asia/Kamchatka,Asia/Karachi,Asia/Kathmandu,Asia/Khandyga,Asia/Kolkata,Asia/Krasnoyarsk,Asia/Kuala_Lumpur,Asia/Kuching,Asia/Kuwait,Asia/Macau,Asia/Magadan,Asia/Makassar,Asia/Manila,Asia/Muscat,Asia/Nicosia,Asia/Novokuznetsk,Asia/Novosibirsk,Asia/Omsk,Asia/Oral,Asia/Phnom_Penh,Asia/Pontianak,Asia/Pyongyang,Asia/Qatar,Asia/Qostanay,Asia/Qyzylorda,Asia/Riyadh,Asia/Sakhalin,Asia/Samarkand,Asia/Seoul,Asia/Shanghai,Asia/Singapore,Asia/Srednekolymsk,Asia/Taipei,Asia/Tashkent,Asia/Tbilisi,Asia/Tehran,Asia/Thimphu,Asia/Tokyo,Asia/Tomsk,Asia/Ulaanbaatar,Asia/Urumqi,Asia/Ust-Nera,Asia/Vientiane,Asia/Vladivostok,Asia/Yakutsk,Asia/Yangon,Asia/Yekaterinburg,Asia/Yerevan,Atlantic/Azores,Atlantic/Bermuda,Atlantic/Canary,Atlantic/Cape_Verde,Atlantic/Faroe,Atlantic/Madeira,Atlantic/Reykjavik,Atlantic/South_Georgia,Atlantic/St_Helena,Atlantic/Stanley,Australia/Adelaide,Australia/Brisbane,Australia/Broken_Hill,Australia/Darwin,Australia/Eucla,Australia/Hobart,Australia/Lindeman,Australia/Lord_Howe,Australia/Melbourne,Australia/Perth,Australia/Sydney,Europe/Amsterdam,Europe/Andorra,Europe/Astrakhan,Europe/Athens,Europe/Belgrade,Europe/Berlin,Europe/Bratislava,Europe/Brussels,Europe/Bucharest,Europe/Budapest,Europe/Busingen,Europe/Chisinau,Europe/Copenhagen,Europe/Dublin,Europe/Gibraltar,Europe/Guernsey,Europe/Helsinki,Europe/Isle_of_Man,Europe/Istanbul,Europe/Jersey,Europe/Kaliningrad,Europe/Kirov,Europe/Kyiv,Europe/Lisbon,Europe/Ljubljana,Europe/London,Europe/Luxembourg,Europe/Madrid,Europe/Malta,Europe/Mariehamn,Europe/Minsk,Europe/Monaco,Europe/Moscow,Europe/Oslo,Europe/Paris,Europe/Podgorica,Europe/Prague,Europe/Riga,Europe/Rome,Europe/Samara,Europe/San_Marino,Europe/Sarajevo,Europe/Saratov,Europe/Simferopol,Europe/Skopje,Europe/Sofia,Europe/Stockholm,Europe/Tallinn,Europe/Tirane,Europe/Ulyanovsk,Europe/Vaduz,Europe/Vatican,Europe/Vienna,Europe/Vilnius,Europe/Volgograd,Europe/Warsaw,Europe/Zagreb,Europe/Zurich,Indian/Antananarivo,Indian/Chagos,Indian/Christmas,Indian/Cocos,Indian/Comoro,Indian/Kerguelen,Indian/Mahe,Indian/Maldives,Indian/Mauritius,Indian/Mayotte,Indian/Reunion,Pacific/Apia,Pacific/Auckland,Pacific/Bougainville,Pacific/Chatham,Pacific/Chuuk,Pacific/Easter,Pacific/Efate,Pacific/Fakaofo,Pacific/Fiji,Pacific/Funafuti,Pacific/Galapagos,Pacific/Gambier,Pacific/Guadalcanal,Pacific/Guam,Pacific/Honolulu,Pacific/Kanton,Pacific/Kiritimati,Pacific/Kosrae,Pacific/Kwajalein,Pacific/Majuro,Pacific/Marquesas,Pacific/Midway,Pacific/Nauru,Pacific/Niue,Pacific/Norfolk,Pacific/Noumea,Pacific/Pago_Pago,Pacific/Palau,Pacific/Pitcairn,Pacific/Pohnpei,Pacific/Port_Moresby,Pacific/Rarotonga,Pacific/Saipan,Pacific/Tahiti,Pacific/Tarawa,Pacific/Tongatapu,Pacific/Wake,Pacific/Wallis,UTC');
    }
}
class DateTime implements DateTimeInterface {
    private $__ts = 0;
    private $__us = 0;
    private $__tz = "UTC";
    public function __construct($datetime = "now", $timezone = null) {
        // The instance zone: the $timezone argument, else the process default.
        // A zone carried by the STRING itself (offset suffix, "UTC", a "@"
        // timestamp's "+00:00") overrides both (PHP quirk).
        $this->__tz = $timezone !== null ? $timezone->getName() : date_default_timezone_get();
        if ($datetime === "now" || $datetime === "" || $datetime === null) {
            $t = microtime(true);
            $this->__ts = (int) $t;
            $this->__us = (int) round(($t - (int) $t) * 1000000);
        } else {
            $parse = $datetime;
            if (is_string($datetime) && preg_match('/\.(\d{1,6})/', $datetime, $m) === 1) {
                $this->__us = (int) str_pad($m[1], 6, '0');
                $parse = preg_replace('/\.\d{1,6}/', '', $datetime, 1);
            }
            $r = __strtotime_tz($parse, null, $this->__tz);
            if ($r === false) {
                throw new Exception("DateTime::__construct(): Failed to parse time string ($datetime)");
            }
            $this->__ts = $r[0];
            if ($r[1] !== null) { $this->__tz = $r[1]; }
        }
    }
    public function getTimezone() { return new DateTimeZone($this->__tz); }
    public function getOffset() {
        $zi = __tz_offset($this->__tz, $this->__ts);
        return $zi === false ? 0 : $zi[0];
    }
    public function setTimezone($timezone) {
        $this->__tz = is_string($timezone) ? $timezone : $timezone->getName();
        return $this;
    }
    public function __unserialize(array $data) {
        // Accept both this class's own state keys and the MANGLED private
        // slots an `(array)` cast produces (symfony DatePoint's constructor
        // does `$this->__unserialize((array) $now)` on our instances). Any
        // OTHER key is a subclass property (inherited serialization round
        // trip) and is restored under its unmangled name.
        foreach ($data as $k => $v) {
            if (substr($k, -4) === '__ts') { $this->__ts = $v; }
            elseif (substr($k, -4) === '__us') { $this->__us = $v; }
            elseif (substr($k, -4) === '__tz') { $this->__tz = $v; }
            else {
                $p = strrpos($k, "\0");
                $this->{$p === false ? $k : substr($k, $p + 1)} = $v;
            }
        }
    }
    public static function createFromInterface($object) {
        // PHP's C implementation returns `static` and creates the instance
        // WITHOUT invoking the subclass constructor (symfony DatePoint's
        // would re-enter Clock::get() -> now() -> createFromInterface).
        $d = (new ReflectionClass(static::class))->newInstanceWithoutConstructor();
        $d->__ts = $object->getTimestamp();
        $d->__us = (int) $object->format('u');
        $d->__tz = $object->getTimezone()->getName();
        return $d;
    }
    public static function createFromImmutable($object) { return static::createFromInterface($object); }
    public function format($format) { return DateTime::__fmt($format, $this->__ts, $this->__us, $this->__tz); }
    // The shared DateTimeInterface::format engine: every zone-dependent
    // specifier (O/P/U/Z/T/e/I, the composite c/r, and this instance's
    // u/v microseconds) is emitted as backslash-escaped literals resolved
    // from the instance zone via __tz_offset; the remaining civil fields
    // come from gmdate() over the zone-shifted timestamp.
    public static function __fmt($format, $ts, $us, $tzs) {
        $zi = __tz_offset($tzs, $ts);
        $off = $zi === false ? 0 : $zi[0];
        $sign = $off < 0 ? '-' : '+';
        $oh = intdiv(abs($off), 3600); $om = intdiv(abs($off) % 3600, 60);
        $out = ''; $esc = false;
        for ($i = 0, $len = strlen($format); $i < $len; $i++) {
            $c = $format[$i];
            if ($esc) { $out .= '\\' . $c; $esc = false; continue; }
            if ($c === '\\') { $esc = true; continue; }
            // PHP renders the composite specifiers with the INSTANCE offset:
            // expand them so the trailing zone part is a literal.
            if ($c === 'c') { $out .= 'Y-m-d\\TH:i:s'; $c = 'P'; }
            elseif ($c === 'r') { $out .= 'D, d M Y H:i:s '; $c = 'O'; }
            $lit = match ($c) {
                'O' => sprintf('%s%02d%02d', $sign, $oh, $om),
                'P', 'p' => sprintf('%s%02d:%02d', $sign, $oh, $om),
                'U' => (string) $ts,
                'Z' => (string) $off,
                'T' => $zi === false ? $tzs : $zi[1],
                'e' => $tzs,
                'I' => ($zi !== false && $zi[2]) ? '1' : '0',
                'u' => sprintf('%06d', $us),
                'v' => sprintf('%03d', intdiv($us, 1000)),
                default => null,
            };
            if ($lit !== null) {
                foreach (str_split($lit) as $d) { $out .= '\\' . $d; }
                continue;
            }
            $out .= $c;
        }
        return gmdate($out, $ts + $off);
    }
    public function getTimestamp() { return $this->__ts; }
    public function setTimestamp($timestamp) { $this->__ts = $timestamp; return $this; }
    public function setDate($year, $month, $day) {
        $w = $this->__ts + $this->getOffset();
        $this->__ts = __tz_wall_ts($this->__tz, gmmktime((int)gmdate('G', $w), (int)gmdate('i', $w), (int)gmdate('s', $w), $month, $day, $year));
        return $this;
    }
    public function setTime($hour, $minute, $second = 0) {
        $w = $this->__ts + $this->getOffset();
        $this->__ts = __tz_wall_ts($this->__tz, gmmktime($hour, $minute, $second, (int)gmdate('n', $w), (int)gmdate('j', $w), (int)gmdate('Y', $w)));
        return $this;
    }
    public static function createFromFormat($format, $datetime, $timezone = null) {
        $r = __date_from_format($format, $datetime);
        if ($r === false) { return false; }
        // The parsed value is a WALL time unless the string carried a zone
        // (then $r[0] is already the epoch and $r[1] its display label); the
        // wall time anchors in the $timezone argument or the default zone.
        $tzName = $r[1] !== null ? $r[1] : ($timezone !== null ? $timezone->getName() : date_default_timezone_get());
        $d = new DateTime("@" . ($r[1] !== null ? $r[0] : __tz_wall_ts($tzName, $r[0])));
        $d->__tz = $tzName;
        $d->__us = $r[2];
        return $d;
    }
    // Warnings/errors of the last createFromFormat parse; false when clean.
    public static function getLastErrors() { return __date_get_last_errors(); }
    public function modify($modifier) {
        // Relative math runs on this instance's zone (wall-clock preserving
        // across DST), not the process default.
        $r = __strtotime_tz($modifier, $this->__ts, $this->__tz);
        if ($r !== false) { $this->__ts = $r[0]; }
        return $this;
    }
    public function add($interval) { $this->__ts = $this->__apply($interval, 1); return $this; }
    public function sub($interval) { $this->__ts = $this->__apply($interval, -1); return $this; }
    private function __apply($iv, $dir) {
        // timelib_add: the Y/M/D part is calendar arithmetic on the WALL
        // clock, re-anchored in the zone (a +P1D across a DST jump keeps the
        // wall time); the H/I/S part then moves LINEARLY on the epoch
        // (bug80610: ±PT20800M across a DST edge must round-trip exactly).
        $sign = $dir * ($iv->invert ? -1 : 1);
        $ts = $this->__ts;
        if ($iv->y || $iv->m || $iv->d) {
            $w = $ts + $this->getOffset();
            $ts = __tz_wall_ts($this->__tz, gmmktime(
                (int)gmdate('G', $w),
                (int)gmdate('i', $w),
                (int)gmdate('s', $w),
                (int)gmdate('n', $w) + $sign * $iv->m,
                (int)gmdate('j', $w) + $sign * $iv->d,
                (int)gmdate('Y', $w) + $sign * $iv->y));
        }
        return $ts + $sign * ($iv->h * 3600 + $iv->i * 60 + $iv->s);
    }
    public function diff($other) { return DateTime::__diff($this, $other); }
    // Port of timelib_diff (ext/date/lib/interval.c). Same named zone →
    // timelib_diff_with_tzid: calendar diff of the two WALL times plus the
    // fall-back / spring-forward corrections around a transition. Anything
    // else → the general branch: local-field diff sorted by instant with the
    // offset delta folded into the seconds, and days = |Δsse| / 86400.
    public static function __diff($a, $b) {
        $tzA = $a->getTimezone()->getName(); $tzB = $b->getTimezone()->getName();
        $tsA = $a->getTimestamp();           $tsB = $b->getTimestamp();
        $zA = $a->getOffset();               $zB = $b->getOffset();
        $iv = new DateInterval("PT0S");
        $named = $tzA !== 'Z' && (!isset($tzA[0]) || ($tzA[0] !== '+' && $tzA[0] !== '-'));
        if ($tzA === $tzB && $named) {
            $invert = 0;
            if ($tsA + $zA > $tsB + $zB) {
                [$one, $two, $zOne, $zTwo] = [$tsB, $tsA, $zB, $zA];
                $invert = 1;
            } else {
                [$one, $two, $zOne, $zTwo] = [$tsA, $tsB, $zA, $zB];
            }
            $dstOne = __tz_offset($tzA, $one)[2]; $dstTwo = __tz_offset($tzA, $two)[2];
            $dstCorr = $zTwo - $zOne;
            $info = __date_diff($one + $zOne, $two + $zTwo);
            $d = $info['d']; $h = $info['h']; $i = $info['i']; $s = $info['s'];
            if ($dstOne && !$dstTwo) { // fall back
                if (($two - $one + $dstCorr) < 86400) {
                    $h -= intdiv($dstCorr, 3600); $i -= intdiv($dstCorr % 3600, 60);
                }
            } elseif (!$dstOne && $dstTwo) { // spring forward
                $tr = __tz_transition($tzA, $two);
                if (!(($one + 86400 > $tr[1]) && ($one + 86400 <= $tr[1] + $dstCorr))
                    && $two >= $tr[1]
                    && (($two - $one + $dstCorr) % 86400) > ($two - $tr[1])) {
                    $h -= intdiv($dstCorr, 3600); $i -= intdiv($dstCorr % 3600, 60);
                }
            } elseif ($two - $one >= 86400) {
                // Transition-period special case (see interval.c:131).
                $tr = __tz_transition($tzA, $two - $zTwo);
                $corr = $zOne - $tr[0];
                if ($two >= $tr[1] - $corr && $two < $tr[1]) { $d--; $h = 24; }
            }
            $iv->y = $info['y']; $iv->m = $info['m']; $iv->d = $d;
            $iv->h = $h; $iv->i = $i; $iv->s = $s;
            $iv->invert = $invert; $iv->days = $info['days'];
        } else {
            $invert = 0;
            if ($tsA > $tsB) {
                [$one, $two, $zOne] = [$tsB, $tsA, $zB];
                $invert = 1;
            } else {
                [$one, $two, $zOne] = [$tsA, $tsB, $zA];
            }
            // Field diff with the offset delta folded into the seconds ==
            // both walls rendered with the EARLIER side's offset.
            $info = __date_diff($one + $zOne, $two + $zOne);
            $iv->y = $info['y']; $iv->m = $info['m']; $iv->d = $info['d'];
            $iv->h = $info['h']; $iv->i = $info['i']; $iv->s = $info['s'];
            $iv->invert = $invert; $iv->days = intdiv(abs($two - $one), 86400);
        }
        return $iv;
    }
}
class DateInterval {
    public $y = 0;
    public $m = 0;
    public $d = 0;
    public $h = 0;
    public $i = 0;
    public $s = 0;
    public $f = 0;
    public $invert = 0;
    public $days = false;
    public function __construct($duration) {
        $p = __interval_parse($duration);
        if ($p === false) {
            throw new Exception("DateInterval::__construct(): Unknown or bad format ($duration)");
        }
        $this->y = $p['y']; $this->m = $p['m']; $this->d = $p['d'];
        $this->h = $p['h']; $this->i = $p['i']; $this->s = $p['s'];
    }
    public function format($format) { return __interval_format($this, $format); }
}
class DateTimeImmutable implements DateTimeInterface {
    private $__ts = 0;
    private $__us = 0;
    private $__tz = "UTC";
    public function __construct($datetime = "now", $timezone = null) {
        // Same zone-resolution order as DateTime::__construct.
        $this->__tz = $timezone !== null ? $timezone->getName() : date_default_timezone_get();
        if ($datetime === "now" || $datetime === "" || $datetime === null) {
            $t = microtime(true);
            $this->__ts = (int) $t;
            $this->__us = (int) round(($t - (int) $t) * 1000000);
        } else {
            $parse = $datetime;
            if (is_string($datetime) && preg_match('/\.(\d{1,6})/', $datetime, $m) === 1) {
                $this->__us = (int) str_pad($m[1], 6, '0');
                $parse = preg_replace('/\.\d{1,6}/', '', $datetime, 1);
            }
            $r = __strtotime_tz($parse, null, $this->__tz);
            if ($r === false) {
                throw new Exception("DateTimeImmutable::__construct(): Failed to parse time string ($datetime)");
            }
            $this->__ts = $r[0];
            if ($r[1] !== null) { $this->__tz = $r[1]; }
        }
    }
    public function getTimezone() { return new DateTimeZone($this->__tz); }
    public function getOffset() {
        $zi = __tz_offset($this->__tz, $this->__ts);
        return $zi === false ? 0 : $zi[0];
    }
    public function setTimezone($timezone) {
        // `clone` keeps the runtime class, so a userland subclass (monolog's
        // JsonSerializableDateTimeImmutable) survives, like PHP's `static`.
        $c = clone $this;
        $c->__tz = is_string($timezone) ? $timezone : $timezone->getName();
        return $c;
    }
    public function __unserialize(array $data) {
        // Accept both this class's own state keys and the MANGLED private
        // slots an `(array)` cast produces (symfony DatePoint's constructor
        // does `$this->__unserialize((array) $now)` on our instances). Any
        // OTHER key is a subclass property (inherited serialization round
        // trip) and is restored under its unmangled name.
        foreach ($data as $k => $v) {
            if (substr($k, -4) === '__ts') { $this->__ts = $v; }
            elseif (substr($k, -4) === '__us') { $this->__us = $v; }
            elseif (substr($k, -4) === '__tz') { $this->__tz = $v; }
            else {
                $p = strrpos($k, "\0");
                $this->{$p === false ? $k : substr($k, $p + 1)} = $v;
            }
        }
    }
    public static function createFromInterface($object) {
        // PHP's C implementation returns `static` and creates the instance
        // WITHOUT invoking the subclass constructor (symfony DatePoint's
        // would re-enter Clock::get() -> now() -> createFromInterface).
        $d = (new ReflectionClass(static::class))->newInstanceWithoutConstructor();
        $d->__ts = $object->getTimestamp();
        $d->__us = (int) $object->format('u');
        $d->__tz = $object->getTimezone()->getName();
        return $d;
    }
    public static function createFromMutable($object) { return static::createFromInterface($object); }
    public function format($format) { return DateTime::__fmt($format, $this->__ts, $this->__us, $this->__tz); }
    public function getTimestamp() { return $this->__ts; }
    // Every "wither" clones: the runtime class survives (PHP returns `static`,
    // monolog's JsonSerializableDateTimeImmutable relies on it), and so do
    // the timezone label and (where PHP keeps them) the microseconds.
    public function setTimestamp($timestamp) {
        $c = clone $this; $c->__ts = $timestamp; $c->__us = 0; return $c;
    }
    public function setDate($year, $month, $day) {
        $c = clone $this;
        $w = $this->__ts + $this->getOffset();
        $c->__ts = __tz_wall_ts($this->__tz, gmmktime((int)gmdate('G', $w), (int)gmdate('i', $w), (int)gmdate('s', $w), $month, $day, $year));
        return $c;
    }
    public function setTime($hour, $minute, $second = 0, $microsecond = 0) {
        $c = clone $this;
        $w = $this->__ts + $this->getOffset();
        $c->__ts = __tz_wall_ts($this->__tz, gmmktime($hour, $minute, $second, (int)gmdate('n', $w), (int)gmdate('j', $w), (int)gmdate('Y', $w)));
        $c->__us = $microsecond;
        return $c;
    }
    public static function createFromFormat($format, $datetime, $timezone = null) {
        $r = __date_from_format($format, $datetime);
        if ($r === false) { return false; }
        // Wall time unless the string carried a zone (see DateTime).
        $tzName = $r[1] !== null ? $r[1] : ($timezone !== null ? $timezone->getName() : date_default_timezone_get());
        $d = new DateTimeImmutable("@" . ($r[1] !== null ? $r[0] : __tz_wall_ts($tzName, $r[0])));
        $d->__tz = $tzName;
        $d->__us = $r[2];
        return $d;
    }
    // Warnings/errors of the last createFromFormat parse; false when clean.
    public static function getLastErrors() { return __date_get_last_errors(); }
    public function modify($modifier) {
        // Relative math runs on this instance's zone (see DateTime::modify).
        $r = __strtotime_tz($modifier, $this->__ts, $this->__tz);
        if ($r === false) { return false; }
        $c = clone $this; $c->__ts = $r[0]; return $c;
    }
    public function add($interval) { $c = clone $this; $c->__ts = $this->__apply($interval, 1); return $c; }
    public function sub($interval) { $c = clone $this; $c->__ts = $this->__apply($interval, -1); return $c; }
    private function __apply($iv, $dir) {
        // timelib_add: Y/M/D wall-calendar re-anchored in the zone, then
        // H/I/S linear on the epoch (see DateTime::__apply, bug80610).
        $sign = $dir * ($iv->invert ? -1 : 1);
        $ts = $this->__ts;
        if ($iv->y || $iv->m || $iv->d) {
            $w = $ts + $this->getOffset();
            $ts = __tz_wall_ts($this->__tz, gmmktime(
                (int)gmdate('G', $w),
                (int)gmdate('i', $w),
                (int)gmdate('s', $w),
                (int)gmdate('n', $w) + $sign * $iv->m,
                (int)gmdate('j', $w) + $sign * $iv->d,
                (int)gmdate('Y', $w) + $sign * $iv->y));
        }
        return $ts + $sign * ($iv->h * 3600 + $iv->i * 60 + $iv->s);
    }
    public function diff($other) { return DateTime::__diff($this, $other); }
}

// --- Procedural date API (step 35): thin global-function wrappers over the OOP
// API above. PHP exposes both styles; these delegate so the two stay identical.
// date_create returns FALSE where the constructor throws (WP's
// wp_resolve_post_date feeds it arbitrary user strings), and it must pass
// the timezone through (get_gmt_from_date anchors wall times with it).
function date_create($datetime = "now", $timezone = null) {
    try {
        return $timezone === null ? new DateTime($datetime) : new DateTime($datetime, $timezone);
    } catch (Exception $e) {
        return false;
    }
}
function timezone_identifiers_list($timezoneGroup = DateTimeZone::ALL, $countryCode = null) { return DateTimeZone::listIdentifiers($timezoneGroup, $countryCode); }
// timezone_open() returns false where the constructor throws (WP's
// wp_timezone_override_offset probes the option value exactly this way).
function timezone_open($timezone) {
    try { return new DateTimeZone($timezone); } catch (Exception $e) { return false; }
}
function timezone_offset_get($object, $datetime) {
    if (!($object instanceof DateTimeZone)) {
        throw new TypeError("timezone_offset_get(): Argument #1 (\$object) must be of type DateTimeZone, " . get_debug_type($object) . " given");
    }
    if (!($datetime instanceof DateTimeInterface)) {
        throw new TypeError("timezone_offset_get(): Argument #2 (\$datetime) must be of type DateTimeInterface, " . get_debug_type($datetime) . " given");
    }
    return $object->getOffset($datetime);
}
function timezone_name_get($object) {
    if (!($object instanceof DateTimeZone)) {
        throw new TypeError("timezone_name_get(): Argument #1 (\$object) must be of type DateTimeZone, " . get_debug_type($object) . " given");
    }
    return $object->getName();
}
function timezone_transitions_get($object, $timestampBegin = PHP_INT_MIN, $timestampEnd = PHP_INT_MAX) {
    if (!($object instanceof DateTimeZone)) {
        throw new TypeError("timezone_transitions_get(): Argument #1 (\$object) must be of type DateTimeZone, " . get_debug_type($object) . " given");
    }
    return $object->getTransitions($timestampBegin, $timestampEnd);
}
function date_create_immutable($datetime = "now", $timezone = null) {
    try {
        return $timezone === null ? new DateTimeImmutable($datetime) : new DateTimeImmutable($datetime, $timezone);
    } catch (Exception $e) {
        return false;
    }
}
function date_format($object, $format) { return $object->format($format); }
function date_timestamp_get($object) { return $object->getTimestamp(); }
function date_diff($base, $target, $absolute = false) {
    $r = $base->diff($target);
    if ($absolute) { $r->invert = 0; }
    return $r;
}
function date_add($object, $interval) { return $object->add($interval); }
function date_sub($object, $interval) { return $object->sub($interval); }
function date_modify($object, $modifier) { return $object->modify($modifier); }
function date_date_set($object, $year, $month, $day) { return $object->setDate($year, $month, $day); }
function date_time_set($object, $hour, $minute, $second = 0) { return $object->setTime($hour, $minute, $second); }
function date_timestamp_set($object, $timestamp) { return $object->setTimestamp($timestamp); }
function date_create_from_format($format, $datetime, $timezone = null) { return DateTime::createFromFormat($format, $datetime, $timezone); }
function date_create_immutable_from_format($format, $datetime, $timezone = null) { return DateTimeImmutable::createFromFormat($format, $datetime, $timezone); }
function date_interval_format($object, $format) { return $object->format($format); }
function date_interval_create_from_date_string($datetime) {
    $p = __interval_from_date_string($datetime);
    if ($p === false) { return false; }
    $iv = new DateInterval("PT0S");
    $iv->y = $p['y']; $iv->m = $p['m']; $iv->d = $p['d'];
    $iv->h = $p['h']; $iv->i = $p['i']; $iv->s = $p['s'];
    return $iv;
}
