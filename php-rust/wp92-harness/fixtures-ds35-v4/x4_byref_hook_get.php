<?php
class P {
    public int $x {
        &get { return $this->x; }
    }
}
class C extends P {
    public string $x {
        &get { return $this->x; }
    }
}
