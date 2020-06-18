;##################################################################
;
;   Phoenix-Z80 (low-level support routines)
;
;   Programmed by Patrick Davidson (pad@ocf.berkeley.edu)
;        
;   Copyright 2015 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last updated April 17, 2015.
;
;##################################################################   

;############## Simple routines

DO_LD_HL_MHL:
        ld      a,(hl)
        inc     hl
        ld      h,(hl)
        ld      l,a
        ret

DO_CP_HL_DE:
        push    hl
        and     a
        sbc     hl,de
        pop     hl
        ret

;############## Synchronization

synchronize:
        ei
        ld      hl,timer
        ld      a,(speed)
        cp      (hl)
        jr      c,too_slow

loop_wait:
        cp      (hl)            ; Test value of 4 - (timer)
        jr      nc,loop_wait    ; NC : timer <= 4
        ld      (hl),0

        ret

too_slow:
        ld      (hl),0
        ret

timer_interrupt:
        push    af
        push    hl
        ld      hl,timer
        inc     (hl)
        pop     hl
        pop     af
#ifdef __MIRAGE__
        ret
#else
        jp      $38
#endif
timer_interrupt_end:

;############## Contrast adjustment

#ifdef __TI82__
CONTRAST_MAX    =$21
CONTRAST_ADJ    =$1E+$c0
#else
CONTRAST_MAX    =$27
CONTRAST_ADJ    =$18+$c0
#endif

SUPER_GET_KEY:
        call    GET_KEY
        cp      KEY_CODE_PLUS
        jr      z,contrast_up
        cp      KEY_CODE_MINUS
        jr      z,contrast_down
        ret

contrast_up:
        ld      a,(CONTRAST)
        cp      CONTRAST_MAX
        ret     z
        inc     a
        ld      (CONTRAST),a
        add     a,CONTRAST_ADJ
        out     ($10),a
        ret

contrast_down:
        ld      a,(CONTRAST)
        or      a
        ret     z
        dec     a
        ld      (CONTRAST),a
        add     a,CONTRAST_ADJ
        out     ($10),a
        ret

;############## Read LCD to 13x64 space at (HL)

Read_LCD:
	ld	hl,GFX_AREA+2
        ld      c,$20
readColumn: 
        DWAIT
        ld      a,c
        out     ($10),a
        cp      $2c
        ret     z
        DWAIT
        ld      a,$80
        out     ($10),a
        DWAIT
        in      a,($11)
        ld      b,64
        ld      de,16
readByte:
        DWAIT
        in      a,($11)
        ld      (hl),a
        add     hl,de
        djnz    readByte
        ld      de,1-(16*64)
        add     hl,de
        inc     c
        jr      readColumn
