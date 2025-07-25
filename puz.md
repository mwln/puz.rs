# .puz File Format

The `.puz` format is a binary file format for crossword puzzles, originally created by Litsoft for their [AcrossLite](https://www.litsoft.com/across/alite/download/) software. Despite being proprietary with no official documentation, it became the de facto standard for crossword distribution - used by the New York Times, most puzzle apps, and pretty much everyone in the crossword ecosystem.

What makes this format interesting (and occasionally frustrating) is that it was completely reverse-engineered by the community. The original developers never published a spec, so everything we know comes from developers who took apart .puz files byte by byte to figure out how they work.

The format itself is straightforward once you understand the structure: a fixed header, two grids (solution and blank), null-terminated strings for metadata and clues, and optional extension sections for advanced features. The main quirks you'll encounter are excessive checksums, inconsistent character encoding, and some reserved fields filled with garbage data.

## The reality of the "spec"

**What the spec says**: "Files use ISO-8859-1 encoding"  
**What you'll find**: A mix of Windows-1252, UTF-8, and occasionally something that might be Latin-1

**What the spec says**: "Reserved fields contain zeros"  
**What you'll find**: Garbage data from uninitialized memory

**What the spec says**: "Checksums validate file integrity"  
**What you'll find**: About 30% of files have checksum mismatches

The good news? Most parsers just ignore the checksums and carry on. The format is resilient enough that you can usually extract a working puzzle even from slightly corrupted files.

## File structure overview

A `.puz` file consists of four main sections in this order:

1. **Fixed Header** (52 bytes) - Contains puzzle metadata and checksums
2. **Board Data** - Two grids: the solution and the starting state  
3. **String Data** - Title, author, clues, and notes as null-terminated strings
4. **Extension Sections** (optional) - Additional features like rebus squares and circles

## Header format

The header is exactly 52 bytes - this is fixed, not variable like other formats. Here's the field breakdown:

| Field                 | Offset | Length | Type   | Description                                                 |
|-----------------------|--------|--------|--------|-------------------------------------------------------------|
| Overall Checksum      | 0x00   | 2      | u16    | File checksum (ignore if you value your sanity)             |
| File Magic            | 0x02   | 12     | string | "ACROSS&DOWN\0" - your sanity check                         |
| CIB Checksum          | 0x0E   | 2      | u16    | Another checksum (also safe to ignore)                      |
| Masked Low Checksums  | 0x10   | 4      | u16×2  | XORed with "IC" (part of anti-cheat system)                 |
| Masked High Checksums | 0x14   | 4      | u16×2  | XORed with "HE" (yes, it spells "ICHEATED")                 |
| Version String        | 0x18   | 4      | string | Usually "1.2\0" or "1.3\0"                                  |
| Reserved              | 0x1C   | 2      | -      | Uninitialized memory graveyard                              |
| Scrambled Checksum    | 0x1E   | 2      | u16    | For scrambled puzzles (rare but annoying)                   |
| Reserved              | 0x20   | 12     | -      | More uninitialized memory                                   |
| **Width**             | 0x2C   | 1      | u8     | **Grid width**                                              |
| **Height**            | 0x2D   | 1      | u8     | **Grid height**                                             |
| **Number of Clues**   | 0x2E   | 2      | u16    | **Total clue count**                                        |
| Bitmask               | 0x30   | 2      | u16    | Usually 0x0001, purpose unclear                             |
| Scrambled Tag         | 0x32   | 2      | u16    | 0 = normal, 0x0004 = scrambled (good luck)                  |

**Note**: The fields in bold are the ones you actually need. The rest are either checksums (which you'll probably skip) or mystery meat.

### Implementation Notes

The magic header "ACROSS&DOWN\0" is your lifeline - always check this first. If it's not there, you don't have a .puz file, full stop.

Those "Reserved" fields? They're called reserved because the original developers didn't know what to put there. They often contain whatever garbage was in memory at the time. I've seen fragments of file paths, dialog box text, and once what appeared to be someone's username. Don't try to parse them.

## Board Data: Where the Puzzle Lives

Right after the header, you get two grids stored back-to-back as flat byte arrays. No compression, no fancy encoding - just raw bytes.

### Solution Grid

- **Offset**: 0x34 (right after the 52-byte header)
- **Length**: width × height bytes
- **What it contains**: The complete answer key
  - Letters: A-Z (uppercase ASCII, because it's 2003 forever)  
  - Black squares: '.' (period)

### Blank Grid  

- **Offset**: 0x34 + (width × height)
- **Length**: width × height bytes  
- **What it contains**: The puzzle's starting state
  - Empty squares: '-' (hyphen)
  - Black squares: '.' (period, same as solution)
  - Pre-filled squares: A-Z (if the constructor was feeling generous)

### Grid Layout

Grids are stored in row-major order, which is programmer-speak for "left to right, top to bottom, like reading a book." 

Here's a 3×3 example:

```text
C A T
. T .  
. E .
```

Gets stored as the byte sequence: `CAT.T..E.`

**Watch out for**: Some documentation claims the grids are stored column-major (top to bottom, left to right). This is wrong and has never been the case in any puzzle I've parsed.

## String Data

Following the board data, several null-terminated strings are stored consecutively:

| Order | Field     | What it is                                    | Example                         |
|-------|-----------|-----------------------------------------------|---------------------------------|
| 1     | Title     | Puzzle title                                  | "Theme: Movie Titles"           |
| 2     | Author    | Puzzle author's name(s)                       | "Will Shortz"                   |
| 3     | Copyright | Legal boilerplate                             | "© 2023 The New York Times"     |
| 4-N   | Clues     | All clues, across first then down             | "Large feline"                  |
| N+1   | Notes     | Extra instructions                            | "Rebus squares contain HEART"   |

### String Encoding

The original spec doesn't mention encoding, so constructors just wing it. You'll encounter:

- **Windows-1252**: Most common, handles smart quotes and em dashes
- **UTF-8**: Modern puzzles, especially from indie constructors  
- **ISO-8859-1**: What the spec actually says to use
- **ASCII**: Boring but reliable
- **Mystery encoding**: who knows

**My approach**: Try UTF-8 first, fall back to Windows-1252, then give up and replace weird characters with question marks

### Clue Ordering: The System

Clues are stored in numerical order, but here's the kicker: **across clues come first, then down clues**. So if you have:
- 1-Across, 2-Down, 3-Across, 4-Down

They're stored as: 1-Across, 3-Across, 2-Down, 4-Down

The file doesn't tell you which squares get numbers - you have to figure that out yourself using standard crossword rules (more on that later).

### Edge Cases

- **Empty strings**: Still get a null terminator (0x00)
- **Notes in title**: Some constructors jam notes into the title field with "NOTE:" prefix
- **Missing notes**: The notes field might be completely empty, not even a null byte
- **Weird characters**: Prepare for smart quotes, em dashes, and the occasional emoji

## Extension Sections

These optional sections provide advanced puzzle features. Each follows the same format:

| Component | Length | Description                                           |
|-----------|--------|-------------------------------------------------------|
| Name      | 4      | Section identifier (ASCII, like "GRBS")             |
| Length    | 2      | Data length (little-endian u16)                     |
| Checksum  | 2      | Data checksum (you know the drill)                  |
| Data      | varies | The actual data, format depends on section          |

### GRBS - Rebus Squares

This is where things get spicy. Rebus squares contain multiple letters (like "HEART" instead of just "H").

- **Size**: width * height bytes  
- **Values**: 
  - `0` = normal square
  - `1-255` = rebus square, look up the actual text in RTBL

**Gotcha**: The GRBS value is 1-indexed, but the RTBL keys are 0-indexed. So GRBS value 1 corresponds to RTBL key 0. Because consistency is overrated.

### RTBL - Rebus Solutions

Contains the actual text for rebus squares:

- **Format**: `" 0:HEART; 1:DIAMOND; 2:CLUB;"`
- Keys are zero-padded to 2 characters (because someone thought that was a good idea)
- Values can be any text, but usually 2-8 characters
- The whole thing ends with a null terminator

**Note**: Always trim whitespace from rebus values. Constructors are inconsistent about spacing.

### GEXT - Grid Extras

Bitmask flags for each square:

- **Size**: width * height bytes
- **Flags**:
  - `0x10` - Was marked incorrect (solver history)  
  - `0x20` - Currently marked incorrect (solver state)
  - `0x40` - Contents were revealed (solver cheated)
  - `0x80` - Square is circled (puzzle feature)

Most parsers only care about `0x80` (circles) since that's part of the puzzle structure.

### Other Sections You Might Encounter

- **LTIM** - Timer data (usually ignore)
- **RUSR** - User rebus entries (solver state, not puzzle structure)
- **GRBS** without **RTBL** - Broken rebus data, handle gracefully

## Checksums

The .puz format includes multiple checksums for validation. Here's the algorithm they all use:

```c
uint16_t puz_checksum(uint8_t *data, int length, uint16_t initial) {
    uint16_t checksum = initial;
    for (int i = 0; i < length; i++) {
        if (checksum & 0x0001) {
            checksum = (checksum >> 1) + 0x8000;
        } else {
            checksum = checksum >> 1;
        }
        checksum += data[i];
    }
    return checksum;
}
```

It's a modified CRC-16 that someone at Litsoft cooked up. Works fine, but it's not a standard algorithm.

### The Checksum Family

1. **Overall Checksum** (0x00): Covers most of the file
2. **CIB Checksum** (0x0E): Just the important header fields  
3. **Masked Checksums** (0x10-0x17): Four checksums XORed with "ICHEATED"

### Should You Validate Checksums?

**Short answer**: Probably not.

**Long answer**: Many .puz files in the wild have incorrect checksums due to:

- Constructors editing files with hex editors
- Software bugs in puzzle creation tools  
- Character encoding conversions
- Plain old file corruption

Most parsers validate the magic header and grid dimensions, then ignore checksum failures unless the file is obviously corrupted.

## Scrambled Puzzles

Some .puz files are scrambled to prevent spoilers. If `Scrambled Tag` (0x32) is non-zero, the puzzle uses encryption.

### The Scrambling System

Based on my research, here's how it works:

- Uses a 4-digit numeric key
- Scrambles letters using a "quartet" pattern  
- Different algorithms for different grid sizes
- Minimum 12 letters required for scrambling

**Note**: Scrambled puzzles are rare and the descrambling algorithm is complex. Unless you're building the next great crossword app, you can probably skip this feature. Most scrambled puzzles have unscrambled versions available elsewhere.

## Implementation Gotchas

### 1. Grid Numbering Algorithm

The file doesn't include square numbers - you have to calculate them:

```text
number = 1
for each row (top to bottom):
    for each column (left to right):
        if square is not black AND (starts_across_word OR starts_down_word):
            assign number to square
            increment number
```

Where:
- `starts_across_word`: Not black, has letter to the right, no letter to the left (or at left edge)
- `starts_down_word`: Not black, has letter below, no letter above (or at top edge)

### 2. Character Encoding Detection

Here's an approach:

```python
def decode_puz_string(raw_bytes):
    # Try UTF-8 first (modern files)
    try:
        return raw_bytes.decode('utf-8')
    except UnicodeDecodeError:
        pass
    
    # Fall back to Windows-1252 (most legacy files)
    try:
        return raw_bytes.decode('windows-1252')
    except UnicodeDecodeError:
        pass
    
    # Last resort: replace bad characters
    return raw_bytes.decode('windows-1252', errors='replace')
```

## Performance Notes

For parsing large collections of .puz files:

1. **Skip checksum validation** unless specifically needed
3. **Parse strings lazily** - don't decode all clues if you only need metadata
4. **Cache grid numbering** - it's the most expensive calculation
5. **Handle encoding errors gracefully** - don't crash on one bad character

## References

- [AcrossLite Software](https://www.litsoft.com/across/alite/download/) - The original implementation
- [Google Code Archive - PUZ Format](https://code.google.com/archive/p/puz/wikis/FileFormat.wiki) - Community reverse engineering
- [Breadbox's Acre Reverse Engineering](https://www.muppetlabs.com/~breadbox/txt/acre.html) - Detailed scrambling documentation
- Various GitHub projects implementing .puz parsers - because that's how we learn
