;##################################################################
;
;   P H O E N I X         F O R        T I - 8 5   /  T I - 8 6
;
;   Programmed by Patrick Davidson (pad@calc.org)
;        
;   Copyright 2008 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last updated September 9, 2008.
;
;##################################################################     

sidesdata       =$FA70
leftsidecoord   =$FB70
leftsidevel     =$FB71
rightsidecoord  =$FB72
rightsidevel    =$FB73

;############## Initialize synchronization, screen sides, external levels

main:   ld      (iy+13),0               ; No-scroll text mode

        ld      hl,$fc00
        ld      (gfx_target),hl

        xor     a
        ld      (in_game),a

        ld      (initsp+1),sp           ; Save SP for exit

        call    set_up_sides            ; Set up scrolling side data

        ld      hl,timer_interrupt      ; Initialize interrupt
        call    INT_INSTALL

        ld      hl,level_table          ; Set default level data
        ld      (level_addr),hl

        call    restore_game            ; Check for saved game

        xor     a                       ; no ext. level (detected later)
        ld      (extlevel),a

no_saved_game:
        ld      hl,$fc00                ; Set graphics frame
        ld      (smc_gfxmem_start+1),hl

        ld      a,4                     ; Set speed to normal (for title)
        ld      (speed),a

        call    check_level_loaded      ; Check for external level

        call    title_screen

;############## Set up new game

prepare_new_game:
        ld      hl,player_y
        ld      (hl),70                 ; Player Y coord = 70
        inc     hl
        ld      (hl),60                 ; Player X coord = 60
        inc     hl
        inc     hl
        inc     hl
        ld      (hl),16                 ; Status of player's ship

        ld      hl,19000
        ld      (time_score),hl
        ld      a,4
        ld      (money_counter),a

pre_main_loop:
        call    convert_settings        ; decode configuration
        call    set_invert
        ld      a,1
        ld      (in_game),a
        ld      hl,-6
        add     hl,sp
        ld      (collision_done+1),hl
        call    display_screen

;############## Game main loop
        
main_loop:
        call    frame_init

        call    hit_enemies            ; Collisions btw. bullets & enemies

        ld      a,(enemies_left)
        or      a
        jr      nz,no_load_level
        call    load_level
        ld      hl,in_game
        ld      a,(hl)
        or      a
        jr      nz,no_load_level
        inc     a
        ld      (timer),a
        ld      (hl),a
no_load_level:

        call    scroll_sides           ; Scroll sides (inside side buffer)
        call    clear_buffer           ; Prepare main display buffer

        call    init_rand

        call    do_player              ; Move and draw player
        call    do_companion           ; Move and draw companion ship

        call    enemies                ; Move and draw enemies

        call    player_bullets         ; Move and draw player bullets

        call    enemy_bullets          ; Move and draw enemy bullets
        call    hit_player             ; Collisions involving player

        call    prepare_indicator      ; Prepare shield indicator

        call    display_money
        call    synchronize
        call    display_screen         ; Synchronize and swap buffers

        call    handle_input           ; Process control keys
        jr      main_loop

game_exit:
        call    INT_REMOVE
        call    set_normal
        ld      a,$7c
        out     (0),a
initsp: ld      sp,0
        ret
