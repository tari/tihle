.nolist

; Trap instruction: ED25 + 16 bit parameter
.addinstr TRAP * 25ED 4 NOP

; Traps: defined parameters to the trap instruction.
;
; Values here must match those in src/traps.rs. See that file for more
; information on each trap.
#define TRAP_RESET 0
#define TRAP_BCALL 1
#define TRAP_BCALL_RETURN 2
#define TRAP_OS_INTERRUPT 3

.list