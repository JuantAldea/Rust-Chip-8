# CHIP-8 Emulator (Interpreter)
Just another CHIP-8 interpreter written in Rust.

## Compile & Run
```$ cargo run --release path/to/rom```
## Controls
CHIP-8 Machines have an hexadecimal pad, which is mapped into standard QWERTY like so:

```
HEX PAD | QWERTY
1 2 3 C | 1 2 3 4
4 5 6 D | Q W E R
7 8 9 E | A S D F
A 0 B F | Z X C V
```

### Debug controls
* `p` -> `Pause CPU`
* `n` -> `Next cycle`
* `o` -> `Reset program`
* `Numkey+` -> `Duplicate clock freq`
* `Numkey+` -> `Halve clock freq`
* `Numkey0` -> `Reset freq (600hz)`

Modifying the Clock frequency will alter the counting speed of the sound (`st`) and the delay (`dt`) registries, as they tick at a tenth of the clock speed (60hz).

## Resources
* http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
* http://mattmik.com/files/chip8/mastering/chip8.html

## Screenshots
![image](https://user-images.githubusercontent.com/1664307/70995082-ef1a0380-20cf-11ea-8f43-97c67a446f4a.png)
![image](https://user-images.githubusercontent.com/1664307/70998539-df062200-20d7-11ea-94f4-464cb76be4bf.png)
