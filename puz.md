# .puz File Documentation

`.puz` is a file format for crossword puzzles.

This file format comes from [AcrossLite](https://www.litsoft.com/across/alite/download/), a free software developed by [litsoft](https://www.litsoft.com) for solving and publishing crosswords.

## File Contents

- A fixed-size [header](#header-format).
- The puzzle solution and empty board state.
- A series of NUL-terminated variable-length strings.
- A series of sections with [additional information](#extra-sections) about the puzzle.

## Header Format

| Component             | Offset | End  | Length | Type       | Description                                                                              |
| --------------------- | ------ | ---- | ------ | ---------- | ---------------------------------------------------------------------------------------- |
| Checksum              | 0x00   | 0x01 | 0x2    | u16        | overall file checksum                                                                    |
| File Magic            | 0x02   | 0x0D | 0xC    | string     | NUL-terminated constant string: 4143 524f 5353 2644 4f57 4e00 ("ACROSS&DOWN")            |
| CIB Checksum          | 0x0E   | 0x0F | 0x2    | u16        | (defined later)                                                                          |
| Masked Low Checksums  | 0x10   | 0x13 | 0x4    | [u16, u16] | A set of checksums, XOR-masked against a magic string.                                   |
| Masked High Checksums | 0x14   | 0x17 | 0x4    | [u16, u16] | A set of checksums, XOR-masked against a magic string.                                   |
| Version String(?)     | 0x18   | 0x1B | 0x4    | string     | e.g. "1.2\0"                                                                             |
| Reserved1C(?)         | 0x1C   | 0x1D | 0x2    | ?          | In many files, this is uninitialized memory                                              |
| Scrambled Checksum    | 0x1E   | 0x1F | 0x2    | u16        | In scrambled puzzles, a checksum of the real solution (details below). Otherwise, 0x0000 |
| Reserved20(?)         | 0x20   | 0x2B | 0xC    | ?          | In files where Reserved1C is garbage, this is garbage too.                               |
| Width                 | 0x2C   | 0x2C | 0x1    | u8         | The width of the board                                                                   |
| Height                | 0x2D   | 0x2D | 0x1    | u8         | The height of the board                                                                  |
| # of Clues            | 0x2E   | 0x2F | 0x2    | u16        | The number of clues for this board                                                       |
| Unknown Bitmask       | 0x30   | 0x31 | 0x2    | u16        | A bitmask. Operations unknown.                                                           |
| Scrambled Tag         | 0x32   | 0x33 | 0x2    | u16        | 0 for unscrambled puzzles. Nonzero (often 4) for scrambled puzzles.                      |

## Board Layout

Useful info for parsing

```rust
let width = 3;
let height = 3;
let size = width * height
```

Example board

```
C A T
# T #
# E #
```

| Component      | Offset    | End            | Length | Type   | Output        |
| -------------- | --------- | -------------- | ------ | ------ | ------------- |
| Blank board    | 0x34      | 0x34 + size    | size   | string | `"---.-..-."` |
| Solution board | 0x34+size | 0x34 + 2\*size | size   | string | `"CAT.T..E."` |

**Edge Cases**

- `.puz` file was saved to disk as player was solving via AcrossLite - meaning the blank board will contain useless characters.

### Strings Section

This section occurs immediately following the layout.

```rust
const NUL_CHAR: char = '\0';
let num_clues: u16; // known from header
```

| Component | Offset         | End             | Length | Type    | Output                       |
| --------- | -------------- | --------------- | ------ | ------- | ---------------------------- |
| Title     | 0x34+(2\*size) | @ NUL character | varied | string  | "A title"                    |
| Author    | n/a            | @ NUL character | varied | string  | "Will Shortz"                |
| Copyright | n/a            | @ NUL character | varied | string  | "© 2013, The New York Times" |
| Clue#1    | n/a            | @ NUL character | varied | string  | "A clue"                     |
| ...       | ...            | ...             | ...    | strings | ...                          |
| Clue#n    | n/a            | @ NUL character | varied | string  | "Last clue"                  |
| Notes     | n/a            | @ NUL character | varied | string  | "A note"                     |

- In some cases, a "Note" has been included in the title instead of using the designated notes field.
  It is separated from the title by a space (ASCII 0x20) and begins with the string "NOTE:" or "Note:".
- The clues are arranged numerically, where Across clues occur before the Down clue.
- Nowhere in the file does it specify which cells get numbers or which clues correspond to which numbers. These are instead derived from the shape of the puzzle.

### Extra Sections

The known extra sections are:

| Section Name | Description                                    |
| ------------ | ---------------------------------------------- |
| GRBS         | where rebuses are located in the solution      |
| RTBL         | contents of rebus squares, referred to by GRBS |
| LTIM         | timer data                                     |
| GEXT         | circled squares, incorrect and given flags     |
| RUSR         | user-entered rebus squares                     |

In official puzzles, the sections always seem to come in this order, when they appear. It is not known if the ordering is guaranteed. The GRBS and RTBL sections appear together in puzzles with rebuses. However, sometimes a GRBS section with no rebus squares appears without an RTBL, especially in puzzles that have additional extra sections.

The extra sections all follow the same general format, with variation in the data they contain. That format is:

| Component | Length (bytes) | Description                                                                                    |
| --------- | -------------- | ---------------------------------------------------------------------------------------------- |
| Title     | 0x04           | The name of the section, these are given in the previous table                                 |
| Length    | 0x02           | The length of the data section, in bytes, not counting the null terminator                     |
| Checksum  | 0x02           | A checksum of the data section, using the same algorithm described above                       |
| Data      | variable       | The data, which varies in format but is always terminated by null and has the specified length |

The format of the data for each section is described below.

#### GRBS

The GRBS data is a "board" of one byte per square, similar to the strings for the solution and user state tables except that black squares, letters, etc. are not indicated. The byte for each square of this board indicates whether or not that square is a rebus. Possible values are:

- `0` indicates a non-rebus square.
- `1+n` indicates a rebus square, the solution for which is given by the entry with key n in the RTBL section.

If a square is a rebus, only the first letter will be given by the solution board and only the first letter of any fill will be given in the user state board.

### RTBL

The RTBL data is a string containing the solutions for any rebus squares.

These solutions are given as an ascii string. For each rebus there is a number, a colon, a string and a semicolon. The number (represented by an ascii string) is always two characters long - if it is only one digit, the first character is a space. It is the key that the GRBS section uses to refer to this entry (it is one less than the number that appears in the corresponding rebus grid squares). The string is the rebus solution.

For example, in a puzzle which had four rebus squares containing "HEART", "DIAMOND", "CLUB", and "SPADE", the string might be:

`" 0:HEART; 1:DIAMOND;17:CLUB;23:SPADE;"`

Note that the keys need not be consecutive numbers, but in official puzzles they always seem to be in ascending order. An individual key may appear multiple times in the GRBS board if there are multiple rebus squares with the same solution.

### LTIM

The LTIM data section stores two pieces of information:

- how much time the solver has used (in seconds)
- whether the timer is running or stopped (0: on, 1: off).

**This data is unimportant, as it is proprietary information based on an AcrossLite session.**

#### GEXT

The GEXT data section is identified by the string `"GEXT"`. This string is then followed by set bytes representing the `board_size`. The byte-wise sequence of `length == board_size` is what we care about.

**Bitmask info, per byte in sequence:**

- `0x10` - square was previously marked incorrect
- `0x20` - square is currently marked incorrect
- `0x40` - contents were given
- `0x80` - square is circled/shaded.

None, some, or all of these bits may be set for each square. For parsing, we only care about the `contents_given` && `square_is_circled` parts, as these **relate to the structure of the puzzle**.

#### RUSR

The RUSR section is currently undocumented, and unimportant to parsing the necessary contents of the file.

### Checksums

The file format uses a variety of checksums.

The checksumming routine used in PUZ is a variant of CRC-16. To checksum a region of memory, the following is used:

```c
unsigned short cksum_region(unsigned char *base, int len, unsigned short cksum) {
    int i;
    for (i = 0; i < len; i++) {
        if (cksum & 0x0001) cksum = (cksum >> 1) + 0x8000; else cksum = cksum >> 1; cksum += *(base+i);
    }
    return cksum;
}
```

The CIB checksum (which appears as its own field in the header as well as elsewhere) is a checksum over eight bytes of the header starting at the board width: `c_cib = cksum_region(data + 0x2C, 8, 0);`

The primary board checksum uses the CIB checksum and other data:

```c
cksum = c_cib;
cksum = cksum_region(solution, w*h, cksum);
cksum = cksum_region(grid, w*h, cksum);

if (strlen(title) > 0)
    cksum = cksum_region(title, strlen(title)+1, cksum);

if (strlen(author) > 0)
    cksum = cksum_region(author, strlen(author)+1, cksum);

if (strlen(copyright) > 0)
    cksum = cksum_region(copyright, strlen(copyright)+1, cksum);

for(i = 0; i < num_of_clues; i++)
    cksum = cksum_region(clue[i], strlen(clue[i]), cksum);

if (strlen(notes) > 0)
    cksum = cksum_region(notes, strlen(notes)+1, cksum);
```

### Masked Checksums

The values from `0x10`-`0x17` are a real pain to generate. They are the result of masking off and XORing four checksums; `0x10`-`0x13` are the low bytes, while `0x14`-`0x17` are the high bytes.

To calculate these bytes, we must first calculate four checksums:

- CIB Checksum: `c_cib = cksum_region(CIB, 0x08, 0x0000);`
- Solution Checksum: `c_sol = cksum_region(solution, w*h, 0x0000);`
- Grid Checksum: `c_grid = cksum_region(grid, w*h, 0x0000);`
- A partial board checksum:

```c
c_part = 0x0000;

if (strlen(title) > 0)
    c_part = cksum_region(title, strlen(title)+1, c_part);

if (strlen(author) > 0)
    c_part = cksum_region(author, strlen(author)+1, c_part);

if (strlen(copyright) > 0)
    c_part = cksum_region(copyright, strlen(copyright)+1, c_part);

for (int i = 0; i < n_clues; i++)
    c_part = cksum_region(clue[i], strlen(clue[i]), c_part);

if (strlen(notes) > 0)
    c_part = cksum_region(notes, strlen(notes)+1, c_part);
```

Once these four checksums are obtained, they're stuffed into the file thusly:

```c
file[0x10] = 0x49 ^ (c_cib & 0xFF);
file[0x11] = 0x43 ^ (c_sol & 0xFF); file[0x12] = 0x48 ^ (c_grid & 0xFF);
file[0x13] = 0x45 ^ (c_part & 0xFF);
file[0x14] = 0x41 ^ ((c_cib & 0xFF00) >> 8);
file[0x15] = 0x54 ^ ((c_sol & 0xFF00) >> 8);
file[0x16] = 0x45 ^ ((c_grid & 0xFF00) >> 8);
file[0x17] = 0x44 ^ ((c_part & 0xFF00) >> 8);
```

Note that these hex values in ASCII are the string "ICHEATED".

### Locked/Scrambled Puzzles

The header contains two pieces related to scrambled puzzles. The short at 0x32 records whether the puzzle is scrambled. If it is scrambled, the short at 0x1E is a checksum suitable for verifying an attempt at unscrambling. If the correct solution is laid out as a string in column-major order, omitting black squares, then 0x1E contains cksum_region(string,0x0000).

### Other

- A version of [this archive page](https://code.google.com/archive/p/puz/wikis/FileFormat.wiki), reformatted for nicer viewing.
