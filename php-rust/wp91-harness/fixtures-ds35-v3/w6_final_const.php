<?php
class P { final const X = 1; }
class C extends P { const X = 2; }
echo "unreachable";
