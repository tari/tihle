; Hello world program
;
; Minimal test for text output.

.nolist
#include "ti83plus.inc"
.list

.org userMem - 2
.db t2ByteTok, tAsmCmp

    bcall(_ClrLCDFull)
    bcall(_HomeUp)
    ld hl, message
    bcall(_Puts)
    ret

message: .db "Hello, world!", 0
