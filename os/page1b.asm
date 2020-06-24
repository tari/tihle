#include "ti83plus.inc"
#define VECTOR(ADDR, PAGE, TARGET) .seek ADDR - $4000 \ .db PAGE \ .dw TARGET

VECTOR(_cpHLDE, 1, $4000)
VECTOR(_PutMap, 1, $4003)
VECTOR(_PutC, 1, $4006)
VECTOR(_DispHL, 1, $4009)
VECTOR(_PutS, 1, $400c)
VECTOR(_ClrLCDFull, 1, $400f)
VECTOR(_HomeUp, 1, $4012)
VECTOR(_VPutMap, 1, $4015)
VECTOR(_VPutS, 1, $4018)
VECTOR(_GrBufCpu, 1, $401b)
VECTOR(_MemSet, 1, $401e)
VECTOR(_GetCSC, 1, $4021)
VECTOR(_DivHLBy10, 1, $4024)

; Ensure vector table isn't truncated
.seek $4000
