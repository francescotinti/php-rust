<?php
chdir('..');
var_dump(get_current_user() != "");
chdir('..');
var_dump(getmyinode() !== false);
var_dump(getlastmod() != false);