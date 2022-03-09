# MOTUS

Current dictionaries are provided in french and can contain some words not included in official motus dictionary.

## Usage with cargo

To try to find a 8-letter word, starting with "c", type

```sh
$ cargo r -r -- c7
```

It will start searching a word starting with "c" and followed with seven unknown letters.
The `-r` flag (for release mode) is optional, but you should use it anyway :smile:

You can bench a specific dictionary and find which word is the best to start a try.
To do this, simply replace `c7` in the previous exemple with the `--bench` flag followed by the dictionary name.
It will print something like :

```sh
$ cargo r -r -- --bench p
For words of len 6, best word is pacify (xxx) in 0.00s
For words of len 7, best word is pudding (xxx) in 0.00s
For words of len 8, best word is potatoes (xxx) in 0.00s
For words of len 9, best word is pacemaker (xxx) in 0.00s
```

To update the dictionary, simply put the four numbers in brackets at the beginning.
Or leave the four first lines empty to disable this feature
