# OS image for tihle

tihle requires some kind of OS image in order to run, but is not designed to be
used with TI's operating system for the calculators. The OS implemented in this
directory provides that operating system.

Rather than attempting to be a reimplementation of TI-OS, this OS traps into
tihle where convenient- the emulator can implement many functions more easily,
more efficiently or both.

## Building

This image is designed to be built with
[spasm](https://github.com/alberthdev/spasm-ng). Other assemblers might work
but are not tested.

Each `page*.asm` file represents a single Flash page.
