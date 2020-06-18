; On-page jump table.
;
; Because there isn't a reasonable mechanism to export addresses to the
; vector table on page 1B, those vectors point to this on-page jump table
; which has more predictable addresses.
;
; TODO support exporting addresses so we don't need this jump table
.org $4000
    jp cpHLDE   ; 4000
    jp PutMap   ; 4003
    jp PutC     ; 4006
    jp DispHL   ; 4009
    jp PutS     ; 400c
    jp ClrLCDFull   ; 400f
    jp HomeUp   ; 4012
    jp VPutMap  ; 4015
    jp VPutS    ; 4018
    jp GrBufCpu ; 401b
    jp MemSet   ; 401e
    jp GetCSC   ; 4021

#include "ti83plus.inc"
#include "tihle-os.inc"

cpHLDE:
    push hl
    or a
    sbc hl,de
    pop hl
    ret

; Implementation provided by the system routines reference.
PutS:
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
VPutS:
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
GetCSC:
    ld hl, kbdScanCode
    di                          ; Atomically read and clear scan code
    ld a, (hl)
    ld (hl), 0
    res kbdSCR, (iy+kbdFlags)   ; Consumed scan code, none ready
    ei
    ret

PutMap: trap _PutMap \ ret
PutC: trap _PutC \ ret
DispHL: trap _DispHL \ ret
ClrLCDFull: trap _ClrLCDFull \ ret
HomeUp: trap _HomeUP \ ret
VPutMap: trap _VPutMap \ ret
GrBufCpu: trap _GrBufCpy \ ret
MemSet: trap _MemSet \ ret
