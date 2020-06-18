;##################################################################
;
;   P H O E N I X         F O R        T I - 8 5    ( U s g a r d )
;
;   Programmed by Patrick Davidson (pad@calc.org)
;        
;   Copyright 2011 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last updated July 30, 2011.
;
;##################################################################     

#define __TI85__
#define __85OR86__

#include "usgard.h"

DO_LD_HL_MHL =LD_HL_MHL
DO_CP_HL_DE =CP_HL_DE

#include "phoenixz.i"

GFX_AREA        =$FC00

        .org    $9800

        .db     "Phoenix ", PVERS, " by Patrick D",0
        
        ld      hl,(PROGRAM_ADDR)
        ld      de,memory_initialize-$9800
        add     hl,de
        ld      de,DELC_LEN
        ld      bc,memory_initialize_end-memory_initialize
        ldir
        jp      DELC_LEN

;############## MEMORY SWAPPING CODE (RUN FROM DELC_LEN)

memory_initialize:
        nop
        ld      hl,(PROGRAM_ADDR)
        ld      de,$9800
        ld      bc,end_of_code-$9800
        call    memory_exchange

        ld      de,$c400
        ld      hl,GRAPH_MEM
        ld      bc,1024
        call    memory_exchange

        call    main_program

        ld      de,$c400
        ld      hl,GRAPH_MEM
        ld      bc,1024
        call    memory_exchange

        ld      de,(PROGRAM_ADDR)
        ld      hl,$9800
        ld      bc,end_of_code-$9800

memory_exchange =$-memory_initialize+DELC_LEN

#include "exchange.asm"

memory_initialize_end:

;############## START OF NORMAL CODE (RUN WITH PROGRAM STARTING AT $9800)

main_program:
        set     0,(iy+3)
        ld      hl,(VAT_END)               ; Locate double-buffer page
        ld      de,-1024                    
        add     hl,de
        ld      l,0
        ld      de,(FIRST_FREE)
        call    CP_HL_DE
        jr      nc,allocok
        call    CLEARLCD
        ld      (CURSOR_ROW),de
        ld      hl,nomem
        call    D_ZT_STR
        jp      OTH_PAUSE

nomem:  .db     "ERROR: 1.25K FREE RAM"
        .db     "REQIURED!",0

allocok:
        ld      (smc_alloc_start+1),hl
        ld      a,h
        and     $7f
        ld      (smc_alloc_page+1),a

LEVEL_LOCATION  =$c400

;############## Include remainder of game files
        
#include "main16.asm"
#include "lib16.asm"
#include "lib.asm"
#include "title16.asm"
#include "disp16.asm"
#include "drwspr.asm"
#include "player16.asm"
#include "shoot.asm"
#include "bullets.asm"
#include "enemies.asm"
#include "init.asm"
#include "enemyhit.asm"
#include "collide.asm"
#include "ebullets.asm"
#include "hityou.asm"
#include "shop16.asm"
#include "helper.asm"
#include "eshoot.asm"
#include "extlev16.asm"
#include "score16.asm"
#include "emove.asm"
#include "info.asm"
#include "images.asm"
#include "levels.asm"
#include "data.asm" 
#include "vars.asm"

end_of_code:
        .end
