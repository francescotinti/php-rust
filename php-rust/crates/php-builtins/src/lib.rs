//! PHP builtin functions (Tier 1 nucleus, plan step 5).
//!
//! Each builtin has the [`php_runtime::BuiltinFn`] signature and is registered
//! by name in [`registry`]. The evaluator dispatches to them through the
//! injected registry (see `php_runtime::builtin`), so this crate depends on
//! php-runtime, not the other way around.
//!
//! Scope: `var_dump`, `strlen`, `gettype`, the `is_*` predicate family, and the
//! `*val` cast helpers. Frequency-driven expansion (implode, count, substr,
//! sprintf, array_*) is step 10.

mod array;
mod bcmath;
mod crypto;
mod csv;
mod ctype;
mod curl;
mod date;
mod dateparse;
mod encoding;
mod env;
mod html;
mod file;
mod format;
mod gmp;
mod grapheme;
mod image;
mod json;
mod math;
mod mbstring;
mod net;
mod openssl;
mod pack;
mod serialize;
mod string;
mod url;
mod var;
mod zlib;
pub(crate) use var::*;

use std::rc::Rc;

use php_runtime::{Builtin, Ctx, Registry};
use php_types::{
    convert, dtoa, numstr, Closure, ClosureRender, Diag, Diags, Key, PhpArray, PhpError, PhpStr,
    PropVis, Zval,
};

/// Build the Tier 1 builtin registry.
pub fn registry() -> Registry {
    let mut r = Registry::new();
    let mut add = |name: &[u8], f: php_runtime::BuiltinFn| {
        r.insert(name.to_vec(), Builtin::Value(f));
    };
    add(b"count", array::count);
    add(b"sizeof", array::count);
    add(b"date", date::date);
    add(b"gmdate", date::gmdate);
    add(b"idate", date::idate);
    add(b"ip2long", net::ip2long);
    add(b"long2ip", net::long2ip);
    add(b"inet_pton", net::inet_pton);
    add(b"inet_ntop", net::inet_ntop);
    add(b"bcadd", bcmath::bcadd);
    add(b"bcsub", bcmath::bcsub);
    add(b"bcmul", bcmath::bcmul);
    add(b"bcdiv", bcmath::bcdiv);
    add(b"bcmod", bcmath::bcmod);
    add(b"bcdivmod", bcmath::bcdivmod);
    add(b"bcpow", bcmath::bcpow);
    add(b"bcpowmod", bcmath::bcpowmod);
    add(b"bcsqrt", bcmath::bcsqrt);
    add(b"bccomp", bcmath::bccomp);
    add(b"bcscale", bcmath::bcscale);
    add(b"bcfloor", bcmath::bcfloor);
    add(b"bcceil", bcmath::bcceil);
    add(b"bcround", bcmath::bcround);
    // gmp low-level primitives (the `GMP` class + gmp_* wrappers live in the prelude).
    add(b"_gmp_parse", gmp::parse);
    add(b"_gmp_strval", gmp::strval);
    add(b"_gmp_intval", gmp::intval);
    add(b"_gmp_cmp", gmp::cmp);
    add(b"_gmp_sign", gmp::sign);
    add(b"_gmp_divq", gmp::divq);
    add(b"_gmp_divr", gmp::divr);
    add(b"_gmp_divqr", gmp::divqr);
    add(b"_gmp_mod", gmp::modulo);
    add(b"_gmp_bin", gmp::bin);
    add(b"_gmp_un", gmp::un);
    add(b"_gmp_powm", gmp::powm);
    add(b"_gmp_gcdext", gmp::gcdext);
    add(b"_gmp_invert", gmp::invert);
    add(b"_gmp_root", gmp::root);
    add(b"_gmp_rootrem", gmp::rootrem);
    add(b"_gmp_sqrtrem", gmp::sqrtrem);
    add(b"_gmp_fact", gmp::fact);
    add(b"_gmp_binomial", gmp::binomial);
    add(b"_gmp_probprime", gmp::probprime);
    add(b"_gmp_nextprime", gmp::nextprime);
    add(b"_gmp_kronecker", gmp::kronecker);
    add(b"_gmp_perfsquare", gmp::perfsquare);
    add(b"_gmp_perfpower", gmp::perfpower);
    add(b"_gmp_setbit", gmp::setbit);
    add(b"_gmp_testbit", gmp::testbit);
    add(b"_gmp_scan0", gmp::scan0);
    add(b"_gmp_scan1", gmp::scan1);
    add(b"_gmp_popcount", gmp::popcount);
    add(b"_gmp_hamdist", gmp::hamdist);
    add(b"date_parse", dateparse::date_parse);
    add(b"mktime", date::mktime);
    add(b"gmmktime", date::gmmktime);
    add(b"checkdate", date::checkdate);
    add(b"strtotime", date::strtotime);
    add(b"time", date::time);
    add(b"microtime", date::microtime);
    add(b"hrtime", date::hrtime);
    add(b"getrusage", date::getrusage);
    add(b"date_default_timezone_set", date::date_default_timezone_set);
    add(b"date_default_timezone_get", date::date_default_timezone_get);
    add(b"getdate", date::getdate);
    add(b"localtime", date::localtime);
    add(b"__interval_parse", date::__interval_parse);
    add(b"__interval_from_date_string", date::__interval_from_date_string);
    add(b"__date_diff", date::__date_diff);
    add(b"__interval_format", date::__interval_format);
    add(b"__date_from_format", date::__date_from_format);
    add(b"json_encode", json::json_encode);
    // ext/curl easy-handle facade (curl.rs); the curl_* PHP surface is prelude
    // functions wrapping these id-keyed hosts around the CurlHandle class.
    add(b"__curl_init", curl::__curl_init);
    add(b"__curl_setopt", curl::__curl_setopt);
    add(b"__curl_exec", curl::__curl_exec);
    add(b"__curl_errno", curl::__curl_errno);
    add(b"__curl_error", curl::__curl_error);
    add(b"__curl_reset", curl::__curl_reset);
    add(b"__curl_getinfo", curl::__curl_getinfo);
    add(b"curl_strerror", curl::curl_strerror);
    add(b"curl_close", curl::curl_close);
    add(b"posix_getpid", env::posix_getpid);
    add(b"strpbrk", string::strpbrk);
    add(b"escapeshellarg", string::escapeshellarg);
    add(b"escapeshellcmd", string::escapeshellcmd);
    // Hashing / encoding builtins (step 62).
    add(b"base64_encode", encoding::base64_encode);
    add(b"base64_decode", encoding::base64_decode);
    add(b"md5", encoding::md5);
    add(b"md5_file", encoding::md5_file);
    add(b"quoted_printable_encode", encoding::quoted_printable_encode);
    add(b"quoted_printable_decode", encoding::quoted_printable_decode);
    add(b"utf8_encode", encoding::utf8_encode);
    add(b"utf8_decode", encoding::utf8_decode);
    add(b"sha1", encoding::sha1);
    add(b"sha1_file", encoding::sha1_file);
    add(b"crc32", encoding::crc32);
    add(b"hash", encoding::hash);
    add(b"hash_equals", encoding::hash_equals);
    add(b"hash_hmac", encoding::hash_hmac);
    add(b"pack", pack::pack);
    add(b"unpack", pack::unpack);
    add(b"crypt", crypto::crypt);
    add(b"password_hash", crypto::password_hash);
    add(b"password_verify", crypto::password_verify);
    add(b"password_get_info", crypto::password_get_info);
    add(b"password_needs_rehash", crypto::password_needs_rehash);
    add(b"password_algos", crypto::password_algos);
    // File / stream builtins (step 51; `fopen` is evaluator-dispatched).
    add(b"fread", file::fread);
    add(b"fwrite", file::fwrite);
    add(b"stream_isatty", file::stream_isatty);
    add(b"stream_set_blocking", file::stream_set_blocking);
    add(b"set_socket_blocking", file::stream_set_blocking); // legacy alias
    add(b"fputs", file::fwrite);
    add(b"fclose", file::fclose);
    add(b"fgets", file::fgets);
    add(b"fgetc", file::fgetc);
    add(b"feof", file::feof);
    add(b"fseek", file::fseek);
    add(b"ftell", file::ftell);
    add(b"rewind", file::rewind);
    add(b"fflush", file::fflush);
    add(b"flock", file::flock);
    add(b"file_get_contents", file::file_get_contents);
    add(b"php_strip_whitespace", file::php_strip_whitespace);
    add(b"http_get_last_response_headers", file::http_get_last_response_headers);
    add(b"http_clear_last_response_headers", file::http_clear_last_response_headers);
    add(b"file_put_contents", file::file_put_contents);
    add(b"error_log", file::error_log);
    add(b"getimagesize", image::getimagesize);
    add(b"getimagesizefromstring", image::getimagesizefromstring);
    add(b"image_type_to_mime_type", image::image_type_to_mime_type);
    add(b"image_type_to_extension", image::image_type_to_extension);
    add(b"file", file::file);
    add(b"readfile", file::readfile);
    add(b"fpassthru", file::fpassthru);
    add(b"stream_get_contents", file::stream_get_contents);
    add(b"stream_copy_to_stream", file::stream_copy_to_stream);
    add(b"ftruncate", file::ftruncate);
    add(b"getenv", file::getenv);
    add(b"putenv", file::putenv);
    add(b"disk_free_space", file::disk_free_space);
    add(b"diskfreespace", file::disk_free_space); // legacy alias
    add(b"disk_total_space", file::disk_total_space);
    // Filesystem predicates / operations (step 52).
    add(b"basename", file::basename);
    add(b"dirname", file::dirname);
    add(b"pathinfo", file::pathinfo);
    add(b"file_exists", file::file_exists);
    add(b"is_file", file::is_file);
    add(b"is_dir", file::is_dir);
    add(b"is_link", file::is_link);
    add(b"is_readable", file::is_readable);
    add(b"is_writable", file::is_writable);
    add(b"is_writeable", file::is_writable);
    add(b"is_executable", file::is_executable);
    add(b"filetype", file::filetype);
    add(b"realpath", file::realpath);
    add(b"getcwd", file::getcwd);
    add(b"chdir", file::chdir);
    add(b"sys_get_temp_dir", file::sys_get_temp_dir);
    add(b"clearstatcache", file::clearstatcache);
    add(b"stat", file::stat);
    add(b"lstat", file::lstat);
    add(b"fstat", file::fstat);
    add(b"filesize", file::filesize);
    add(b"filemtime", file::filemtime);
    add(b"fileatime", file::fileatime);
    add(b"filectime", file::filectime);
    add(b"fileperms", file::fileperms);
    add(b"fileinode", file::fileinode);
    add(b"fileowner", file::fileowner);
    add(b"filegroup", file::filegroup);
    add(b"unlink", file::unlink);
    add(b"mkdir", file::mkdir);
    add(b"rmdir", file::rmdir);
    add(b"rename", file::rename);
    add(b"copy", file::copy);
    add(b"touch", file::touch);
    add(b"symlink", file::symlink);
    add(b"link", file::link);
    add(b"readlink", file::readlink);
    add(b"chmod", file::chmod);
    add(b"scandir", file::scandir);
    add(b"glob", file::glob);
    add(b"tempnam", file::tempnam);
    add(b"get_resource_type", file::get_resource_type);
    add(b"readdir", file::readdir);
    add(b"closedir", file::closedir);
    add(b"rewinddir", file::rewinddir);
    add(b"fprintf", file::fprintf);
    add(b"vfprintf", file::vfprintf);
    add(b"array_keys", array::array_keys);
    add(b"array_values", array::array_values);
    add(b"in_array", array::in_array);
    add(b"array_merge", array::array_merge);
    add(b"array_replace", array::array_replace);
    add(b"array_replace_recursive", array::array_replace_recursive);
    add(b"range", array::range);
    add(b"array_slice", array::array_slice);
    add(b"array_reverse", array::array_reverse);
    add(b"array_unique", array::array_unique);
    add(b"array_sum", array::array_sum);
    add(b"array_key_exists", array::array_key_exists);
    add(b"key_exists", array::array_key_exists);
    add(b"array_search", array::array_search);
    add(b"array_fill", array::array_fill);
    add(b"array_fill_keys", array::array_fill_keys);
    add(b"array_chunk", array::array_chunk);
    add(b"array_merge_recursive", array::array_merge_recursive);
    add(b"hash_algos", encoding::hash_algos);
    add(b"stream_get_wrappers", encoding::stream_get_wrappers);
    // ext/zlib string (de)compression (zlib-rs backend, byte-identical).
    add(b"gzdeflate", zlib::gzdeflate);
    add(b"gzinflate", zlib::gzinflate);
    add(b"gzcompress", zlib::gzcompress);
    add(b"gzuncompress", zlib::gzuncompress);
    add(b"gzencode", zlib::gzencode);
    add(b"gzdecode", zlib::gzdecode);
    add(b"zlib_encode", zlib::zlib_encode);
    add(b"zlib_decode", zlib::zlib_decode);
    add(b"zlib_get_coding_type", zlib::zlib_get_coding_type);
    add(b"array_flip", array::array_flip);
    add(b"array_change_key_case", array::array_change_key_case);
    add(b"array_count_values", array::array_count_values);
    add(b"array_combine", array::array_combine);
    add(b"array_pad", array::array_pad);
    add(b"array_product", array::array_product);
    add(b"array_is_list", array::array_is_list);
    add(b"array_key_first", array::array_key_first);
    add(b"array_key_last", array::array_key_last);
    add(b"array_first", array::array_first);
    add(b"array_last", array::array_last);
    add(b"array_diff", array::array_diff);
    add(b"array_intersect", array::array_intersect);
    add(b"array_diff_key", array::array_diff_key);
    add(b"array_intersect_key", array::array_intersect_key);
    add(b"array_diff_assoc", array::array_diff_assoc);
    add(b"array_intersect_assoc", array::array_intersect_assoc);
    add(b"array_column", array::array_column);
    add(b"array_rand", array::array_rand);
    add(b"implode", string::implode);
    add(b"join", string::implode);
    add(b"explode", string::explode);
    add(b"strcmp", string::strcmp);
    add(b"strncmp", string::strncmp);
    add(b"strcasecmp", string::strcasecmp);
    add(b"strncasecmp", string::strncasecmp);
    add(b"strnatcmp", string::strnatcmp);
    add(b"strnatcasecmp", string::strnatcasecmp);
    add(b"ctype_alnum", ctype::ctype_alnum);
    add(b"ctype_alpha", ctype::ctype_alpha);
    add(b"ctype_cntrl", ctype::ctype_cntrl);
    add(b"ctype_digit", ctype::ctype_digit);
    add(b"ctype_lower", ctype::ctype_lower);
    add(b"ctype_graph", ctype::ctype_graph);
    add(b"ctype_print", ctype::ctype_print);
    add(b"ctype_punct", ctype::ctype_punct);
    add(b"ctype_space", ctype::ctype_space);
    add(b"ctype_upper", ctype::ctype_upper);
    add(b"ctype_xdigit", ctype::ctype_xdigit);
    add(b"substr", string::substr);
    add(b"strpos", string::strpos);
    add(b"strrpos", string::strrpos);
    add(b"stripos", string::stripos);
    add(b"strripos", string::strripos);
    add(b"strspn", string::strspn);
    add(b"strcspn", string::strcspn);
    add(b"strtr", string::strtr);
    add(b"chunk_split", string::chunk_split);
    add(b"strip_tags", string::strip_tags);
    add(b"quotemeta", string::quotemeta);
    add(b"soundex", string::soundex);
    add(b"metaphone", string::metaphone);
    add(b"convert_uuencode", encoding::convert_uuencode);
    add(b"convert_uudecode", encoding::convert_uudecode);
    add(b"levenshtein", string::levenshtein);
    add(b"strstr", string::strstr);
    add(b"strchr", string::strstr);
    add(b"stristr", string::stristr);
    add(b"strrchr", string::strrchr);
    add(b"bin2hex", string::bin2hex);
    add(b"hex2bin", string::hex2bin);
    add(b"random_bytes", string::random_bytes);
    add(b"addcslashes", string::addcslashes);
    add(b"addslashes", string::addslashes);
    add(b"stripslashes", string::stripslashes);
    add(b"stripcslashes", string::stripcslashes);
    add(b"substr_replace", string::substr_replace);
    add(b"nl2br", string::nl2br);
    add(b"wordwrap", string::wordwrap);
    add(b"htmlspecialchars", html::htmlspecialchars);
    add(b"htmlspecialchars_decode", html::htmlspecialchars_decode);
    add(b"htmlentities", html::htmlentities);
    add(b"html_entity_decode", html::html_entity_decode);
    add(b"str_getcsv", csv::str_getcsv);
    add(b"fgetcsv", file::fgetcsv);
    add(b"fputcsv", file::fputcsv);
    add(b"str_replace", string::str_replace);
    add(b"str_ireplace", string::str_ireplace);
    add(b"strtoupper", string::strtoupper);
    add(b"strtolower", string::strtolower);
    add(b"ucfirst", string::ucfirst);
    add(b"lcfirst", string::lcfirst);
    add(b"ucwords", string::ucwords);
    add(b"str_repeat", string::str_repeat);
    add(b"str_pad", string::str_pad);
    add(b"chr", string::chr);
    add(b"ord", string::ord);
    add(b"trim", string::trim);
    add(b"ltrim", string::ltrim);
    add(b"rtrim", string::rtrim);
    add(b"strrev", string::strrev);
    add(b"str_rot13", string::str_rot13);
    add(b"str_contains", string::str_contains);
    add(b"str_starts_with", string::str_starts_with);
    add(b"str_ends_with", string::str_ends_with);
    add(b"str_split", string::str_split);
    add(b"str_shuffle", string::str_shuffle);
    add(b"substr_count", string::substr_count);
    add(b"substr_compare", string::substr_compare);
    add(b"str_increment", string::str_increment);
    add(b"str_decrement", string::str_decrement);
    add(b"count_chars", string::count_chars);
    add(b"str_word_count", string::str_word_count);
    add(b"mb_strlen", mbstring::mb_strlen);
    add(b"mb_substr", mbstring::mb_substr);
    add(b"mb_str_split", mbstring::mb_str_split);
    add(b"mb_strtoupper", mbstring::mb_strtoupper);
    add(b"mb_strtolower", mbstring::mb_strtolower);
    add(b"mb_convert_case", mbstring::mb_convert_case);
    add(b"mb_ucfirst", mbstring::mb_ucfirst);
    add(b"mb_lcfirst", mbstring::mb_lcfirst);
    add(b"mb_strpos", mbstring::mb_strpos);
    add(b"mb_stripos", mbstring::mb_stripos);
    add(b"mb_strrpos", mbstring::mb_strrpos);
    add(b"mb_strripos", mbstring::mb_strripos);
    add(b"mb_strstr", mbstring::mb_strstr);
    add(b"mb_stristr", mbstring::mb_stristr);
    add(b"mb_strrchr", mbstring::mb_strrchr);
    add(b"mb_strrichr", mbstring::mb_strrichr);
    add(b"mb_substr_count", mbstring::mb_substr_count);
    add(b"mb_ord", mbstring::mb_ord);
    add(b"mb_chr", mbstring::mb_chr);
    add(b"mb_str_pad", mbstring::mb_str_pad);
    add(b"mb_trim", mbstring::mb_trim);
    add(b"mb_ltrim", mbstring::mb_ltrim);
    add(b"mb_rtrim", mbstring::mb_rtrim);
    add(b"mb_check_encoding", mbstring::mb_check_encoding);
    add(b"mb_strwidth", mbstring::mb_strwidth);
    add(b"mb_strimwidth", mbstring::mb_strimwidth);
    add(b"mb_strcut", mbstring::mb_strcut);
    add(b"mb_convert_encoding", mbstring::mb_convert_encoding);
    add(b"iconv", mbstring::iconv);
    add(b"mb_detect_encoding", mbstring::mb_detect_encoding);
    add(b"mb_internal_encoding", mbstring::mb_internal_encoding);
    add(b"mb_encode_numericentity", mbstring::mb_encode_numericentity);
    add(b"mb_decode_numericentity", mbstring::mb_decode_numericentity);
    add(b"grapheme_strlen", grapheme::grapheme_strlen);
    add(b"grapheme_substr", grapheme::grapheme_substr);
    add(b"grapheme_str_split", grapheme::grapheme_str_split);
    add(b"grapheme_strpos", grapheme::grapheme_strpos);
    add(b"grapheme_stripos", grapheme::grapheme_stripos);
    add(b"grapheme_strrpos", grapheme::grapheme_strrpos);
    add(b"grapheme_strripos", grapheme::grapheme_strripos);
    add(b"grapheme_strstr", grapheme::grapheme_strstr);
    add(b"grapheme_stristr", grapheme::grapheme_stristr);
    add(b"grapheme_levenshtein", grapheme::grapheme_levenshtein);
    add(b"number_format", string::number_format);
    add(b"version_compare", string::version_compare);
    add(b"openssl_x509_parse", openssl::openssl_x509_parse);
    add(b"parse_url", url::parse_url);
    add(b"urlencode", url::urlencode);
    add(b"urldecode", url::urldecode);
    add(b"rawurlencode", url::rawurlencode);
    add(b"rawurldecode", url::rawurldecode);
    add(b"http_build_query", url::http_build_query);
    add(b"sprintf", format::sprintf);
    add(b"printf", format::printf);
    add(b"vsprintf", format::vsprintf);
    add(b"vprintf", format::vprintf);
    add(b"abs", math::abs);
    add(b"sin", math::sin);
    add(b"cos", math::cos);
    add(b"tan", math::tan);
    add(b"asin", math::asin);
    add(b"acos", math::acos);
    add(b"atan", math::atan);
    add(b"atan2", math::atan2);
    add(b"sinh", math::sinh);
    add(b"cosh", math::cosh);
    add(b"tanh", math::tanh);
    add(b"asinh", math::asinh);
    add(b"acosh", math::acosh);
    add(b"atanh", math::atanh);
    add(b"exp", math::exp);
    add(b"expm1", math::exp_m1);
    add(b"log", math::log);
    add(b"log10", math::log10);
    add(b"log1p", math::ln_1p);
    add(b"hypot", math::hypot);
    add(b"fmod", math::fmod);
    add(b"deg2rad", math::deg2rad);
    add(b"rad2deg", math::rad2deg);
    add(b"pi", math::pi);
    add(b"is_nan", math::is_nan);
    add(b"is_finite", math::is_finite);
    add(b"is_infinite", math::is_infinite);
    add(b"mt_rand", math::mt_rand);
    add(b"rand", math::mt_rand);
    add(b"mt_srand", math::mt_srand);
    add(b"srand", math::mt_srand);
    add(b"mt_getrandmax", math::mt_getrandmax);
    add(b"getrandmax", math::mt_getrandmax);
    add(b"gethostname", env::gethostname);
    add(b"sys_getloadavg", env::sys_getloadavg);
    add(b"php_uname", env::php_uname);
    add(b"usleep", env::usleep);
    add(b"sleep", env::sleep);
    add(b"uniqid", env::uniqid);
    add(b"max", math::max);
    add(b"min", math::min);
    add(b"intdiv", math::intdiv);
    add(b"dechex", math::dechex);
    add(b"base_convert", math::base_convert);
    add(b"decoct", math::decoct);
    add(b"decbin", math::decbin);
    add(b"hexdec", math::hexdec);
    add(b"octdec", math::octdec);
    add(b"bindec", math::bindec);
    add(b"fdiv", math::fdiv);
    add(b"pow", math::pow);
    add(b"sqrt", math::sqrt);
    add(b"floor", math::floor);
    add(b"ceil", math::ceil);
    add(b"round", math::round);
    add(b"var_dump", var_dump);
    add(b"var_export", var_export);
    add(b"serialize", serialize::serialize);
    add(b"strlen", strlen);
    add(b"gettype", gettype);
    add(b"is_int", is_int);
    add(b"is_integer", is_int);
    add(b"is_long", is_int);
    add(b"is_float", is_float);
    add(b"is_double", is_float);
    add(b"is_string", is_string);
    add(b"is_bool", is_bool);
    add(b"is_null", is_null);
    add(b"is_array", is_array);
    add(b"is_object", is_object);
    add(b"is_resource", is_resource);
    add(b"is_scalar", is_scalar);
    add(b"is_numeric", is_numeric);
    add(b"intval", intval);
    add(b"floatval", floatval);
    add(b"doubleval", floatval);
    add(b"strval", strval);
    add(b"setlocale", setlocale);
    add(b"extension_loaded", extension_loaded);
    add(b"phpversion", phpversion);
    add(b"get_loaded_extensions", get_loaded_extensions);
    add(b"boolval", boolval);
    add(b"filter_var", filter_var);
    add(b"filter_var_array", var::filter_var_array);
    add(b"localeconv", var::localeconv);
    add(b"print_r", print_r);
    // Environment / runtime-introspection stubs (no real engine state modelled).
    add(b"gc_collect_cycles", env::gc_collect_cycles);
    add(b"gc_enable", env::gc_enable);
    add(b"gc_disable", env::gc_enable);
    add(b"gc_enabled", env::gc_enabled);
    add(b"gc_mem_caches", env::gc_mem_caches);
    add(b"gc_status", env::gc_status);
    add(b"getmypid", env::getmypid);
    add(b"memory_get_usage", env::memory_get_usage);
    add(b"memory_get_peak_usage", env::memory_get_usage);
    add(b"memory_reset_peak_usage", env::memory_reset_peak_usage);
    add(b"php_sapi_name", env::php_sapi_name);
    add(b"ini_get", env::ini_get);
    add(b"ini_set", env::ini_set);
    add(b"ini_restore", env::ini_restore);
    add(b"posix_geteuid", env::posix_geteuid);
    add(b"php_ini_loaded_file", env::php_ini_loaded_file);
    add(b"php_ini_scanned_files", env::php_ini_scanned_files);
    add(b"phpinfo", env::phpinfo);
    // By-reference builtins (step 11c): their first argument binds the caller's
    // variable cell (D-R7).
    let mut add_ref = |name: &[u8], f: php_runtime::BuiltinRefFn| {
        r.insert(name.to_vec(), Builtin::RefFirst(f));
    };
    add_ref(b"array_push", array::array_push);
    add_ref(b"settype", array::settype);
    add_ref(b"sort", array::sort);
    add_ref(b"shuffle", array::shuffle);
    add_ref(b"rsort", array::rsort);
    add_ref(b"asort", array::asort);
    add_ref(b"natsort", array::natsort);
    add_ref(b"natcasesort", array::natcasesort);
    add_ref(b"arsort", array::arsort);
    add_ref(b"ksort", array::ksort);
    add_ref(b"krsort", array::krsort);
    add_ref(b"array_pop", array::array_pop);
    add_ref(b"array_shift", array::array_shift);
    add_ref(b"array_unshift", array::array_unshift);
    add_ref(b"array_splice", array::array_splice);
    r
}

/// First positional argument, or an `ArgumentCountError`-style fatal.
fn arg1<'a>(args: &'a [Zval], fname: &str) -> Result<&'a Zval, PhpError> {
    args.first()
        .ok_or_else(|| PhpError::Error(format!("{fname}() expects exactly 1 argument, 0 given")))
}

// --- output ---












// --- string / type inspection ---



// --- type predicates ---












// --- value casts ---






/// The extensions phpr substantially models, reported by `extension_loaded` and
/// `phpversion` so polyfill/feature guards take the same branch as the oracle:
/// Core, standard, SPL, pcre, json, mbstring, hash, date, openssl. Names lowercase.
/// openssl: phpr models TLS via the rustls-backed http/https stream wrapper (the
/// openssl/Composer-network filone). curl: the *easy* API only (curl.rs facade
/// over the same ureq transport); curl_multi_* stays undefined, so a dual-backend
/// consumer that probes for it (Composer's HttpDownloader::isCurlEnabled) still
/// takes the stream-wrapper path, while extension_loaded('curl') consumers
/// (monolog's handlers, Guzzle's sync CurlHandler) get the easy surface.
const LOADED_EXTENSIONS: &[&[u8]] = &[
    b"core", b"standard", b"spl", b"pcre", b"json", b"mbstring", b"hash", b"date", b"openssl",
    b"zip", b"dom", b"libxml", b"reflection", b"ctype", b"curl", b"pcntl", b"posix",
    b"pdo", b"pdo_sqlite", b"sqlite3", b"simplexml", b"bcmath", b"gmp",
    // Declared for PHPUnit's bootstrap gate; their heavy surfaces (token_get_all,
    // xml_parser_*, XMLWriter) are filled in test-driven — a use ahead of the
    // implementation surfaces as an honest "undefined function".
    b"xml", b"xmlwriter", b"tokenizer", b"phar",
];

/// The same list with PHP's canonical casing, as `get_loaded_extensions()`
/// reports it (the check side is case-insensitive, the listing is not).
const LOADED_EXTENSIONS_CASED: &[&[u8]] = &[
    b"Core", b"standard", b"SPL", b"pcre", b"json", b"mbstring", b"hash", b"date", b"openssl",
    b"zip", b"dom", b"libxml", b"Reflection", b"ctype", b"curl", b"pcntl", b"posix",
    b"PDO", b"pdo_sqlite", b"sqlite3", b"bcmath", b"gmp",
    b"xml", b"xmlwriter", b"tokenizer", b"Phar",
];





