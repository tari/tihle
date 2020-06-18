;##################################################################
;
;   Phoenix-Z80 (New game initialization)
;
;   Programmed by Patrick Davidson (pad@ocf.berkeley.edu)
;        
;   Copyright 2015 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last updated May 6, 2015.
;
;##################################################################     

;############## Main title screen

title_screen:
        ld      hl,data_zero_start
        ld      bc,data_zero_end-data_zero_start-1
        call    OTH_CLEAR
        
redraw_title:
        ld      hl,title_main
	xor	a

show_title:
	ld	(title_loop+1),a
        call    display_hl_msgs
	call	Read_LCD
        
title_loop:
	jr	title_skip
	call	render_sides_short
title_skip:
        call    SUPER_GET_KEY
        cp      KEY_CODE_5
        jr      z,redraw_title
        cp      KEY_CODE_4
        jr      z,show_instructions
        cp      KEY_CODE_3
        jr      z,show_contact
        cp      KEY_CODE_2
        jr      z,_options_screen
        cp      KEY_CODE_1
        ret     z
        cp      KEY_CODE_ALPHA
        jr      z,show_highs
        cp      KEY_CODE_MODE
        jr      z,title_exit
        cp      KEY_CODE_CLEAR
        jr      z,title_exit
        cp      KEY_CODE_DEL
        jr      nz,title_loop
title_exit:
        jp      game_exit

show_highs:
        call    no_high_score
        jr      redraw_title

show_contact:
        ld      hl,title_contact
	ld	a,3
        jr      show_title

show_instructions:
        ld      hl,title_instructions
	ld	a,3
        jr      show_title

;############## Options screen

_options_screen:                         ; initialize options screen
        xor     a
        ld      (option_item),a

option_redraw:                          ; redraw options screen
        call    convert_settings
        ld      hl,options_msg
        call    display_hl_msgs

option_draw:                            ; draw option arrow
        call    option_position
        ld      hl,draw_pointer
        ROM_CALL(D_ZT_STR)

options_loop:                           ; option main loop
        call    SUPER_GET_KEY
        cp      KEY_CODE_5
        jr      z,redraw_title
        cp      KEY_CODE_MODE
        jr      z,redraw_title
        cp      KEY_CODE_CLEAR
        jr      z,redraw_title
        cp      KEY_CODE_DEL
        jr      z,redraw_title
        cp      KEY_CODE_UP
        jr      z,options_up
        cp      KEY_CODE_DOWN
        jr      z,options_down
        cp      KEY_CODE_SECOND
        jr      nz,options_loop

        ld      a,(option_item)         ; dispatch chosen optoin
        add     a,a
        ld      (smc_optionjump+1),a

smc_optionjump:
        jr      option_jumptable
option_jumptable:
        jr      option_skill
        jr      option_terrain
        jr      option_speed
        jr      option_side
        jr      option_scroll
        jp      redraw_title

options_up:                             ; move arrow up
        call    option_erase
        ld      hl,option_item
        dec     (hl)
        jp      p,option_draw
        ld      (hl),5
        jr      option_draw

options_down:                           ; move arrow down
        call    option_erase
        ld      hl,option_item
        inc     (hl)
        ld      a,6
        cp      (hl)
        jr      nz,option_draw
        ld      (hl),0
        jr      option_draw

option_erase:                           ; erase arrow
        call    option_position
        ld      hl,erase_pointer
        ROM_CALL(D_ZT_STR)

option_position:                        ; calculate arrow position
        ld      a,(option_item)
        add     a,2
        ld      l,a
        ld      h,0
        ld      (CURSOR_ROW),hl
        ret

erase_pointer:
        .db     "  ",0
draw_pointer:
        .db     "->",0

option_skill:                           ; change skill
        ld      hl,difficulty
        ld      c,3
option_common:
        ld      a,(hl)
        inc     a
        cp      c
        jr      c,difficulty_ok
        xor     a
difficulty_ok:
        ld      (hl),a
goto_option_redraw:
        jp      option_redraw
        
option_speed:                           ; change speed
        ld      hl,speed_option
        ld      c,2
        jr      option_common

option_terrain:                         ; change color
        ld      hl,invert
        ld      a,(hl)
        cpl
        ld      (hl),a
        jr      goto_option_redraw

option_side:
        ld      hl,sides_flag
        ld      a,1
        xor     (hl)
        ld      (hl),a
        jr      goto_option_redraw

option_scroll:
        ld      hl,scroll_flag
        ld      a,1
        xor     (hl)
        ld      (hl),a
        jr      goto_option_redraw

;############## Load temporary variables from settings

convert_settings:
        ld      a,(difficulty)
        ld      b,a
        add     a,a
        add     a,b             ; A = difficulty * 3
        add     a,a
        add     a,a             ; A = difficulty * 12
        sub     b               ; A = difficulty * 11
        ld      hl,difficulty_data
        call    ADD_HL_A
        ld      de,level_end-7
        ld      bc,6
        ldir

        ld      de,money_amount
        ld      bc,5
        ldir

        ld      a,(speed_option)
        ld      b,a
        add     a,a
        add     a,b
        add     a,a
        add     a,b             ; A = speed * 6
        ld      hl,speed_data
        call    ADD_HL_A
        ld      de,speed_end-5
        ld      bc,4
        ldir

        ld      a,(hl)
        ld      (speed),a
        inc     hl
        call    DO_LD_HL_MHL
        ld      de,(bonus_score)
        add     hl,de
        ld      (bonus_score),hl

        ld      a,(invert)
        inc     a               ; 0 = black, 1 = white
        ld      b,a            
        add     a,a
        add     a,a
        add     a,b             ; 0 = black, 5 = white
        ld      hl,terrain_data
        call    ADD_HL_A
        ld      de,terrain_end-6
        ld      bc,5
        ldir

        ld      a,(sides_flag)
        ld      b,a
        add     a,a
        add     a,b
        ld      hl,sides_data
        call    ADD_HL_A
        ld      de,sides_end-4
        ld      bc,3
        ldir

        ld      a,(scroll_flag)
        ld      b,a
        add     a,a
        add     a,b
        ld      hl,sides_data
        call    ADD_HL_A
        ld      de,scroll_end-4
        ld      bc,3
        ldir
        ret

sides_data:
        .db     "OFF"
        .db     "ON "

terrain_data:
        .db     "BLACK"
        .db     "WHITE"
     
speed_data:
        .db     "SLOW",3      ; slow (delay 6, bonus 0)
        .dw     0
        .db     "FAST",2      ; fast (delay 4, bonus 5000)
        .dw     5000

difficulty_data:
        .db     "EASY  ",100    ; easy (cash 100, bonus 0)
        .dw     0
        .db     $1,$00
        .db     "MEDIUM",50     ; medium (cash 50, bonus 5000)
        .dw     5000
        .db     $0,$50
        .db     "HARD  ",25     ; hard (cash 25, bonus 15000)
        .dw     15000
        .db     $0,$25
#ifdef  ENABLE_CHEATS
        .db     "CHEAT ",100    ; hard (cash 100, bonus 0)
        .dw     0
        .db     $1,$00
#endif

;############## Title screen messages

title_main:
        .db     "  Phoenix  ",PVERS,0
        .db     -1
	.db	12,16,"by Patrick Davidson",0
	.db     31,23," 1 - Start Game",0
        .db     39,27,"  2 - Settings",0
        .db     47,22,"3 - Contact Info",0
        .db     55,22,"4 - Instructions",0
	.db	0

title_instructions:
        .db     "Arrows Move Ship",0
        .db     "#s Select Weapon",0
        .db     "2nd Fires Weapon",0
        .db     "MODE Saves&Exits",0
        .db     "ENTER Pauses",0
        .db     "+,- Contrast Adj",0
        .db     0
        .db     "5: Main Menu",0
        .db     -1
	.db	0

title_contact:
	.db     "  Phoenix  ",PVERS,0
	.db	-1
	.db	15,9,"eeulplek@hotmail.com",0
	.db	24,1,"www.ocf.berkeley.edu/pad",0
	.db	32,5,"IRC: PatrickD on EfNet #ti",0
	.db	40,0,"eeulplek on twitter,Youtube",0
	.db	56,25,"5 - Main Menu",0
	.db	0

options_msg:
        .db     "Use up/down/2nd",0
        .db     0         
        .db     "   Level ......",0
level_end:
        .db     "   Backgr .....",0
terrain_end:
        .db     "   Speed ....",0
speed_end:
        .db     "   Sides ...",0 
sides_end:
        .db     "   Scrolling ...",0
scroll_end:
        .db     "   Exit options",0
        .db     -1
	.db	0