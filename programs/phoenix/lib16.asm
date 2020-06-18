;##################################################################
;
;   Phoenix-Z80 (low-level support routines)
;
;   Programmed by Patrick Davidson (pad@calc.org)
;        
;   Copyright 2001 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last updated June 11, 2001.
;
;##################################################################   

;############## Synchronization

synchronize:
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
        in      a,(3)
        bit     1,a
        jr      z,int_exit
        ld      a,(in_game)
        or      a
        jr      nz,timer_swapping
        ld      a,$7c
        out     (0),a
        ld      a,(timer)
        inc     a
        ld      (timer),a
int_exit:
        pop     af
        ret

timer_swapping:
        push    hl
        ld      hl,timer
        inc     (hl)
        ld      a,(speed)
        cp      (hl)
        jr      nz,ie2

which_page:
        ld      a,0
        cpl
        ld      (which_page+1),a
        or      a
        jr      z,main_screen

smc_alloc_page:
        ld      a,$a
        out     (0),a
        ld      hl,$fc00
        ld      (gfx_target),hl
        pop     hl
        pop     af
        ret

main_screen:
        ld      a,$7c
        out     (0),a
smc_alloc_start:
        ld      hl,$ca00
        ld      (gfx_target),hl
ie2:    pop     hl
        pop     af
        ret
timer_interrupt_end:

;############## Contrast adjustment

SUPER_GET_KEY:
        call    GET_KEY
        cp      K_PLUS
        jr      z,contrast_up
        cp      K_MINUS
        jr      z,contrast_down
        ret

contrast_up:
        ld      a,(CONTRAST)
        cp      $1f
        ret     z
        inc     a
        ld      (CONTRAST),a
        out     ($2),a
        xor     a
        ret

contrast_down:
        ld      a,(CONTRAST)
        or      a
        ret     z
        dec     a
        ld      (CONTRAST),a
        out     ($2),a
        xor     a
        ret
