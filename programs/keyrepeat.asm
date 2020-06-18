; GetCSC/key repeat test utility.
;
; Displays a '.' every time a key is returned from GetCSC, which can be used
; to investigate key repeat.

.nolist
#include "ti83plus.inc"
.list

.org $9d93
.db $bb, $6d

loop:
    bcall(_GetCSC)
    or a
    jr z, loop
    cp skClear
    ret z
    ld a, '.'
    bcall(_PutC)
    jr loop
