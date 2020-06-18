;##################################################################
;
;   Phoenix-Z80 (New game initialization)
;
;   Programmed by Patrick Davidson (pad@ocf.berkeley.edu)
;        
;   Copyright 2015 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last updated April 28, 2015.
;
;##################################################################     

;############## Main title screen

title_screen:
        ld      hl,data_zero_start
        ld      bc,data_zero_end-data_zero_start-1
        call      OTH_CLEAR
        
        call    convert_settings
redraw_title:
	xor	a
	ld	(title_main_later),a
        ld      hl,title_main

show_title:
        call    display_hl_msgs
        
title_loop:
        call    scroll_sides
        call    display_sides

        call    synchronize
        call    SUPER_GET_KEY
        sub     K_F5
        jr      z,redraw_title
        dec     a
        jr      z,show_instructions
        dec     a    
        jp      z,show_contact
        dec     a
        jr      z,options_screen
        dec     a
        ret     z
        cp      K_ALPHA-K_F1
        jr      z,show_highs
        cp      K_EXIT-K_F1
        jr      nz,title_loop
        jp      game_exit

show_highs:
        call    no_high_score
        jr      redraw_title

show_instructions:
        ld      hl,title_instructions
        jr      show_title

;############## Options screen

options_screen:                         ; initialize options screen
        xor     a
        ld      (option_item),a

option_redraw:                          ; redraw options screen
        call    convert_settings
        ld      hl,options_msg
        call    display_hl_msgs

option_draw:                            ; draw option arrow
        call    option_position
        ld      hl,draw_pointer
        call    D_ZT_STR

options_loop:                           ; option main loop
        call    scroll_sides
        call    display_sides
        call    synchronize
        call    SUPER_GET_KEY
        cp      K_F5
        jr      z,redraw_title
        cp      K_EXIT
        jr      z,redraw_title
        cp      K_UP
        jr      z,options_up
        cp      K_DOWN
        jr      z,options_down
        cp      K_ENTER
        jr      z,item_selected
        cp      K_SECOND
        jr      nz,options_loop

item_selected:
        ld      a,(option_item)         ; dispatch chosen option
        add     a,a
        ld      (smc_optionjump+1),a

smc_optionjump:
        jr      option_jumptable
option_jumptable:
        jr      option_skill
        jr      option_color
        jr      option_speed
        jr      option_side
        jp     	redraw_title

options_up:                             ; move arrow up
        call    option_erase
        ld      hl,option_item
        dec     (hl)
        jp      p,option_draw
        ld      (hl),4
        jr      option_draw

options_down:                           ; move arrow down
        call    option_erase
        ld      hl,option_item
        inc     (hl)
        ld      a,5
        cp      (hl)
        jr      nz,option_draw
        ld      (hl),0
        jr      option_draw

option_erase:                           ; erase arrow
        call    option_position
        ld      hl,erase_pointer
        jp      D_ZT_STR

option_position:                        ; calculate arrow position
        ld      a,(option_item)
        add     a,2
        ld      l,a
        ld      h,2
        ld      (CURSOR_ROW),hl
        ret

erase_pointer:
        .db     "  ",0
draw_pointer:
        .db     "->",0

option_skill:                           ; change skill
        ld      hl,difficulty
option_common:
        ld      a,(hl)
        inc     a
        cp      3
        jr      c,difficulty_ok
        xor     a
difficulty_ok:
        ld      (hl),a
goto_option_redraw:
        jp      option_redraw
        
option_speed:                           ; change speed
        ld      hl,speed_option
        jr      option_common

option_color:                         ; change color
        ld      hl,invert
        ld      a,(hl)
        cpl
        ld      (hl),a

        ld      c,64
        ld      hl,sidesdata
revo:   ld      b,3
revi:   ld      a,(hl)
        cpl
        ld      (hl),a
        inc     hl
        djnz    revi
        ld      a,(hl)
        cpl
        and     $fc
        ld      (hl),a
        inc     hl
        dec     c
        jr      nz,revo

        jr      goto_option_redraw

option_side:
        ld      hl,sides_flag
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
        add     a,a
        add     a,a
        add     a,b             ; A = speed * 8
        ld      hl,speed_data
        call    ADD_HL_A
        ld      de,speed_end-7
        ld      bc,6
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
        ret

sides_data:
        .db     "OFF"
        .db     "ON "

terrain_data:
        .db     "BLACK"
        .db     "WHITE"
     
speed_data:
        .db     "SLOW  ",6      ; slow (delay 6, bonus 0)
        .dw     0
        .db     "MEDIUM",5      ; medium (delay 5, bonus 1000)
        .dw     1000
        .db     "FAST  ",4      ; fast (delay 4, bonus 5000)
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

;############## Show contact addresses

show_contact:
	ld	a,-1
	ld	(title_main_later),a
	ld	hl,title_main
	call	display_hl_msgs
	ld      hl,$207
	ld	(CURSOR_ROW),hl
	ld	hl,title_to_main
	call	D_ZT_STR
	
	ld	de,$1918
	ld	(CURSOR_X),de
	call	D_ZM_STR
	ld	de,$2010
	ld	(CURSOR_X),de
	call	D_ZM_STR
	ld	de,$2716
	ld	(CURSOR_X),de
	call	D_ZM_STR
	ld	de,$2E0f
	ld	(CURSOR_X),de
	call	D_ZM_STR
	
        jp      title_loop
	
;############## Title screen messages

title_main:
        .db     "  Phoenix   ",PVERS,0
        .db     "  Programmed by",0
        .db     "Patrick  Davidson",0
title_main_later:
        .db     0
        .db     " F1 - Start Game",0
        .db     "  F2 - Settings",0
        .db     "F3 - Contact Info",0
        .db     "F4 - Instructions",0
        .db     -1

title_instructions:
        .db     "Arrows: Move Ship",0
        .db     "Fn: Select Weapon",0
        .db     "2nd: Shoot Weapon",0
        .db     "MORE: Save + Exit",0
        .db     "ENTER: Pause Game",0
        .db     "+,-: Contrast Adj",0
        .db     0
title_to_main:
        .db     "F5: Main Menu",0
        .db     -1

title_mail:	.db	"eeulplek@hotmail.com",0
title_web:	.db	"www.ocf.berkeley.edu/pad/",0
title_irc:	.db	"IRC:PatrickD on EfNet #ti",0
title_oth:	.db	"eeulplek on Twitter/YouTube",0

options_msg:
        .db     "Use up/down/2nd",0
        .db     0         
        .db     "   Level ......",0
level_end:
        .db     "   Backgr .....",0
terrain_end:
        .db     "   Speed ......",0
speed_end:
        .db     "   Sides ...",0
sides_end:
        .db     "   Exit settings",0
        .db     -1
