# Module Organization

## Modules

### Reader

- reads the input of a file or stdin
- should be able to read & parse different kinds of types in our file u8, u16, u32, strings, etc
- returns an object with the parsed information in an effective format for processing
- implements its own test suite across `data/stdin/` & `data/files/`

#### Header Functions

- read_puz
- check_validity
- get_header
- get_layout
- get_board_info
- get_clues
- get_notes
- get_extras
- get_gext
- get_ltim
- get_grbs
- get_gext

### Processor

- used for processing the data we receive from reader 
- we should be able to assume that the reader handles any errors in formatting before we reach this point
- we don't want to process anything that isn't relevant to the output we require.
	- this should be handled by the reader, by not returning an object that is unimportant / doesn't hold information
- should use class like structure with children for each key component of the puzzle.
- should have switch/match like statement for keyed object being processed. 
- keyed object could have subimplementation on a per case basis, or just sub modules
- if we reuse alogrithms we could extract those to a library.

##### Processor Functions

- process_puz

