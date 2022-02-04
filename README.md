# wpm
A simple rust program to test typing speed.

Simply type the prompted words and your wpm will be calculated based on the 
characters typed within the time limit.


![wpm demo](wpm-demo.gif)

---

## Compilation

After ensuring that rustup is installed and a toolchain is set:
```
git clone https://github.com/RottenFishbone/wpm.git
cd wpm
cargo run --release
```

`dict.txt` holds the list of words, if the binary is moved, this will need to be in the folder EXECUTING it.
This will be fixed in the future.

## Author
Jayden Dumouchel -- jdumouch@ualberta.ca | rottenfishbone@pm.me

### Notes
At the moment this project is hard paused, I will likely pick it up again in the future when my schooling is done.

This is untested on windows but afaik it will compile and run correctly.
