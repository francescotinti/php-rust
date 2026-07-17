// (segmento del prelude — concatenato via include_str! in lower/mod.rs;
//  NIENTE <?php qui: il tag di apertura vive solo in core.php)
// ext/session handler surface (C2 of the session port). SessionHandler
// delegates to the built-in files module via the __session_files_op host
// hook, so `extends SessionHandler` + parent::read()/write() keeps working
// (Symfony's StrictSessionHandler does exactly that).
interface SessionHandlerInterface {
    public function open(string $path, string $name): bool;
    public function close(): bool;
    public function read(string $id): string|false;
    public function write(string $id, string $data): bool;
    public function destroy(string $id): bool;
    public function gc(int $max_lifetime): int|false;
}
interface SessionIdInterface {
    public function create_sid(): string;
}
interface SessionUpdateTimestampHandlerInterface {
    public function validateId(string $id): bool;
    public function updateTimestamp(string $id, string $data): bool;
}
class SessionHandler implements SessionHandlerInterface, SessionIdInterface {
    public function open(string $path, string $name): bool {
        return __session_files_op('open', $path, $name);
    }
    public function close(): bool {
        return __session_files_op('close');
    }
    public function read(string $id): string|false {
        return __session_files_op('read', $id);
    }
    public function write(string $id, string $data): bool {
        return __session_files_op('write', $id, $data);
    }
    public function destroy(string $id): bool {
        return __session_files_op('destroy', $id);
    }
    public function gc(int $max_lifetime): int|false {
        return __session_files_op('gc', $max_lifetime);
    }
    public function create_sid(): string {
        return __session_files_op('create_sid');
    }
}
