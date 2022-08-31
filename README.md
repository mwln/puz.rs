# noclue
A .puz file parser

## Usage

* Takes in one or more `.puz` files and parses them to TOML format on stdout
* assigns a "smartId", as a means of identifying unique puzzles based on 
  solution grid.
* Can return the following to stdout / file(s) / or through a module function via
  crate import:

```toml
[info]
authors = "Will Shortz & Someone"
size = [15, 15]
id = "something smart"

[grid]
blank = [
   "...",
   "...",
   "...",
]
solution = [
    "dog",
    "oat",
    "bra",
]
extensions [
    # nothing here means puzzle is standard
    # circled squares = o 
    # tile contents are given = g 
    # tile has rebus @ index n = n (`u16`)
    "og3", # circled, contents given, rebus.options[3]
    "...",
    "...",
]

[clues]
across = {
    1: "something",
    2: "else",
    3: "is up!",
}
down = {
    1: "something",
    2: "else",
    3: "is up!",
}

[rebus]
options = [ "CLUB", "DIAMOND", "SPADE", "HEARTS"]
```

## Thoughts

* Think of ways to handle multiple files when outputting to stdout

## Considerations

* How do scrambled puzzles get read?
* Assigning rebus's properly
* GEXT analysis for board setup

