.nolist
#include "ti83plus.inc"
#include "mirage.inc"
.list

.db $bb, $6d
.org userMem

    bcall(_HomeUp)
    bcall(_ClrLCDFull)
    ld hl, message
    bcall(_PutS)

    ; Select all key groups
    xor a
    out (1), a
    ; Wait until no keys are pressed
waitNoKeys:
    in a, (1)
    inc a
    jr nz, waitNoKeys

    ; Service only timer interrupts
    im 1
    ld a, $02
    out (3), a

    ld hl, 0
loop:
    ei
    halt
    ; Quit if any key pressed
    in a, (1)
    inc a
    jr nz, exit
    ; Increment and display counter
    inc hl
    push hl
    ld de, $0B03
    ld (curRow), de
    bcall(_DispHL)
    pop hl
    jr loop

exit:
    ld a, $0B
    out (3), a
    ret

message: .db "Press any key to"
         .db "exit.", 0
