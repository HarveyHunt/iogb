# iogb
A GameBoy emulator written in Rust

# Running
```iogb``` should be run using cargo. Pass the path to your ROM file using ```-r / --rom``` and the path to your GB Bootrom using ```-b / --bootrom```.

```
cargo run -- --rom ~/legal_rom.gb --bootrom brom.gb
```

# TODO
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
  - Render texture from GPU back buffer
    - Create OpenGL window using glium
    - Write fragment and vertex shaders (helpful for applying effects)
    - Configure texture coords
    - Render back buffer onto a quad as a texture
  - Pass inputs to GB
- Misc
  - Implement Gameboy Colour
  - Consider project layout
  - Fix TODOs
  - Pass all of Blargg's tests
  - OPEN SOURCE!
  
# Special Thanks
- [Mooneye GB](https://github.com/Gekkio/mooneye-gb)
- [Pan Docs](http://bgb.bircd.org/pandocs.htm)
- [GB instruction set](http://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html)
- [GBdev Wiki](http://gbdev.gg8.se/wiki/articles/Main_Page)
