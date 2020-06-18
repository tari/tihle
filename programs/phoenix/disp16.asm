;##################################################################
;
;   Phoenix-Z80 (Screen display routines)
;
;   Programmed by Patrick Davidson (pad@calc.org)
;        
;   Copyright 2005 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last update November 28, 2005.
;
;##################################################################   

;############## Show information panel

display_money:
        ld      de,(smc_gfxmem_start+1)
        ld      hl,$3b0
        ld      a,(player_x)
        cp      60
        jr      nc,left_side
        ld      hl,$3be
left_side:
        add     hl,de
        ex      de,hl
        ld      hl,decimal_cash
        jp      display_number_bcd

;############## Set text-drawing according to invert flag

set_invert:
        set     3,(iy+5)
        ld      a,(invert)
        or      a
        ret     nz
set_normal:
        res     3,(iy+5)
        ld      hl,rightsidetable
        ld      de,leftsidetable
        ret

;############## Clear screen in appropriate color

cls:    ld      hl,$fc00
        ld      a,(invert)
        ld      (hl),a
        ld      de,$fc01
        ld      bc,$3ff
        ldir
        ret

;############## Display -1 terminated list of strings at (HL)

display_hl_msgs:
        push    hl
        call    clear_buffer
        call    set_invert
        pop     hl

        ld      de,$0200
show_loop:
        ld      (CURSOR_ROW),de
        push    de
        call    D_ZT_STR
        pop     de
        ld      a,(hl)
        inc     a
        ret     z
        inc     e
        jr      show_loop

;############## Initialize side data

set_up_sides:
        ld      a,1
        ld      (leftsidevel),a
        ld      (rightsidecoord),a
        inc     a
        ld      (leftsidecoord),a
        ld      a,-1
        ld      (rightsidevel),a
        ld      b,64
loop_sus:
        push    bc
        call    scroll_sides
        pop     bc
        djnz    loop_sus
        ret

;############## Scroll the sides down one pixel 
        
scroll_sides:
        ld      de,sidesdata+$ff 
        ld      hl,sidesdata+$fb
        ld      bc,$fc
        lddr

        ld      hl,rightsidecoord
        call    scroll_side
        add     a,2

        ld      hl,rightsidetable
        bit     3,a
        jr      z,right_side_small
        sub     8
        call    ADD_HL_A
        ld      l,(hl)
        ld      h,$FC
        ld      (sidesdata+2),hl
        jr      scroll_do_left_side        
right_side_small:       
        call    ADD_HL_A
        ld      a,(hl)
        and     $FC
        ld      h,a
        ld      l,0
        ld      (sidesdata+2),hl

scroll_do_left_side:    
        ld      hl,leftsidecoord
        call    scroll_side

        ld      hl,leftsidetable
        bit     3,a
        jr      z,left_side_small
        sub     8
        call    ADD_HL_A
        ld      h,(hl)
        ld      l,$ff
        ld      (sidesdata),hl
        jr      scroll_final
        
left_side_small:  
        call    ADD_HL_A
        ld      l,(hl)
        ld      h,0
        ld      (sidesdata),hl

scroll_final:
        ld      a,(sides_flag)
        or      a
        jr      nz,sides_on
        ld      hl,sidesdata
        ld      (hl),0
        ld      bc,3
        call    OTH_FILL
sides_on:

        ld      a,(invert)
        or      a
        ret     z
        ld      hl,sidesdata
        ld      b,3
loop_reverse_scroll_top:
        ld      a,(hl)
        cpl
        ld      (hl),a
        inc     hl
        djnz    loop_reverse_scroll_top
        ld      a,(hl)
        xor     $FC
        ld      (hl),a
        ret

;############## Calculate new position of side at (HL) 
        
scroll_start:
        ld      (hl),1
        call    FAST_RANDOM
        add     a,a
        jr      c,scroll_adjust_done
        ld      (hl),-1
        jr      scroll_adjust_done        
        
scroll_side:
        call    FAST_RANDOM
        and     7
        jr      nz,nosiderand
        inc     hl
        ld      a,(hl)
        or      a
        jr      z,scroll_start
        ld      (hl),0
scroll_adjust_done:
        dec     hl
nosiderand:     
        ld      a,(hl)
        inc     hl
        add     a,(hl)
        cp      2
        jr      nc,noforceli
        ld      (hl),1
noforceli:
        cp      12
        jr      nz,noforceld
        ld      (hl),-1
noforceld:      
        dec     hl
        ld      (hl),a
        ret

rightsidetable:
        .db     %00000000
        .db     %00000001
        .db     %00000011
        .db     %00000111
        .db     %00001111
        .db     %00011111
        .db     %00111111
        .db     %01111111
        .db     %11111111

leftsidetable:
        .db     %00000000
        .db     %10000000
        .db     %11000000
        .db     %11100000
        .db     %11110000
        .db     %11111000
        .db     %11111100
        .db     %11111110
        .db     %11111111

;############## Copy sides only to the LCD
                      
display_sides:
        call    frame_init
        call    init_rand
        ld      a,$3c
        out     (0),a
        ld      hl,sidesdata
        ld      de,$fc00
        ld      b,64

display_sides_loop:
        push    bc
        ld      a,(hl)
        ld      (de),a
        inc     hl
        inc     de   

        ld      a,(hl)
        and     $f0
        ld      b,a
        ld      a,(de)
        and     $f
        or      b
        ld      (de),a

        ld      a,13
        add     a,e
        ld      e,a
        inc     hl
        
        ld      a,(hl)
        and     $3f
        ld      b,a
        ld      a,(de)
        and     $c0
        or      b
        ld      (de),a
        inc     hl
        inc     de

        ld      a,(hl)
        ld      (de),a
        inc     hl
        inc     de

        pop     bc
        djnz    display_sides_loop
        ret

;############## Clears screen buffer (draw sides only)

clear_buffer:
        ld      hl,sidesdata
        ld      de,(smc_gfxmem_start+1)
        ld      b,65
        ld      a,(invert)
loop_clear:     
        ldi
        ldi

        ld      (de),a
        inc     de
        ld      (de),a
        inc     de
        ld      (de),a
        inc     de
        ld      (de),a
        inc     de
        ld      (de),a
        inc     de
        ld      (de),a
        inc     de
        ld      (de),a
        inc     de
        ld      (de),a
        inc     de
        ld      (de),a
        inc     de
        ld      (de),a
        inc     de
        ld      (de),a
        inc     de
        ld      (de),a
        inc     de
                
        ldi
        ldi

        djnz    loop_clear
        
        ret

;############## Prepare shield indicator

prepare_indicator:
        ld      hl,(smc_gfxmem_start+1)
prepare_indicator_hl:
        ld      de,1023
        add     hl,de

        ld      a,(player_pwr)
        add     a,a
        ret     z
        add     a,a
        cp      65
        jr      c,nlhs
        ld      a,64
nlhs:   ld      de,-16
        ld      b,a
loop_ind:
        set     0,(hl)
        add     hl,de
        djnz    loop_ind
        ret

;############## Display entire screen from buffer

display_screen:
        ld      hl,(gfx_target)
        ld      (smc_gfxmem_start+1),hl
#ifdef __TI85__
        ld      (smc_test_1+1),hl
        ld      (smc_test_2+1),hl
        ld      (smc_test_3+1),hl
#endif
        ld      de,-512
        add     hl,de
        ld      (smc_gfxmem_minus512+1),hl
        ret
