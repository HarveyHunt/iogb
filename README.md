# iogb
A GameBoy emulator written in Rust

## Usage
```iogb``` should be run using cargo. Pass the path to your ROM file using ```-r / --rom```.

```
cargo run -- --rom ~/legal_rom.gb
```

The following command line arguments **may** be passed to ```iogb```:
- ```-s```, ```--scale``` The displaying scaling to use (1, 2, 4, 8, 16, 32)
- ```-b```, ```--bootrom``` The path to a gameboy bootrom

## TODO
- CPU
  - Implement all instructions
  - Switch to cycle accurate timing
  - Implement DMA
- GPU
  - Render sprites
  - Render window
  - Switch to cycle accurate timing
- Input
  - Implement controller data register (0xFF00)
- Audio
  - Implement all 4 sound channels
  - Implement sound control registers
- MBC
  - Implement MBC2
  - Implement MBC3
  - Implement MBC5
- Interface
  - Pass inputs to GB
- Misc
  - Implement Gameboy Colour
  - Consider project layout
  - Fix TODOs
  - Pass all of Blargg's tests
  
# Special Thanks
- [Mooneye GB](https://github.com/Gekkio/mooneye-gb)
- [Pan Docs](http://bgb.bircd.org/pandocs.htm)
- [GB instruction set](http://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html)
- [GBdev Wiki](http://gbdev.gg8.se/wiki/articles/Main_Page)
