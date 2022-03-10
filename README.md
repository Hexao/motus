# MOTUS

Current dictionaries are provided in french and can contain some words not included in official motus dictionary. In addition, dictionary must contains only characters among `/[a-z]/`. Otherwise il will produce some errors during dictionary loading. Finally, dictionary must start with four empty rows or IDs ([see bellow][1])

Tips: All commands use the release mod. For more details, see `cargo -h` and `cargo run -h`

## Usage: Find a word

This is the normal use of this program. Simply give it the first letter and the number of unknown letters and it will print something like that:

```sh
$ cargo r -r -- b7
Dico loaded in 650µs (1393 words)
Best word: brulions
Result:
```

At this point, you must give it the anwser the motus game give you, something like "rybyybbb". 'r' for a correct char at the wright place, 'y' for a correct char at the wrong place and 'b' for an incorrect char. Then, it will continue:

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

Once you give the result, the program print how many possible words left in the dictionary, or "Obviously" followed by the only possible word. If there is more than one word, the program will print the next best word with the average words remainings after this try in brackets.

## Usage: bench a dictionary

You can bench a specific dictionary and find which word is the best to start a try. To do this, simply replace `b7` in the previous exemple with the `--bench` flag followed by the dictionary name. It will print something like:

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

If you already know which word you should find, you can do a run with the same output as [the 1st part][2] but without need to put the result. To do this, simply type:

```sh
$ cargo r -r -- -a blizzard
```

[1]:#usage-bench-a-dictionary
[2]:#usage-find-a-word
