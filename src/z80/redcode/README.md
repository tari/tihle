This directory contains a copy of Manual Sainz de Baranda y Goñi's Z80
emulator core as available at https://github.com/redcode/Z80/. This code
is taken from revision 887f5407bf780ea45ad48686e255f5135ab2617e of that
repository.

Z80.c and Z80.h are the core implementation and public API, respectively.

z80bits.h provides the necessary definitions in reasonably-portable C that
are meant to be provided by the [Z library](http://zeta.st). In experimentation
I found that Z was very large and surprisingly unreliable with the compilers
I was using, so opted to create a trimmed down and hopefully more reliable
single-header version of the definitions required for the CPU core.

Z/Z80.h is the Z80 machine definition header copied from Z, because the
emulator is tightly coupled to that set of definitions.

The Z80 core is licensed under the GNU GPLv3, and the Z library is Lesser
GPLv3.
