# Qoboy

A gameboy emulator with an embedded debugger and a video ram viewer.

## Installation and running

Clone and build the projet with the following commands

```shell
git clone https://github.com/qoda-dev/qoboy.git

cd qoboy/

cargo build --release
```

To use the emulator, you have to bring your own **boot rom** and **game rom** files. Then you can run the game with the following command: 

```shell
cargo run <boot_rom_path> <game_rom_path>
```

The keyboard mapping is defined as follows:

| Gameboy control | Keyboard |
| ----------------- | ------- |
| A | a |
| B | z |
| start | backspace |
| select | enter |
| left | left arrow |
| right | right arrow |
| up | up arrow |
| down | down arrow |

## Embedded debugger

This emulator comes with an embedded **video ram viewer** and a light **debugger** which can ease the development of your game or your own emulator by using this one as a reference.

To launch the debugger, add **--debug** when running your game rom:

```shell
cargo run <boot_rom_path> <game_rom_path> --debug
```

Till now, the debugger can handle the following commands:

| command | argument | description |
| ----------------- | ------- | ------ |
| run | none | run the cpu until it encounters a breakpoint or a halt command is received |
| halt | none | when the cpu is running, halt its execution to the current program counter |
| step | none | when the cpu is halted, execute the instruction pointed by the program counter and update the PC to the next instruction |
| break_set | address | set a breakpoint to the address |
| break_reset | none | reset the breakpoint |

The emulator can manage only **one breakpoint** and the address passed to the **break_set** command shall meet the following format:

```shell
break_set C012
```

Where **C012** is the program address on which we want to break in **hexadecimal** format.

> When launched with the **--debug** option, the emulator stops at address 0x0000 by default and waits for a command just like after a **halt** command has been typed. 
> Type **run** or **step** to run your program.

## Tests

In addition to unit tests for each module, more general functionnal tests are done with blargg's and Acid2 test roms.

### Blargg's tests

Source files can be found [here](https://github.com/retrio/gb-test-roms). These roms are used to test general behaviour of CPU, timer and memory subsystems.

| Blargg's test rom | Comment | Result |
| ----------------- | ------- | ------ |
| cpu_instrs | none | :heavy_check_mark: |
| instr_timing | none | :heavy_check_mark: |
| interrupt_time | need sound to pass | :x: |
| dmg_sound | need sound to pass | :x: |
| oam_bug | not implemented | :x: |
| halt_bug | not implemented | :x: |
| mem_timing | need a clock cycle accurate emulator | :x: |
| mem_timing-2 | need a clock cycle accurate emulator | :x: |

### Acid2 tests

Source files can be found [here](https://github.com/mattcurrie/dmg-acid2). This rom is used to test the PPU unit.

| test rom | Comment | Result |
| -------- | ------- | ------ |
| dmg_acid2 | none | :x: |

## Ressources

### General

- https://gbdev.io/pandocs/Specifications.html
- https://gbdev.gg8.se/wiki/articles/Main_Page

### Opcodes

- https://meganesu.github.io/generate-gb-opcodes/
- https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html

### Reference emulators

- https://github.com/Gekkio/mooneye-gb
- https://github.com/mohanson/gameboy
- https://github.com/rylev/DMG-01