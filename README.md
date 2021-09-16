# Rustboy

An implementation of a gameboy emulator in rust, following the book in [gameboy book](https://github.com/rylev/DMG-01/blob/master/book/src/SUMMARY.md).

The gameboy CPU is designed around a Zilog Z80 which is based on a Intel 8080 (it was shipped in 1971; for reminder the Intel 4004 was shipped in 1971).

Complete informations about the gameboy instructions set can be found [here](https://gbdev.io/pandocs/CPU_Instruction_Set.html), [here](https://meganesulli.com/generate-gb-opcodes/) and [here](https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html).