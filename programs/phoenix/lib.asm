;##################################################################
;
;   Phoenix-Z80 (generic low-level support routines)
;
;   Programmed by Patrick Davidson (pad@calc.org)
;        
;   Copyright 2002 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last updated June 26, 2002.
;
;##################################################################   

;############## Basic computations

table_look_up:
        add     a,a
        ld      hl,speed_table
ADD_HL_A:
        add     a,l
        ld      l,a
        ret     nc
        inc     h
        ret     

;############## Frame-averaging division by 16
;                                                
; Divides the value in A by 16.  This routine uses the timer to decide
; whether fractions are rounded up, so a fractional part of x/16 is rounded
; up x frame out of every 16.  This allows movement of objects by fractional
; amounts to appear smooth.  Changes A, B, and C.

Div_A_16:
        ld      bc,(game_timer)
        ld      b,a
        xor     a
        rr      c
        rla
        rr      c
        rla
        rr      c
        rla
        rr      c
        rla
        add     a,b
        sra     a
        sra     a
        sra     a
        sra     a
        ret

;############## Frame initialize / random numbers

frame_init:
        ld      hl,(game_timer)                 ; count frame
        inc     hl
        ld      (game_timer),hl

        bit     0,l                             ; count down score
        ret     z
        ld      hl,(time_score)
        ld      a,h
        or      l
        ret     z
        dec     hl
        ld      (time_score),hl
        ret 

init_rand:
        ld      hl,(game_timer)                 ; seed random numbers
        ld      a,(player_x)
        rlca
        xor     l
        xor     h
        rlca
        rlca
        rlca
        ld      e,a
        ld      d,0
        ld      hl,img_enemy_4
        add     hl,de
        ld      (FAST_RANDOM+2),hl
        ret
        

FAST_RANDOM:
        push    hl
        ld      hl,0
        ld      a,(hl)
        inc     hl
        rrca
        add     a,(hl)
        inc     hl
        rrca
        xor     (hl)
        inc     hl
        ld      (FAST_RANDOM+2),hl
        pop     hl
        ret
