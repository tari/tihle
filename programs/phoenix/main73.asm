;##################################################################
;
;   P H O E N I X       F O R      T I - 7 3     ( M a l l a r d )
;
;   Programmed by Patrick Davidson (pad@calc.org)
;        
;   Copyright 2005 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last updated November 28, 2005.
;
;##################################################################     

#define __TI73__
#define HORRIBLE_KEYBOARD

#include "mallard.inc"
#include "keys.i"

#define TEXT_MEM saferam2+200

#define ROM_CALL(kewl_routinez) bcall(kewl_routinez)

interrupt_entry =$8686
interrupt_byte  =$86
interrupt_table =$8700
interrupt_reg   =$87
backup_storage  =saferam2

GFX_AREA        =plotsscreen
GRAPH_MEM       =plotsscreen

D_ZT_STR        =_puts
D_HL_DECI       =_disphl
TX_CHARPUT      =_putc
CLEARLCD        =_ClrLCD
CURSOR_ROW      =currow
CURSOR_COL      =curcol
UNPACK_HL       =_divhlby10

#include "phoenixz.i"

        .org    userMem
        .db     $D9,$00,"Duck"
        .dw     start
        .db     "Phoenix ", PVERSION,0

start:
        ld      de,backup_storage       ; Save data after GRAPH_MEM
        ld      hl,GFX_AREA+768
        ld      bc,256
        ldir
        call    main
        ld      (iy+13),6
restore_memory:
        ld      hl,backup_storage      ; Restore data after GRAPH_MEM
        ld      de,GFX_AREA+768
        ld      bc,256
        ldir
        ret

GET_KEY:
        bcall(_getcsc)
        ret

;############## Include remainder of game files

#include "main12.asm"
#include "extlev12.asm"
#include "exchange.asm"
#include "lib12.asm"
#include "lib.asm"
#include "title12.asm"
#include "disp12.asm"
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
#include "images.asm"
#include "info.asm"
#include "data.asm"
#include "levels.asm"
#include "vars.asm"

        .end
