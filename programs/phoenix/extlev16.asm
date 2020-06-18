;##################################################################
;
;   Phoenix-Z80 (External level handling on TI-85 and TI-86)
;
;   Programmed by Patrick Davidson (pad@calc.org)
;        
;   Copyright 2007 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last updated August 10, 2007.
;
;##################################################################     

;############## Startup check for external level

check_level_loaded:
        call    verify_level
        ret     nz
load_external_level:
        xor     a                       ; If found, erase marker
        ld      (LEVEL_LOCATION),a
        inc     a                       ; Flag level use
        ld      (extlevel),a
        ld      hl,LEVEL_LOCATION+11    ; Use its level data address
        ld      (level_addr),hl         
        ld      de,level_name
        ld      bc,8
        ldir
        ld      (level_addr),hl         ; Adjust pointer to level info
        ret 

;############## Restore game saved in external level

extlevel_saved:
        call    verify_level
        jr      nz,load_error

        ld      hl,level_name
        ld      de,LEVEL_LOCATION+11
        ld      b,8
test_loop:
        ld      a,(de)
        cp      (hl)
        jr      nz,load_error
        inc     hl
        inc     de
        djnz    test_loop
        jr      load_external_level

;############## Display loading error message

load_error:
        ld      hl,0
        ld      (CURSOR_ROW),hl
        ld      hl,load_error_msg
        call    D_ZT_STR

error_loop:
        call    GET_KEY
        cp      K_CLEAR
        jp      z,game_exit
        cp      K_F1
        jr      nz,error_loop

        xor     a
        ld      (saved_flag),a
        ld      (extlevel),a
        ld      sp,(initsp+1)
        jp      no_saved_game

load_error_msg:
        .db     "ERROR:  There is a   "
        .db     "saved game from an   "
        .db     "external level.  To  "             
        .db     "restore the game, run"
        .db     "that level.  To start"
        .db     "a new game, press F1."
        .db     "Press CLEAR to exit.",0

;############## Check for presence of an external level

verify_level:
        ld      hl,(LEVEL_LOCATION)     ; Check for external level
        ld      de,31338
        call    DO_CP_HL_DE
        ret     nz

        ld      hl,LEVEL_LOCATION+2
        ld      a,VERS_BYTE
        cp      (hl)
        ret     c

        ld      de,test_string
        ld      b,8
loop_verify:
        dec     de
        inc     hl
        ld      a,(de)
        cp      (hl)
        ret     nz
        djnz    loop_verify
        ret

        .db     0,"xInEoHp"
test_string:
