;##################################################################
;
;   P H O E N I X         F O R        T I - 8 3      ( I o n )
;
;   Programmed by Patrick Davidson (pad@ocf.berkeley.edu)
;        
;   Copyright 2015 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last updated April 16, 2015.
;
;##################################################################     

#define __TI83__
#define TI83

#include "ion.inc"
#include "keys.i"

#define TEXT_MEM saferam2+200

#define ROM_CALL(kewl_routinez) bcall(kewl_routinez)

        .org    progstart

interrupt_entry =$8282
interrupt_byte  =$82
interrupt_table =$8300
interrupt_reg   =$83
backup_storage  =$8401

GFX_AREA        =plotsscreen

    ret
    jr  nc,start

D_ZT_STR        =_puts
D_HL_DECI       =_disphl
TX_CHARPUT      =_putc
M_CHARPUT	=_vputmap
CLEARLCD        =_clrlcdf
CURSOR_ROW      =currow
CURSOR_COL      =curcol
UNPACK_HL       =_divhlby10
CURSOR_X	=pencol
CURSOR_Y	=penrow
D_ZM_STR	=_vputs

#include "phoenixz.i"

        .db     "Phoenix ", PVERS,0

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
#include "images.asm"
#include "info.asm"
#include "data.asm"
#include "levels.asm"
#include "vars.asm"

        .end
