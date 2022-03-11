# MOTUS

Current dictionaries are provided in french and can contain some words not included in the official Motus dictionary. Additionally, dictionaries must contain only characters in `/[a-z]/`, otherwise it will produce errors when loading. Finally, dictionaries must start with four empty rows or IDs ([see below][1]).

Tips: All commands use the `release` flag. For more detail, see `cargo -h` and `cargo run -h`.

## Usage: Find a word

This is the normal use of this program. Simply give it the first letter and the number of unknown letters and it will print something like this:

```sh
$ cargo r -r -- b7
Dico loaded in 650µs (1393 words)
Best word: brulions
Result:
```

At this point, you have to input the answer given by the game, something like "rybyybbb". 'r' for a correct character at the right place, 'y' for a correct character at the wrong place and 'b' for an incorrect character. Then, it will continue until the correct word is found:

```sh
$ cargo r -r -- b7
Dico loaded in 650µs (1393 words)
Best word: brulions
Result: rybyybbb
7 words remaining
Word found in 0.10s
Best word: babiller (1.00)
Result: rybyybby
Obviously: blizzard
```

Once you input the result, the program prints how many possible words are left in the dictionary, or "Obviously" followed by the only possible word. If there is more than one word, the program will print the next best word with the average words remainings after this try in brackets.

## Usage: bench a dictionary

You can bench a specific dictionary to find which word is the best to start a try with. To do this, simply replace `b7` in the previous exemple with the `--bench` flag followed by the dictionary name. It will print something like this:

```sh
$ cargo r -r -- --bench p
For words of len 6, best word is pacify (xxx) in 0.00s
For words of len 7, best word is pudding (xxx) in 0.00s
For words of len 8, best word is potatoes (xxx) in 0.00s
For words of len 9, best word is pacemaker (xxx) in 0.00s
```

To update the dictionary, simply put the four numbers in brackets at the beginning.
Or leave the four first lines empty to disable this feature

## Usage: auto-play

If you already know which word you should find, you can do a run with the same output as [the 1st part][2] but without having to input the results. To do this, simply type:

```sh
$ cargo r -r -- -a blizzard
```

[1]:#usage-bench-a-dictionary
[2]:#usage-find-a-word
