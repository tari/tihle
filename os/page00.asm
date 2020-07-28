; MULTIPAGE:PAGE:00

.nolist
#include "tihle-os.inc"
#include "ti83plus.inc"

; Fill the unimplemented parts of memory with rst 00h
.fill $4000, $c7
.list

.seek $0000
    trap TRAP_RESET
    rst 00h

.seek $0028
    jr bcall_handler

.seek $0038
    di
    trap TRAP_OS_INTERRUPT
    ex af, af'
    ; Acknowledge all interrupts by disabling them
    xor a
    out (3), a
    ; Enable interrupts again
    ld a, $17
    out (3), a
    ex af, af'
    ei
    reti

; Implement bcalls.
;
; See https://wikiti.brandonw.net/index.php?title=83Plus:OS:How_BCALLs_work for
; information on how TI-OS implements bcalls; we do largely the same thing, but
; trap into the emulator to do the tricky parts.
;
; In short, we need to:
;  * read a flash page and address from the vector table on page 1B
;  * map the page from the vector table into bank A
;  * jump to the address on that page with the same register values as we
;    had on entry.
;  * map the original page back into bank A and return without modifying
;    any registers
;
; It's much easier to do most of these things in the emulator so we don't need
; to be careful to preserve registers. The trap will:
;  1. Fixup the return address from the handler
;  2. Push the current page mapped into bank A as the high byte of a value
;     on the stack.
;  3. Read the vector table
;  4. Map the page from the vector table into bank A
;  5. Call the target from the vector table
;
; When the target returns, we run another trap to restore.
bcall_handler:
    trap TRAP_BCALL
    trap TRAP_BCALL_RETURN
