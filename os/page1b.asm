; MULTIPAGE:PAGE:1B

#include "ti83plus.inc"
#define VECTOR(ADDR, PAGE, TARGET) .seek ADDR - $4000 \ .db PAGE \ .dw TARGET

VECTOR(_cpHLDE, cpHLDE_PAGE, cpHLDE)            ; MULTIPAGE:IMPORT:cpHLDE
VECTOR(_PutMap, PutMap_PAGE, PutMap)            ; MULTIPAGE:IMPORT:PutMap
VECTOR(_PutC, PutC_PAGE, PutC)                  ; MULTIPAGE:IMPORT:PutC
VECTOR(_DispHL, DispHL_PAGE, DispHL)            ; MULTIPAGE:IMPORT:DispHL
VECTOR(_PutS, PutS_PAGE, PutS)                  ; MULTIPAGE:IMPORT:PutS
VECTOR(_ClrLCDFull, ClrLCDFull_PAGE, ClrLCDFull); MULTIPAGE:IMPORT:ClrLCDFull
VECTOR(_HomeUp, HomeUp_PAGE, HomeUp)            ; MULTIPAGE:IMPORT:HomeUp
VECTOR(_VPutMap, VPutMap_PAGE, VPutMap)         ; MULTIPAGE:IMPORT:VPutMap
VECTOR(_VPutS, VPutS_PAGE, VPutS)               ; MULTIPAGE:IMPORT:VPutS
VECTOR(_GrBufCpy, GrBufCpy_PAGE, GrBufCpy)      ; MULTIPAGE:IMPORT:GrBufCpy
VECTOR(_MemSet, MemSet_PAGE, MemSet)            ; MULTIPAGE:IMPORT:MemSet
VECTOR(_GetCSC, GetCSC_PAGE, GetCSC)            ; MULTIPAGE:IMPORT:GetCSC
VECTOR(_DivHLBy10, DivHLBy10_PAGE, DivHLBy10)   ; MULTIPAGE:IMPORT:DivHLBy10
VECTOR(_GrBufClr, GrBufClr_PAGE, GrBufClr)      ; MULTIPAGE:IMPORT:GrBufClr

; Ensure vector table isn't truncated
.seek $4000
