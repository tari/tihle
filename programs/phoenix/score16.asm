;##################################################################
;
;   Phoenix-Z80 (High scores)
;
;   Programmed by Patrick Davidson (pad@calc.org)
;        
;   Copyright 2007 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last updated August 10, 2007.
;
;##################################################################     

scoring_msg:
        .db     "Shields",0
        .db     "Speed",0
        .db     "Bonus",0
        .db     "Money",0
        .db     0
        .db     "Score",0
        .db     0
        .db     "  Press [ENTER]",0
        .db     -1

highscore_title:
        .db     "   High Scores",0

highscore_prompt:
        .db     " Enter your name",0

;############## Calculate and display player's score

game_finished:
        xor     a
        ld      (in_game),a

        ld      hl,completed
        ld      a,(hl)
        or      a
        jp      nz,no_high_score
        ld      (hl),1

        call    cls
        ld      hl,scoring_msg
        call    display_hl_msgs

        ld      hl,$0E03
        ld      (CURSOR_ROW),hl
        ld      hl,(player_cash)        ; max 10000
        call    D_HL_DECI

        ld      hl,$0E02
        ld      (CURSOR_ROW),hl
        ld      hl,(bonus_score)        ; max 20000
        call    D_HL_DECI

        ld      hl,$0E01
        ld      (CURSOR_ROW),hl
        ld      hl,(time_score)         ; max 19000
        call    D_HL_DECI

        ld      a,(player_pwr)          ; max 16000
        ld      hl,0
        ld      de,1000
        or      a
        jr      z,shield_done
        ld      b,a
shield_addup:
        add     hl,de
        djnz    shield_addup
shield_done:
        push    hl
        ld      de,(time_score)
        add     hl,de
        ld      de,(bonus_score)
        add     hl,de
        ld      de,(player_cash)
        add     hl,de
        ld      (bonus_score),hl

        ld      de,$0E05
        ld      (CURSOR_ROW),de
        call    D_HL_DECI

        ld      hl,$0E00
        ld      (CURSOR_ROW),hl
        pop     hl
        call    D_HL_DECI

loop_show_score:
        call    scroll_sides
        call    synchronize
        call    display_sides
        
        call    GET_KEY
        cp      K_ENTER
        jr      nz,loop_show_score

        ld      a,(extlevel)
        or      a
        jp      nz,restart

;############## Check if you have high score

        ld      hl,(high_scores+89)              ; Check against lowest score
        ld      de,(bonus_score)
        call    DO_CP_HL_DE
        jr      nc,no_high_score

        ld      (high_scores+89),de              ; Put your score in bottom
        ld      hl,high_scores+(13*6)
        ld      b,10
loop_space:
        ld      (hl),' '
        inc     hl
        djnz    loop_space
        
        ld      b,6                             ; Bubble it towards the top
        ld      de,high_scores+(13*7)-2         ; DE -> entry to move up
loop_bubble:                             
        ld      hl,-13
        add     hl,de                           ; HL -> entry to compare with

        push    de

        call    DO_LD_HL_MHL                       ; HL = score above this one
        push    hl
        ex      de,hl
        ld      e,(hl)
        inc     hl
        ld      d,(hl)                          ; DE = this score
        pop     hl
        call    DO_CP_HL_DE
        pop     de                              ; DE -> this entry
        jr      nc,no_bubble_up

        inc     de                              ; DE -> very end of entry
        ld      hl,-13
        add     hl,de                           ; HL -> previous entry
        push    bc
        ld      b,13
loop_exchange:
        ld      a,(de)
        ld      c,a
        ld      a,(hl)
        ld      (de),a
        ld      (hl),c
        dec     hl
        dec     de
        djnz    loop_exchange
        pop     bc
        dec     de                              ; DE -> previous entry

        djnz    loop_bubble

no_bubble_up:
        ld      hl,-11
        add     hl,de

        push    hl
        push    bc
        call    display_high_scores
        ld      hl,$0200
        ld      (CURSOR_ROW),hl
        ld      hl,highscore_prompt
        call    D_ZT_STR
        pop     bc
        pop     hl
        inc     b
        ld      c,b
        ld      b,2
        ld      (CURSOR_ROW),bc
        call    input_name

;############## Show high scores

no_high_score:
        xor     a
        ld      (in_game),a

        call    display_high_scores
        ld      hl,$0200
        ld      (CURSOR_ROW),hl
        ld      hl,highscore_title
        call    D_ZT_STR
        call    loop_show_highs
        jp      restart

loop_show_highs:
        call    scroll_sides
        call    synchronize
        call    display_sides
        
        call    GET_KEY
        cp      6
        jr      c,loop_show_highs
        ret

;############## Prompt for name entry

input_name:
        push    hl
        pop     ix
        ld      b,0
enter_name_loop:
        push    bc
        call    scroll_sides
        call    synchronize
        call    display_sides
        pop     bc

        call    GET_KEY
        or      a
        jr      z,enter_name_loop
        cp      K_DEL
        jr      z,backup
        cp      K_ENTER
        ret     z
        ld      c,a
        ld      a,10
        cp      b
        jr      z,enter_name_loop
        ld      hl,chartable-10
        ld      e,c
        ld      d,0
        add     hl,de
        ld      a,(hl)
        ld      (ix),a
        call    TX_CHARPUT 
        inc     b
        inc     ix
        jr      enter_name_loop
backup: xor     a
        cp      b
        jr      z,enter_name_loop
        dec     b
        dec     ix
        ld      (ix),32
        ld      hl,CURSOR_COL
        dec     (hl)
        ld      a,32
        call    TX_CHARPUT
        dec     (hl)
        jr      enter_name_loop

chartable:
        .db     "XTOJE."
        .db     ". WSNID!.ZVRMHC?"
        .db     ".YUQLGB#x~+PKFA|"
        .db     "@54321.~+"

;############## Display the high score table

display_high_scores:
        call    cls
        call    GET_KEY

        ld      hl,high_scores
        ld      b,7
        xor     a
        ld      (CURSOR_ROW),a
loop_display_hs:
        push    hl
        ld      hl,CURSOR_ROW
        inc     (hl)
        inc     hl
        ld      (hl),2
        pop     hl
        push    hl
        call    D_ZT_STR
        pop     hl
        ld      de,11
        add     hl,de
        push    hl
        call    DO_LD_HL_MHL
        ld      a,$e
        ld      (CURSOR_COL),a
        call    D_HL_DECI
        pop     hl
        inc     hl
        inc     hl
        djnz    loop_display_hs
        ret


