;##################################################################
;
;   Phoenix for TI-83 Plus - MirageOS version
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
#define __MIRAGE__
#define TI83P

#include "ti83plus.inc"
#include "mirage.inc"
#include "keys.i"

backup_storage  =SavesScreen
#define TEXT_MEM SavesScreen+256
GFX_AREA        =plotsscreen   
GRAPH_MEM       =plotsscreen

D_ZT_STR        =_puts
D_HL_DECI       =_disphl
TX_CHARPUT      =_putc
M_CHARPUT	=_vputmap
CLEARLCD        =_ClrLCDFull
CURSOR_ROW      =currow
CURSOR_COL      =curcol
ionDetect       =idetect
UNPACK_HL       =_divhlby10
CURSOR_X	=pencol
CURSOR_Y	=penrow
D_ZM_STR	=_vputs

#define ROM_CALL(kewl_routinez) bcall(kewl_routinez)

        .org    $9D93
        .db     $BB,$6D
        ret
        .db     1
        .db     %00000000,%01001000
        .db     %00011000,%10010010
        .db     %00011000,%00000000
        .db     %00011000,%00000000
        .db     %00011000,%10000100
        .db     %00011000,%01000010
        .db     %00000000,%00000000
        .db     %00000000,%00010000
        .db     %10000001,%00100100
        .db     %10000001,%00000000
        .db     %10000001,%00000000
        .db     %10011001,%00000000
        .db     %10100101,%00000000
        .db     %11000011,%00000000
        .db     %10000001,%00000000

#include "phoenixz.i"

        .db     "Phoenix ",PVERS,0

start:  di
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
