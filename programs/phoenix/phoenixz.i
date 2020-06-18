;##################################################################
;
;   Phoenix-Z80 (Variable and data structure defintions)
;
;   Programmed by Patrick Davidson (pad@ocf.berkeley.edu)
;        
;   Copyright 2011 by Patrick Davidson.  This software may be freely
;   modified and/or copied with no restrictions.  There is no warranty.
;
;   This file was last updated July 30, 2011.
;
;##################################################################   

#define PVERS "4.3"
#define VERS_BYTE $43
;#define ENABLE_CHEATS

;############## Level initialization script commands

L_END           =0      ; end level definition
L_SHOP          =1      ; go to shop
L_GAMEEND       =4      ; mark end of game
L_SET_POWER     =7      ; set enemy power (byte)
L_SET_MOVETYPE  =10     ; set movement type (byte)
L_SET_MOVEDATA  =13     ; set movement data (word)
L_SET_MOVEDATA2 =16     ; set third byte of movement data (byte)
L_IMAGE_STILL   =19     ; set image to sprite (pointer)
L_IMAGE_ANIM    =22     ; set image to animated (pointer)
L_SET_FIRETYPE  =25     ; set firing type (byte)
L_SET_FIRERATE  =28     ; set firing rate (byte)
L_SET_WEAPON    =31     ; set weapon
L_SET_FIREPOWER =34     ; set damage
L_INSTALL_ONE   =37     ; install one enemy (byte X, byte Y)
L_INSTALL_ROW   =40     ; install enemy row (X, Y, byte num, byte spacing)
L_GOTO          =43     ; go to the following word
L_DEFAULT_ROW   =46     ; install enemy row (X, Y) num = 6, spacing = 15
                        ; do not use this one in external levels

;############## Enemy movement types

EM_STANDARD     =0      ; standard swinging enemy
EM_NONE         =3      ; doesn't move
EM_BOSS         =4      ; boss
EM_BOUNCE       =7      ; bouncing enemy
EM_PATTERNSTART =10     ; pattern-following enemy initialization
EM_PATTERNWAIT  =13     ; pattern-following enemy waiting
EM_PATTERNMAIN  =16     ; pattern-following enemy in pattern
EM_RAMPAGE      =19     ; "rampaging" enemy
EM_RAMPAGEWAIT  =22     ; standard enemy which is ready to rampage
EM_SWOOPHORIZ   =25     ; stages of swwop
EM_SWOOPDOWN    =28
EM_SWOOPUP      =31
EM_SWOOPWAIT    =34     ; swooping enemy waiting to enter
EM_RAMPAGEINIT  =37     ; enemy rampaging from the start

;############## Firing types

FT_RANDOM       =0      ; e_firerate/256 probability per frame
FT_NONE         =2      ; never fires
FT_PERIODIC     =3      ; fires every e_firerate frames

;############## Weapon types

W_NORMAL        =0      ; small bullet, straight down
W_DOUBLE        =3      ; two aimed bullets (as used by boss)
W_SEMIAIM       =6      ; accounting for X and Y position, limited angle
W_BIG           =9      ; large, fully aimed bullet
W_HUGE          =12     ; huge, fully aimed bullet
W_ARROW         =15
W_SINGLEBIG     =18     ; single-shot versionsof big, huge
W_SINGLEHUGE    =21
                                                             
;############## Player bullet structure definition

pb_type         =0
pb_dmg          =1
pb_x            =2
pb_w            =3
pb_y            =4
pb_h            =5
pb_img          =6
pb_data         =8

pb_size         =9

pb_num          =16

;############## Enemy structure definition

e_pwr           =0      ; 0 = dead, -1 = exploding
e_movetype      =1      ; 0 = nonmoving (code in emove.asm)
e_movedata      =2      ; 3 bytes of data for movement sequencing
e_x             =5      ; X coordinate
e_w             =6      ; width
e_y             =7      ; Y coordinate
e_h             =8      ; height
e_imageseq      =9      ; countdown to next image (0 = still image)
e_imageptr      =10     ; pointer to image if still, sequence otherwise
e_firetype      =12     
e_firerate      =13     ; fire rate (random probability or timing)
e_firedata      =14     ; firing countdown
e_fireweapon    =15     ; weapon used
e_firepower     =16     ; bullet strength

e_size          =17
e_num           =18

;############## Enemy bullet structure definition

eb_type         =0
eb_dmg          =1
eb_x            =2
eb_w            =3
eb_y            =4
eb_h            =5
eb_data         =6

eb_size         =6

eb_num          =15

;############## Temporary variables

enemy_buffer    =TEXT_MEM

x_offset        =TEXT_MEM+98
option_item     =TEXT_MEM+99            ; item chosen in option menu
speed           =TEXT_MEM+100           ; interrupt delay counter
money_amount    =TEXT_MEM+101           ; value of each $ dropped
bonus_score     =TEXT_MEM+102           ; bonus if you win
decimal_amount  =TEXT_MEM+104           ; money value in decimal
enemy_pointer   =TEXT_MEM+106
gfx_target      =TEXT_MEM+108
in_game         =TEXT_MEM+110
misc_flags      =TEXT_MEM+111
data_addr       =TEXT_MEM+112
level_addr      =TEXT_MEM+115
tempdata        =TEXT_MEM+118
shop_item       =TEXT_MEM+121
test_coords     =TEXT_MEM+122           ; scratch for collision detection
timer           =TEXT_MEM+126
jp2nd           =TEXT_MEM+127           ; 2nd pressed last frame?
