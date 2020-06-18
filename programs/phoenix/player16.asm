;##################################################################
;
;   Phoenix-Z80 (Player handling)
;
;   Programmed by Patrick Davidson (pad@ocf.berkeley.edu)
;        
;   Copyright 2011 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last updated June 2, 2011.
;
;##################################################################     

;############## Movement, Firing (by OTH_ARROW) and ship display

do_player:
        call    OTH_ARROW
        ld      c,a

        ld      hl,fire_counter
        bit     5,c
        jr      z,player_fire
        ld      (hl),0
        jr      fire_done

player_fire:
        ld      d,4
        ld      a,(weapon_upgrade)
        or      a
        jr      nz,autofire
        ld      d,10
autofire:
        ld      a,(hl)
        or      a
        jr      z,do_shoot
        dec     (hl)
        jr      fire_done
do_shoot:
        ld      (hl),d
        push    bc
        call    player_shoot
        pop     bc
fire_done:

        ld      hl,player_y
        rr      c     
        jr      c,no_down       
        ld      a,(hl)  
        add     a,1     
        cp      90
        jr      z,no_down       
        ld      (hl),a  
no_down:
        inc     hl      
        rr      c       
        jr      c,no_left       
        ld      a,(hl)  
        add     a,-2
        cp      12
        jr      z,no_left       
        ld      (hl),a  
no_left:
        rr      c       
        jr      c,no_right      
        ld      a,(hl)  
        add     a,2
        cp      108     
        jr      z,no_right      
        ld      (hl),a  
no_right:
        ld      d,(hl)  
        dec     hl      
        rr      c       
        jr      c,no_up 
        ld      a,(hl)  
        add     a,-1     
        cp      68            
        jr      z,no_up 
        ld      (hl),a  
no_up:                            
        ld      de,(player_y)
        ld      hl,img_player_ship_normal
        ld      a,(player_pwr)
        cp      4
        jp      nc,drw_spr
        ld      hl,img_player_ship_damaged
        jp      drw_spr

;############## Control keys (GET_KEY)

pause:  ld      hl,$fc00
        ld      de,(smc_alloc_start+1)
        ld      bc,$400
        ld      a,(which_page+1)
        jr      z,hl_shown_page
        ex      de,hl
hl_shown_page:
        ldir

        ld      hl,$0304
        ld      (CURSOR_ROW),hl
        ld      hl,pause_msg
        call    D_ZT_STR

        ld      hl,$fc00
        ld      de,(smc_alloc_start+1)
        ld      bc,$280
        ldir

loop_pause:
        call    SUPER_GET_KEY
        cp      K_ENTER
        jr      nz,loop_pause
        ret

pause_msg:
        .db     "PAUSED [ENTER]",0

handle_input:
        call    SUPER_GET_KEY
        cp      K_ENTER
        jr      z,pause
        cp      K_MORE
        jp      z,game_save
        cp      K_EXIT
        jp      z,game_exit

        ld      hl,chosen_weapon
        cp      K_F5
        jr      z,select_weapon_5
        cp      K_F4
        jr      z,select_weapon_4
        cp      K_F3
        jr      z,select_weapon_3
        cp      K_F2
        jr      z,select_weapon_2
        sub     K_F1
        ret     nz
        ld      (hl),a
        ret

select_weapon_2:
        ld      a,(weapon_2)
        or      a
#ifndef ENABLE_CHEATS
        ret     z
        ld      (hl),a
#else
        ld      (hl),1
#endif
        ret

select_weapon_3:
        ld      a,(weapon_3)    
        or      a
#ifndef ENABLE_CHEATS
        ret     z
#endif
        ld      (hl),2
        ret

select_weapon_4:
        ld      a,(weapon_4)
        or      a
#ifndef ENABLE_CHEATS
        ret     z
#endif
        ld      (hl),3
        ret

select_weapon_5:
        ld      a,(weapon_5)
        or      a
#ifndef ENABLE_CHEATS
        ret     z
#endif
        ld      (hl),4
        ret
