# iogb
A GameBoy emulator written in Rust

# Running
```iogb``` should be run using cargo. Pass the path to your ROM file using ```-r / --rom``` and the path to your GB Bootrom using ```-b / --bootrom```.

```
cargo run -- --rom ~/legal_rom.gb --bootrom brom.gb
```

# Special Thanks
- [Mooneye GB](https://github.com/Gekkio/mooneye-gb)
- [Pan Docs](http://bgb.bircd.org/pandocs.htm)
- [GB instruction set](http://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html)
- [GBdev Wiki](http://gbdev.gg8.se/wiki/articles/Main_Page)
