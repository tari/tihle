;##################################################################
;
;   Phoenix-Z80 (Shop)
;
;   Programmed by Patrick Davidson (pad@calc.org)
;        
;   Copyright 2005 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last updated November 28, 2005.
;
;##################################################################     

;############## Test for money and intialize shop screen

shop:   ld      hl,(player_cash)
        ld      a,h
        or      l
        ret     z

        xor     a
        ld      (in_game),a

        call    cls
        call    display_sides
        ld      hl,0
        ld      (CURSOR_ROW),hl  
        ld      hl,shopmessage
        call    D_ZT_STR

        ld      a,1
        ld      (shop_item),a

;############## Shop main loop

shop_loop:
        ld      hl,$1000
        ld      (CURSOR_ROW),hl
        ld      hl,(player_cash)
        ld      a,l
        or      h
        jr      z,end_of_shop_z
        call    D_HL_DECI

        ld      hl,$ffff
        ld      de,-16
        ld      b,64
loop_down:
        res     0,(hl)
        add     hl,de
        djnz    loop_down
        inc     hl
        call    prepare_indicator_hl

        call    SUPER_GET_KEY
        ld      hl,shop_item
        cp      K_DOWN
        jr      z,shop_down
        cp      K_UP
        jr      z,shop_up
        cp      K_EXIT
end_of_shop_z:
        jp      z,convert_to_decimal
        cp      K_ENTER
        call    z,shop_select
        jr      shop_loop

;############## Shop cursor movement

shop_down:
        ld      a,(hl)
        cp      7
        jr      z,shop_loop

        push    hl
        ld      l,a
        ld      h,0
        ld      (CURSOR_ROW),hl
        ld      a,' '
        call    TX_CHARPUT
        pop     hl

        inc     (hl)
        ld      a,(hl)
        ld      l,a
        ld      h,0
        ld      (CURSOR_ROW),hl
        ld      a,'>'
        call    TX_CHARPUT
        jr      shop_loop

shop_up:
        ld      a,(hl)
        cp      1
        jp      z,shop_loop

        push    hl
        ld      l,a
        ld      h,0
        ld      (CURSOR_ROW),hl
        ld      a,' '
        call    TX_CHARPUT
        pop     hl

        dec     (hl)
        ld      a,(hl)
        ld      l,a
        ld      h,0
        ld      (CURSOR_ROW),hl
        ld      a,'>'
        call    TX_CHARPUT
        jp      shop_loop

;############## Shop item purchases

item3:  ld      bc,500
        sbc     hl,bc
        ret     c

        ld      a,(companion_pwr)
        cp      16
        ret     z
        ld      (player_cash),hl
get_companion:
        ld      hl,companion_pwr
        ld      (hl),16
        inc     hl
        ld      (hl),90
        inc     hl
        ld      (hl),60
        ret

shop_select:
        ld      a,(hl)
        dec     a
        add     a,a
        ld      (shop_jump_offset+1),a
        ld      hl,(player_cash)
shop_jump_offset:
        jr      item_list
item_list:
        jr      item1
        jr      item2
        jr      item3
        jr      item4
        jr      item5
        jr      item6

item7:  ld      bc,2000
        ld      a,4
        jr      common_weapon_add

item1:  ld      bc,100
        sbc     hl,bc
        ret     c
        ld      a,(player_pwr)
        cp      16
        ret     z
        inc     a
        ld      (player_pwr),a
        ld      (player_cash),hl
        ret

item2:  ld      bc,300                  ; BC = weapon cost
        ld      a,1                     ; A = weapon # - 1
common_weapon_add:
        sbc     hl,bc                   ; HL = money left after purcahse
        ret     c                       ; if negative, can't purchase
        ex      de,hl                   ; DE = money left after purchase
        ld      (chosen_weapon),a       ; set chosen weapon to this one
        ld      hl,weapon_2-1
        call    ADD_HL_A                ; HL -> weapon purchase flag
        ld      a,(hl)
        or      a
        ret     nz                      ; exit if already purchased
        inc     (hl)                    ; flag as purchased
        ld      (player_cash),de        ; set cash to new value
        ret

item5:  ld      bc,1000
        ld      a,2
        jr      common_weapon_add

item6:  ld      bc,1250
        ld      a,3
        jr      common_weapon_add


item4:  ld      de,weapon_upgrade
        ld      a,(de)
        or      a
        ret     nz
        ld      bc,750
        sbc     hl,bc
        ret     c
        ld      (player_cash),hl
        inc     a
        ld      (de),a
        ret

;############## Shop messages

shopmessage:
        .db     "Phoenix Shop - $     "
        .db     "> Extra shield   $100"
        .db     "  Weapon (F2)    $300"
        .db     "  Companion      $500"
        .db     "  Weapon Plus    $750"
        .db     "  Weapon (F3)   $1000"
        .db     "  Weapon (F4)   $1250"
        .db     "  Weapon (F5)   $2000",0
