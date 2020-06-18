;##################################################################
;
;   Phoenix for TI-82 (Ace)
;
;   Programmed by Patrick Davidson (pad@calc.org)
;        
;   Copyright 2015 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last updated April 11, 2015.
;
;##################################################################     

#define __TI82__

#include    "acebeta.inc"

GFX_AREA    =$9000

interrupt_entry =$9494
interrupt_byte  =$94
interrupt_table =$9500
interrupt_reg   =$95

;backup_storage  =$8401

#include "phoenixz.i"
#include "keys.i"

        .db     "Phoenix ", PVERS,0

        ld      a,($8D0F)               ; Prevent running from MirageOS
        or      a
        ret     z
      ;  ld      a,(INT_STATE)           ; Make sure no TSR installed
      ;  or      a
      ;  jp      z,main
;
;interrupt_installed:
;        ROM_CALL(CLEARLCD)
;        ld      (CURSOR_ROW),de
;        ld      hl,int_err
;        ROM_CALL(TX_CHARPUT)
;        jp      CR_KHAND
;
;int_err:
;        .db     "ERROR:  You must"
;        .db     "remove installed"
;        .db     "interrupts.",0

#include "main12.asm"

D_HL_DECI:
        push    hl
        ld      hl,(CURSOR_ROW)
        push    hl
        ld      hl,spaces
        ROM_CALL(D_ZT_STR)
        pop     hl
        ld      (CURSOR_ROW),hl
        pop     hl
        call    HL_DECI
        ROM_CALL(D_ZT_STR)
        ret

spaces: .db     "     ",0

CLEARLCD        =CLEAR_LCD

;############## Include remainder of game files

#include "extlev12.asm"
#include "exchange.asm"
#include "disp12.asm"
#include "lib12.asm"
#include "lib.asm"
#include "title12.asm"
#include "drwspr.asm"
#include "player12.asm"
#include "shoot.asm"
#include "bullets.asm"
#include "enemies.asm"
#include "init.asm"
#include "enemyhit.asm"
#include "collide.asm"
#include "ebullets.asm"
#include "hityou.asm"
#include "shop12.asm"
#include "helper.asm"
#include "eshoot.asm"
#include "score12.asm"
#include "emove.asm"
#include "info.asm"
#include "images.asm"
#include "data.asm"
#include "levels.asm"
#include "vars.asm"

program_end:
