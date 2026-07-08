<?php
namespace Pdo;
class Sqlite extends \PDO {
    public const DETERMINISTIC = 2048;
    public const OPEN_READONLY = 1;
    public const OPEN_READWRITE = 2;
    public const OPEN_CREATE = 4;
    public const ATTR_OPEN_FLAGS = 1000;
    public const ATTR_READONLY_STATEMENT = 1001;
    public const ATTR_EXTENDED_RESULT_CODES = 1002;
    public const ATTR_BUSY_STATEMENT = 1003;
    public const ATTR_EXPLAIN_STATEMENT = 1004;
    public const ATTR_TRANSACTION_MODE = 1005;
    public const TRANSACTION_MODE_DEFERRED = 0;
    public const TRANSACTION_MODE_IMMEDIATE = 1;
    public const TRANSACTION_MODE_EXCLUSIVE = 2;
    public const EXPLAIN_MODE_PREPARED = 0;
    public const EXPLAIN_MODE_EXPLAIN = 1;
    public const EXPLAIN_MODE_EXPLAIN_QUERY_PLAN = 2;
    public const OK = 0;
    public const DENY = 1;
    public const IGNORE = 2;
}
