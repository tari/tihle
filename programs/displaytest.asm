; A simple display test; writes a test pattern direct to the LCD.
;
; Exercises X+ autoaddressing mode, data access

.nolist
#include "ti83plus.inc"
.list

.org $9D93
.db $bb, $6d

    xor a
    ld hl, plotSScreen
    ld b, 3
displayFill:
    ld (hl), a
    inc a
    inc hl
    jr nz, displayFill
    djnz displayFill

    ld hl, plotSScreen
    ld c, $20
updateRow:
    ld a, c
    out ($10), a

writeColumn:
    ld a, $80
    out ($10), a
    ld b, 64
writeColumnLoop:
    ld a, (hl)
    inc hl
    out ($11), a
    djnz writeColumnLoop
    ; trap PRINT_CPU_STATE
    .db $ed, $25, $FF, $FF
    inc c
    ld a, c
    cp $2f
    jr nz, updateRow

exit:
    jr exit
    ret

