; MULTIPAGE:PAGE:01
.org $4000

#include "ti83plus.inc"
#include "tihle-os.inc"

cpHLDE:     ; MULTIPAGE:EXPORT:cpHLDE
    push hl
    or a
    sbc hl,de
    pop hl
    ret

; Implementation provided by the system routines reference.
PutS:       ; MULTIPAGE:EXPORT:PutS
    push bc
    push af
    ld b, 8     ; Modified from reference: we don't support split screen,
                ; so hard-code 8 rather than reading winBtm.
PutS_loop:
    ld a, (hl)
    inc hl
    or a
    scf
    jr z, PutS_done
    call PutC
    ld a, (curRow)
    cp b
    jr c, PutS_loop
PutS_done:
    pop bc
    ld a, b
    pop bc
    ret

; Implementation provided by the system routines reference.
VPutS:      ; MULTIPAGE:EXPORT:VPutS
    push af
    push de
    push ix
VPutS_loop:
    ld a, (hl)
    inc hl
    or a
    jr z, VPutS_done
    call VPutMap
    jr nc, VPutS_loop
VPutS_done:
    pop ix
    pop de
    pop af
    ret


; GetCSC depends on interrupt keyboard polling, so is pretty easy.
GetCSC:     ; MULTIPAGE:EXPORT:GetCSC
    ld hl, kbdScanCode
    di                          ; Atomically read and clear scan code
    ld a, (hl)
    ld (hl), 0
    res kbdSCR, (iy+kbdFlags)   ; Consumed scan code, none ready
    ei
    ret

PutMap: trap _PutMap \ ret          ; MULTIPAGE:EXPORT:PutMap
PutC: trap _PutC \ ret              ; MULTIPAGE:EXPORT:PutC
DispHL: trap _DispHL \ ret          ; MULTIPAGE:EXPORT:DispHL
ClrLCDFull: trap _ClrLCDFull \ ret  ; MULTIPAGE:EXPORT:ClrLCDFull
HomeUp: trap _HomeUP \ ret          ; MULTIPAGE:EXPORT:HomeUp
VPutMap: trap _VPutMap \ ret        ; MULTIPAGE:EXPORT:VPutMap
GrBufCpy: trap _GrBufCpy \ ret      ; MULTIPAGE:EXPORT:GrBufCpy
MemSet: trap _MemSet \ ret          ; MULTIPAGE:EXPORT:MemSet
DivHLBy10: trap _DivHLBy10 \ ret    ; MULTIPAGE:EXPORT:DivHLBy10
