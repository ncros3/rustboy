# Rustboy

An implementation of a gameboy emulator in rust, following the book in [gameboy book](https://github.com/rylev/DMG-01/blob/master/book/src/SUMMARY.md).

The gameboy CPU is designed around a Zilog Z80 which is based on a Intel 8080 (it was shipped in 1974; for reminder the Intel 4004 was shipped in 1971). 

Its famous sucessors, intel's 8086 and 8088 (respectively shipped in 1978 and 1979) were the base of the IBM PC. This personnal computer was famous because of its success and its operating system: MS-DOS (the first operating system shipped by Microsoft).

Complete informations about the gameboy can be found in [PanDocs](https://gbdev.io/pandocs/Specifications.html). More precise informations about the instruction set can be found in [megan sulli blog](https://meganesulli.com/generate-gb-opcodes/) and [pastraiser website](https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html).
