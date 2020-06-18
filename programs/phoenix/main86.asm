;##################################################################
;
;   P H O E N I X         F O R        T I - 8 6
;
;   Programmed by Patrick Davidson (pad@ocf.berkeley.edu)
;        
;   Copyright 2015 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last updated April 13, 2015.
;
;##################################################################

;############## Definitions of system calls

#define ROM_CALL(addr) call addr

DO_LD_HL_MHL    =$4010              ; _ldHLind
UNPACK_HL       =$4044              ; _divHLby10
_flushAllMenus  =$49DC
_putc           =$4A2B            
_puts           =$4A37          
CLEARLCD        =$4A7E              ; _clrLCD
_runindicoff    =$4AB1              
_vputs		=$4AA5

;############## Definitions of memory areas

GFX_AREA        =$9400
EXT_LEVEL       =$B100
VIDEO_2         =$CA00
LEVEL_LOCATION  =$B100

CONTRAST        =$C008              
CURSOR_ROW      =$C00F              ; _curRow
CURSOR_COL      =$C010              ; _curCol
TEXT_MEM        =$C0F9              ; _textShadow
CURSOR_X	=$C37C

_asm_exec_ram   =$D748

;############## Definitions of GET_KEY values

K_MORE          =$38
K_EXIT          =$37
K_SECOND        =$36
K_F1            =$35
K_F2            =$34
K_F3            =$33
K_F4            =$32
K_F5            =$31
K_ALPHA         =$30
K_DEL           =$20
K_3             =$12
K_CLEAR         =$0F
K_MINUS         =$0B
K_PLUS          =$0A
K_ENTER         =$09
K_UP            =$04
K_DOWN          =$01

;############## Good stuff

#define __TI86__
#define __85OR86__
#include "phoenixz.i"

;############## TI-86 program header (for YAS, etc.)

        .org    _asm_exec_ram
        nop
        jp      startup
        .dw     0
        .dw     title

#include "vars.asm"

title:  .db     "Phoenix ", PVERS, " by Patrick D",0

;############## Initialization / exit

startup:              
        call    _flushAllMenus
        call    _runindicoff

        call    main

        ld      hl,variable_name
        call    $42d7                   ;_MOV10TOOP1
        call    $46CB                   ; _FINDSYM; BDE -> start of program
        ld      hl,perm_vars+4-_asm_exec_ram
        ld      a,b
        add     hl,de
        adc     a,0                     ; AHL -> perm storage in variable
        call    $5285                   ; _SET_ABS_DEST_ADDR
        ld      a,0
        ld      hl,perm_vars
        call    $4647                   ; _SET_ABS_SRC_ADDR
        ld      a,0
        ld      hl,perm_vars_end-perm_vars
        call    $464f                   ; _SET_MM_NUM_BYTES
        call    $52ed                   ; _mm_ldir

        ld      hl,TEXT_MEM
        ld      (hl),' '
        ld      bc,167
        ld      de,TEXT_MEM+1
        ldir
        call    $4A7E                   ;_clrLCD
        ld      hl,0
        ld      ($C00F),hl              ;CURSOR_ROW
        ld      (iy+13),6
        set     0,(iy+3)
        ret

variable_name:
        .db     0,7,"phoenix"

;############## Interrupt handler

interrupt_code:
        push    af
        push    hl
        call    $0                      ; Actual address written over this
        pop     hl

        in      a,(3)                   ; Bit 1 = ON key status
        and     1
        add     a,9                     ; A = 10 if ON pressed, 9 if not
        out     (3),a                  
        ld      a,11
        out     (3),a                   
        ei
        pop     af
        reti
interrupt_code_end:

;############## Simulation of Usgard functions

OTH_CLEAR:
        ld      (hl),0
OTH_FILL:
        ld      d,h
        ld      e,l
        inc     de
        ldir
        ret

OTH_ARROW:
        ld      a,%00111111
        out     (1),a
        push    ix
        pop     ix
        in      a,(1)
        or      %00001111
        ld      b,a
        ld      a,%01111110
        out     (1),a
        push    ix
        pop     ix
        in      a,(1)
        and     b
        ret

INT_INSTALL:
        ld      (interrupt_code+3),hl
        ld      hl,$9000
        ld      (hl),$91
        ld      bc,256
        call    OTH_FILL
        ld      hl,interrupt_code
        ld      de,$9191
        ld      bc,interrupt_code_end-interrupt_code
        ldir
        ld      a,$90
        ld      i,a
        im      2
        ret

INT_REMOVE:
        im      1
        ret           

;############## GET_KEY replacement

GET_KEY:
        push    hl
        push    de
        push    bc
        ld      e,0                     ; E = GET_KEY result
        ld      hl,getkeylastdata       ; HL = ptr to last read's table
        ld      a,$fe                   ; A = key port mask
        ld      d,1                     ; D = individual key's mask
        ld      c,0                     ; C = key number counter
gkol:   out     (1),a
        ld      b,8                         
        push    af
gkl:    inc     c
        in      a,(1)
        and     d
        jr      nz,nokey
        ld      a,(hl)
        and     d
        jr      z,nokey
        ld      e,c
nokey:  rlc     d
        djnz    gkl
        in      a,(1)
        ld      (hl),a
        pop     af
        inc     hl
        rlca     
        cp      $7F
        jr      nz,gkol
        ld      a,e
        pop     bc
        pop     de
        pop     hl
        ret

getkeylastdata:
        .db     $ff,$ff,$ff,$ff,$ff,$ff,$ff

;############## CP_HL_DE replacement

DO_CP_HL_DE:
        push    hl
        and     a
        sbc     hl,de
        pop     hl
        ret

;############## D_HL_DECI replacement

D_HL_DECI:
        push    bc
        ld      de,up_data+4
        ld      b,5
ldhld:  call    UNPACK_HL
        add     a,48
        ld      (de),a
        dec     de
        djnz    ldhld
        ld      hl,up_data
        ld      b,4
lis:    ld      a,(hl)
        cp      48
        jr      nz,dis
        ld      (hl),32
        inc     hl
        djnz    lis
dis:    ld      hl,up_data
        call    D_ZT_STR
        pop     bc
        ret

up_data:
        .db     "PAD98",0               ; Because this was coded in 1998!

;############## Custom-font-safe display routines

D_ZT_STR:
        di
        call    _puts
        ei
        ret

D_ZM_STR:
	di	
	call	_vputs
	ei
	ret
	
TX_CHARPUT:
        di
        call    _putc
        ei
        ret

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

        .end
