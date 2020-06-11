; Minimal tihle OS image.
;
; This OS image is meant to be built with Spasm:
; https://github.com/alberthdev/spasm-ng
;
; > spasm os.asm os.bin
;
; It is designed to provide a minimal OS image that traps known entry points
; into the emulator, and forces a reset on others.

; Trap instruction: ED25 + 16 bit parameter
.addinstr TRAP * 25ED 4 NOP

; Fill the unimplemented parts of memory with rst 00h
.fill $4000, $c7

#define TRAP_RESET 0
#define TRAP_BCALL 1
#define TRAP_OS_INTERRUPT 2

.seek $0000
    trap TRAP_RESET
    rst 00h

.seek $0028
    trap TRAP_BCALL
    ret

.seek $0038
    trap TRAP_OS_INTERRUPT
    reti

.seek $4000
