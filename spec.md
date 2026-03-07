s# AYML version 1.0

**2026-03-07**

**Status of this Document**

This is the **AYML specification v1.0**.
It defines the **AYML 1.0 data language**.

## Abstract

AYML is a software configuration language that looks like YAML and acts like JSON.
It is meant to be a human-friendly, cross language, Unicode based software
configuration language. Unlike YAML, it is not designed as a fully featured
data serialization framework. There are no references types, unordered sets,
duplicate nodes, document prefixes, or other complex features supported by YAML.

## Goals

The design goals for AYML are, in decreasing priority:

1. AYML should be easy to understand
1. AYML should be easy to author correctly
1. AYML should be expressible in the native data structures of modern languages
1. AYML should be easy to deserialize into strongly typed data structures
1. AYML should incorporate comments as a formal part of the document structure

## Terminology

The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD",
"SHOULD NOT", "RECOMMENDED",  "MAY", and "OPTIONAL" in this document are to be
interpreted as described in RFC 2119.

## Language Overview

This section provides a quick glimpse into the expressive power of AYML.
It is not expected that the first-time reader grok all of the examples.
Rather, these selections are used as motivation for the remainder of the
specification.

### Collections

A block collection uses indentation for scope and begins each entry on its own line.
A block sequence indicates each entry with a dash and space `- `.
A mapping uses a colon and space `: ` to mark each key/value pair. A mapping may not have duplicate keys.
A comment begins with a `#`.

#### Mapping Key Ordering

If an AYML mapping is coded without a target data model, mapping keys are not guaranteed to be ordered (e.g. a `HashMap`)
If an AYML mapping is coded with a target data model, the data model determines ordering guarantees.

**Example: Sequence of Scalars (ball players)**

```
- Mark McGwire
- Sammy Sosa
- Ken Griffey
```


**Example: Mapping Scalars to Scalars (player statistics)**

```
hr:  65    # Home runs
avg: 0.278 # Batting average
rbi: 147   # Runs Batted In
```


**Example: Mapping Scalars to Sequences (ball clubs in each league)**

```
american:
- Boston Red Sox
- Detroit Tigers
- New York Yankees
national:
- New York Mets
- Chicago Cubs
- Atlanta Braves
```


**Example: Sequence of Mappings (players' statistics)**

```
- name: Mark McGwire
  hr:   65
  avg:  0.278
- name: Sammy Sosa
  hr:   63
  avg:  0.288
```

AYML also allows flow styles that use explicit indicators to denote scope.
A flow sequence is written as a comma-separated list within square brackets `[]`.
A flow mapping uses curly braces.

**Example: Sequence of Sequences**

```
- [name, hr, avg]
- [Mark McGwire, 65, 0.278]
- [Sammy Sosa, 63, 0.288]
```

**Example: Mapping of Mappings**

```
Mark McGwire: {hr: 65, avg: 0.278}
Sammy Sosa: {
  hr: 63,
  avg: 0.288,
}
```

**Example: Document with Two Comments**

```
hr: # 1998 hr ranking
- Mark McGwire
- Sammy Sosa
# 1998 rbi ranking
rbi:
- Sammy Sosa
- Ken Griffey
```

**Example: Compact Nested Mapping**

```
# Products purchased
- item: Super Hoop
  quantity: 1
- item: Basketball
  quantity: 4
- item: Big Shoes
  quantity: 1
```

### Primitives

Supported primitives are `bool`, `int`, `float`, `str`, and `null`.

**Boolean**

A boolean is either `true` or `false`, with no quotes.

```
is_good: true
is_bad: false
```

**Integer**

An AYML decoder must support 64-bit integers

```
decimal: 12345
binary: 0b10101010
octal: 0o14
hexadecimal: 0xC
```

**Float**

An AYML decoder must support 64-bit floating point numbers, including `inf` and `nan`:

```
canonical: 1.23015e+3
exponential: 12.3015e+02
fixed: 1230.15
negative infinity: -inf
not a number: nan
```

**Strings**

There are three kinds of strings: bare, single-quoted, and double-quoted.

```
unicode: "Sosa did fine.\u263A"
control: "\b1998\t1999\t2000\n"
hex esc: "\x0d\x0a is \r\n"

single: '"Howdy!" he cried.'
quoted: ' # Not a ''comment''.'
tie-fighter: '|\-*-/|'
```

Multi-line strings are allowed. Every newline in a string is preserved unless the line ends with `\`, in which case the next line is folded directly into the current line. A multi-line string that starts with `|` will allow quotes.

```
plain:
  This unquoted scalar
  spans many lines.

double-quoted: "So does this
  quoted scalar.\n"

single-quoted: 'And so
  does this'

one-line: This is where \
  you might think there is a line break \
  but there is no line break here.

bar-prefix: |
  You can add "quotes" and 'quotes' here.
  Anything that might cause a problem if the `|` was not present.
```

**Null**

The null value is `null` with no quotes.

```
optional field: null
```

#### Comments

A comment is a form of documentation that may be associated with a particular AYML element.
By default, all comments are ignored during deserialization.
A deserializer may decide to allow comments; if so, a comment may be associated to:

* the node below it, if the comment is on its own line
* the node it follows, if the comment follows a node

Multi-line comments are allowed.

**Example: Top and Side Comments**

```
# Network connection rules
# Use these rules to allow socket connections
network:
  rules:
    - host: github.com
      ports:
      - 22 # Git (SSH)
      - 443 # Site
```

A comment must not appear inside a scalar, but it may be associated with a scalar inside of a collection.
Any comment atop or to the side of a line in a multi-line string is considered to be part of that multi-line string.

**Example: Multi Line String Non-Comments**

```
code:
  # thread-safe work
  mutex.lock()
  do_work()
  mutex.unlock() # done
```

Comments are allowed as an extension to the object model because they are often critical to
understanding the purpose of a configuration block, and operations like formatting and linting
should be supported without destroying of that understanding.

### Full Length Example

Below are two full-length examples of AYML.
The first is a sample invoice; the second is a sample log file.

**Example: Invoice**

```
invoice: 34843
date: 2001-01-23
bill-to:
  given: Chris
  family: Dumars
  address:
    lines:
      458 Walkman Dr.
      Suite #292
    city    : Royal Oak
    state   : MI
    postal  : 48046
ship-to: null
product:
- sku         : BL394D
  quantity    : 4
  description : Basketball
  price       : 450.00
- sku         : BL4438H
  quantity    : 1
  description : Super Hoop
  price       : 2392.00
tax  : 251.42
total: 4443.52
comments:
  Late afternoon is best.
  Backup contact is Nancy
  Billsmer @ 338-4338.
```

**Example: Log File**

```
Date: 2001-11-23T15:03:17-5:00
User: ed
Fatal: Unknown variable "bar"
Stack:
- file: TopClass.py
  line: 23
  code: |
    x = MoreObject("345\n")
- file: MoreClass.py
  line: 58
  code: |
    foo = bar
```

## BNF Grammar

This section defines the BNF grammar for AYML.
Whenever possible, basic structures are specified before the more complex
structures using them in a "bottom up" fashion.

Each rule is accompanied by one or more examples.

### Production Syntax

Productions are defined using the syntax `production-name ::= term`, where a
term is either:

An atomic term:

* A quoted string (`"abc"`), which matches that concatenation of characters. A
  single character is usually written with single quotes (`'a'`).
* A hexadecimal number (`x0A`), which matches the character at that Unicode
  code point.
* A range of hexadecimal numbers (`[x20-x7E]`), which matches any character
  whose Unicode code point is within that range.
* The name of a production (`c-printable`), which matches that production.

A lookaround:

* `[ lookahead = term ]`, which matches the empty string if `term` would match.
* `[ lookahead ≠ term ]`, which matches the empty string if `term` would not
  match.
* `[ lookbehind = term ]`, which matches the empty string if `term` would match
  beginning at any prior point on the line and ending at the current position.

A special production:

* `<start-of-line>`, which matches the empty string at the beginning of a line.
* `<end-of-input>`, matches the empty string at the end of the input.
* `<empty>`, which (always) matches the empty string.

A parenthesized term matches its contents.

A concatenation is `term-one term-two`, which matches `term-one` followed by `term-two`.

A alternation is `term-one | term-two`, which matches the `term-one` if possible, or
`term-two` otherwise.

A quantified term:

* `term?`, which matches `(term | <empty>)`.
* `term*`, which matches `(term term* | <empty>)`.
* `term+`, which matches `(term term*)`.

> Note: Quantified terms are always greedy.

The order of precedence is parenthesization, then quantification, then concatenation, then alternation.

Some lines in a production definition may have a comment like:

```
production-a ::=
  production-b      # clarifying comment
```

These comments are meant to be informative only.
For instance a comment that says `# not followed by non-ws char` just means
that you should be aware that actual production rules will behave as described
even though it might not be obvious from the content of that particular
production alone.


### Production Parameters

Some productions have parameters in parentheses after the name, such as `c-double-quoted(n)`.
A parameterized production is shorthand for a (infinite) series of productions,
each with a fixed value for each parameter.

The parameters are as follows:

* `n` : The current indentation level. May be any natural number, including zero.
* `c`: Context to distinguish between block and flow parsing. May be one of:
  * `BLOCK` -- inside a block collection
  * `FLOW` -- inside a flow collection


### Production Naming Conventions

To make it easier to follow production combinations, production names use a
prefix-style naming convention.

* `e-` : A production matching no characters.
* `c-` : A production starting and ending with a special character.
* `b-` : A production matching a single line break.
* `nb-` : A production starting and ending with a non-break character.
* `s-` : A production starting and ending with a white space character.
* `ns-` : A production starting and ending with a non-space character.
* `l-` : A production matching complete line(s).


# Chapter: Character Productions

## Character Set

To ensure readability, AYML uses only the printable subset of the Unicode character set.

The allowed character range excludes the C0 control block `x00-x1F` 
(except for TAB `x09`, LF `x0A`, and CR `x0D` which are allowed),
DEL `x7F`, the C1 control block `x80-x9F` (except for NEL `x85` which is allowed), 
the surrogate block `xD800-xDFFF`, `xFFFE`, and `xFFFF`.
```
c-printable ::=
                         # 8 bit
    x09                  # Tab (\t)
  | x0A                  # Line feed (LF \n)
  | x0D                  # Carriage return (CR \r)
  | [x20-x7E]            # Printable ASCII
                         # 16 bit
  | x85                  # Next Line (NEL)
  | [xA0-xD7FF]          # Basic Multilingual Plane (BMP)
  | [xE000-xFFFD]        # Additional Unicode Areas
  | [x010000-x10FFFF]    # 32 bit
```


## Character Encoding

All characters in this specification are Unicode code points.
AYML files MUST be encoded in UTF-8. No other encodings are supported.

A byte order mark (BOM) MUST NOT appear in an AYML file.

## Line Break Characters

```
b-line-feed ::= x0A
b-carriage-return ::= x0D
b-char ::=
    b-line-feed
  | b-carriage-return
```

All other characters, including the form feed (`x0C`), are considered to be
non-break characters.

```
nb-char ::= c-printable - b-char
```

Line breaks are interpreted differently by different systems and have multiple
widely used formats.

```
b-break ::=
  ( b-carriage-return b-line-feed )    # CR LF
  | b-carriage-return                  # CR
  | b-line-feed                        # LF
```

Line breaks inside scalar content MUST be _normalized_ by the AYML processor.
Each such line break MUST be parsed into a single line feed character.

```
b-as-line-feed ::= b-break
```

Outside scalar content, AYML allows any line break to be used to terminate lines.

```
b-non-content ::= b-break
```


## White Space Characters

AYML recognizes two _white space_ characters: _space_ and _tab_.

```
s-space ::= x20
s-tab ::= x09
s-white ::= s-space | s-tab
```

The rest of the printable non-break characters are considered to be
non-space characters.

```
ns-char ::= nb-char - s-white
```


## Miscellaneous Characters

A decimal digit for numbers:

```
ns-dec-digit ::= [x30-x39]             # 0-9
```

A hexadecimal digit for escape sequences:

```
ns-hex-digit ::=
    ns-dec-digit         # 0-9
  | [x41-x46]            # A-F
  | [x61-x66]            # a-f
```

An octal digit:

```
ns-oct-digit ::= [x30-x37]             # 0-7
```

A binary digit:

```
ns-bin-digit ::= '0' | '1'
```

ASCII letter (alphabetic) characters:

```
ns-ascii-letter ::=
    [x41-x5A]            # A-Z
  | [x61-x7A]            # a-z
```


## Indicator Characters

_Indicators_ are characters that have special semantics.

"`-`" (`x2D`, hyphen) denotes a block sequence entry.

```
c-sequence-entry ::= '-'
```

"`:`" (`x3A`, colon) denotes a mapping value.

```
c-mapping-value ::= ':'
```

"`,`" (`x2C`, comma) separates entries in a flow collection.

```
c-collect-entry ::= ','
```

"`[`" (`x5B`, left bracket) starts a flow sequence.

```
c-sequence-start ::= '['
```

"`]`" (`x5D`, right bracket) ends a flow sequence.

```
c-sequence-end ::= ']'
```

"`{`" (`x7B`, left brace) starts a flow mapping.

```
c-mapping-start ::= '{'
```

"`}`" (`x7D`, right brace) ends a flow mapping.

```
c-mapping-end ::= '}'
```

"`#`" (`x23`, octothorpe) denotes a comment.

```
c-comment ::= '#'
```

"`'`" (`x27`, apostrophe) surrounds a single-quoted scalar.

```
c-single-quote ::= "'"
```

"`"`" (`x22`, double quote) surrounds a double-quoted scalar.

```
c-double-quote ::= '"'
```

"`\`" (`x5C`, backslash) begins an escape sequence in double-quoted scalars,
and denotes line folding in bare scalars.

```
c-escape ::= '\'
```

"`|`" (`x7C`, vertical bar) denotes a bar-prefix string.

```
c-bar-prefix ::= '|'
```

> **Note:** AYML does not use the following YAML indicators: `?` (explicit
> mapping key), `&` (anchor), `*` (alias), `!` (tag), `>` (folded scalar),
> `%` (directive, except `%AYML`), `@`, or `` ` `` (reserved).

The union of all indicator characters:

```
c-indicator ::=
    c-sequence-entry     # '-'
  | c-mapping-value      # ':'
  | c-collect-entry      # ','
  | c-sequence-start     # '['
  | c-sequence-end       # ']'
  | c-mapping-start      # '{'
  | c-mapping-end        # '}'
  | c-comment            # '#'
  | c-single-quote       # "'"
  | c-double-quote       # '"'
  | c-escape             # '\'
  | c-bar-prefix         # '|'
```

Flow indicators are the subset that denote structure in flow collections:

```
c-flow-indicator ::=
    c-collect-entry      # ','
  | c-sequence-start     # '['
  | c-sequence-end       # ']'
  | c-mapping-start      # '{'
  | c-mapping-end        # '}'
```


## Escape Sequences

All non-printable characters MUST be _escaped_.
Escape sequences are only interpreted in double-quoted scalars.
In all other scalar styles, the "`\`" character has no special meaning
(except for line folding in bare strings).

```
c-ns-esc-char ::=
    c-escape
    (
        '0'                   # Null (x00)
      | 'a'                   # Bell (x07)
      | 'b'                   # Backspace (x08)
      | 't' | x09             # Horizontal tab (x09)
      | 'n'                   # Line feed (x0A)
      | 'v'                   # Vertical tab (x0B)
      | 'f'                   # Form feed (x0C)
      | 'r'                   # Carriage return (x0D)
      | 'e'                   # Escape (x1B)
      | x20                   # Space (x20)
      | '"'                   # Double quote (x22)
      | '/'                   # Slash (x2F)
      | '\'                   # Backslash (x5C)
      | ( 'x' ns-hex-digit{2} )    # 8-bit Unicode
      | ( 'u' ns-hex-digit{4} )    # 16-bit Unicode
      | ( 'U' ns-hex-digit{8} )    # 32-bit Unicode
    )
```


# Chapter: Indentation and Separation

## Indentation

AYML uses spaces for indentation. Tabs MUST NOT be used for indentation.
The indentation level is tracked as a parameter `n` representing the number
of leading spaces.

```
s-indent(n) ::= s-space{n}
```

```
s-indent-less-than(n) ::=
    s-space{m}    # where m < n
```

```
s-indent-less-or-equal(n) ::=
    s-space{m}    # where m <= n
```

## Separation

A separation in line is one or more white space characters.

```
s-separate-in-line ::= s-white+
```


# Chapter: Comments

A comment is indicated by a `#` character. When a `#` appears preceded by
whitespace (or at the start of a line) in a non-scalar context, it begins a
comment that extends to the end of the line.

Inside multi-line scalars (bare strings and bar-prefix strings), `#` is
treated as literal content, not as a comment indicator.

```
c-nb-comment-text ::= c-comment nb-char*
```

An inline comment (trailing a node on the same line):

```
s-b-comment ::=
    ( s-separate-in-line c-nb-comment-text? )?
    b-break
```

A full-line comment:

```
l-comment ::=
    s-indent-less-or-equal(n)
    c-nb-comment-text
    b-break
```

Consecutive comment lines form a multi-line comment:

```
l-comment-block ::= l-comment+
```


# Chapter: Scalar Productions

## Null

```
c-null ::= "null"
```

**Example:**

```
optional field: null
```


## Boolean

```
c-bool ::= "true" | "false"
```

**Example:**

```
is_good: true
is_bad: false
```


## Integer

An AYML decoder MUST support 64-bit signed integers.

```
ns-integer ::=
    ( '-' | '+' )? ns-dec-digit+                    # Decimal
  | ( '-' | '+' )? "0b" ns-bin-digit+               # Binary
  | ( '-' | '+' )? "0o" ns-oct-digit+               # Octal
  | ( '-' | '+' )? "0x" ns-hex-digit+               # Hexadecimal
```

**Example:**

```
decimal: 12345
negative: -9876
binary: 0b10101010
octal: 0o14
hexadecimal: 0xC
```


## Float

An AYML decoder MUST support 64-bit (double precision) floating point numbers.

```
ns-float ::=
    ( '-' | '+' )? ns-dec-digit+ '.' ns-dec-digit*
        ( ( 'e' | 'E' ) ( '-' | '+' )? ns-dec-digit+ )?    # Fixed/exponential
  | ( '-' | '+' )? ns-dec-digit+
        ( 'e' | 'E' ) ( '-' | '+' )? ns-dec-digit+         # Pure exponential
  | ( '-' | '+' )? "inf"                                   # Infinity
  | "nan"                                                  # Not a number
```

**Example:**

```
canonical: 1.23015e+3
exponential: 12.3015e+02
fixed: 1230.15
negative infinity: -inf
not a number: nan
```


## Double-Quoted String

A double-quoted string is delimited by `"` characters and supports escape
sequences. It may span multiple lines; continuation lines are indented to at
least the current indentation level.

```
nb-double-char ::=
    c-ns-esc-char
  | ( nb-char - c-double-quote - c-escape )
```

```
nb-double-one-line ::= nb-double-char*
```

```
s-double-next-line(n) ::=
    b-break
    s-indent(n)
    nb-double-one-line
```

```
c-double-quoted(n) ::=
    c-double-quote
    nb-double-one-line
    s-double-next-line(n)*
    c-double-quote
```

**Example:**

```
unicode: "Sosa did fine.\u263A"
control: "\b1998\t1999\t2000\n"
multi-line: "So does this
  quoted scalar.\n"
```


## Single-Quoted String

A single-quoted string is delimited by `'` characters. Single quotes within
the string are escaped by doubling them (`''`). No other escape sequences are
recognized. It may span multiple lines.

```
nb-single-char ::=
    "''"                                  # Escaped single quote
  | ( nb-char - c-single-quote )
```

```
nb-single-one-line ::= nb-single-char*
```

```
s-single-next-line(n) ::=
    b-break
    s-indent(n)
    nb-single-one-line
```

```
c-single-quoted(n) ::=
    c-single-quote
    nb-single-one-line
    s-single-next-line(n)*
    c-single-quote
```

**Example:**

```
single: '"Howdy!" he cried.'
quoted: ' # Not a ''comment''.'
multi-line: 'And so
  does this'
```


## Bare String

A bare (unquoted) string has no delimiters. Its content is determined by
context. Bare strings undergo scalar resolution — if the content matches
`null`, `true`, `false`, an integer, or a float pattern, it is parsed as
that type rather than as a string.

On the **first line** of a bare value (the line containing or following the
mapping value indicator `:`), a `#` preceded by whitespace starts a comment
and terminates the scalar.

On **continuation lines** (indented beyond the parent indentation), all
content including `#` characters is literal string content, per the
multi-line string comment rule.

A `\` at the end of a line folds the next line into the current one — the
line break and leading whitespace of the continuation line are removed.

In a **flow context**, bare strings are additionally terminated by flow
indicators (`,`, `]`, `}`).

```
ns-plain-first-char(c) ::=
    ( ns-char - c-indicator )
  | ( ( '-' | ':' ) [ lookahead = ns-char ] )
```

```
ns-plain-char(BLOCK) ::=
    ( ns-char - c-comment - c-escape )
  | ( ':' [ lookahead = ns-char ] )          # ':' not followed by space
  | ( [ lookbehind = ns-char ] '#' )         # '#' preceded by non-space
```

```
ns-plain-char(FLOW) ::=
    ns-plain-char(BLOCK)
  - c-flow-indicator
```

Single-line bare content:

```
ns-bare-one-line(c) ::=
    ns-plain-first-char(c)
    ns-plain-char(c)*
```

A continuation line within a multi-line bare string. All content is literal,
including `#` characters:

```
s-bare-continuation-line(n) ::=
    b-break
    s-indent(n)
    nb-char+
```

A folded line — `\` at end of line joins with the next:

```
s-bare-fold(n) ::=
    c-escape b-break
    s-indent(n)
```

> **Flag:** The `\` line-folding mechanism is only demonstrated for bare
> strings in the Language Overview. It is unclear whether `\` folding also
> applies in bar-prefix strings or single-quoted strings. This grammar
> restricts line folding to bare strings only.

```
ns-bare-string(n,c) ::=
    ns-bare-one-line(c)
    ( s-bare-fold(n) ns-bare-one-line(c)
    | s-bare-continuation-line(n)
    )*
```

**Example:**

```
plain:
  This unquoted scalar
  spans many lines.

one-line: This is where \
  you might think there is a line break \
  but there is no line break here.
```


## Bar-Prefix String

A bar-prefix string begins with `|` on the mapping value line, followed by
indented content on subsequent lines. All content — including quotes, `#`
characters, and `\` — is literal.

```
l-bar-content-line(n) ::=
    s-indent(n) nb-char+
```

```
ns-bar-prefix-string(n) ::=
    c-bar-prefix
    b-break
    l-bar-content-line(n)
    ( b-break l-bar-content-line(n) )*
```

> **Flag:** Trailing newline handling is unspecified. YAML's block scalars
> have "chomping" (strip/clip/keep) for trailing newlines. The Language
> Overview does not address this. Does the final line of a bar-prefix string
> include a trailing newline or not?

**Example:**

```
bar-prefix: |
  You can add "quotes" and 'quotes' here.
  Anything that might cause a problem if the `|` was not present.
```


## Scalar Resolution

When parsing an unquoted (bare) scalar value, the parser SHOULD attempt to
match in this order:

1. `c-null`
2. `c-bool`
3. `ns-integer`
4. `ns-float`
5. `ns-bare-string`

A quoted string (`c-double-quoted`, `c-single-quoted`) or bar-prefix string
(`ns-bar-prefix-string`) is always a string regardless of its content.

### Reserved Words

The following bare words are interpreted as non-string types and MUST be
quoted to be used as strings:

| Word    | Type  |
| ------- | ----- |
| `null`  | null  |
| `true`  | bool  |
| `false` | bool  |
| `inf`   | float |
| `+inf`  | float |
| `-inf`  | float |
| `nan`   | float |

There are no other implicit type conversions.
Words like `yes`, `no`, `on`, `off`, and date-like strings
such as `2001-01-23` are parsed as strings.

```
ns-scalar(n,c) ::=
    c-double-quoted(n)
  | c-single-quoted(n)
  | ns-bar-prefix-string(n)
  | c-null
  | c-bool
  | ns-integer
  | ns-float
  | ns-bare-string(n,c)
```


# Chapter: Collection Productions

## Block Sequence

A block sequence is a series of entries, each indicated by a `- ` (dash
followed by a space). Each entry begins at the same indentation level.

```
l-block-seq-entry(n) ::=
    s-indent(n)
    c-sequence-entry s-white
    ns-block-indented-node(n+1,BLOCK)
```

```
l-block-sequence(n) ::=
    l-block-seq-entry(n)
    ( b-break l-block-seq-entry(n) )*
```

> **Flag:** In the "Mapping Scalars to Sequences" example, the sequence
> entries are at the *same* indentation level as the parent mapping key:
> ```
> american:
> - Boston Red Sox
> ```
> This is allowed in YAML (the `-` indicator provides an implicit extra
> level of indentation). The grammar handles this through `ns-block-node`
> allowing a block sequence at the current level when the sequence is the
> value of a mapping entry.

**Example:**

```
- Mark McGwire
- Sammy Sosa
- Ken Griffey
```


## Block Mapping

A block mapping is a series of key/value pairs. Keys are single-line scalars.
A mapping MUST NOT have duplicate keys (this is a semantic constraint enforced
by the parser, not expressible in the grammar).

```
ns-block-mapping-key(n) ::=
    ns-bare-one-line(BLOCK)
  | c-double-quoted(n)
  | c-single-quoted(n)
```

> **Flag:** The Language Overview only shows single-line mapping keys. This
> grammar restricts keys to single-line scalars. Multi-line keys are not
> supported.

A mapping entry. The value may appear on the same line or on subsequent
indented lines:

```
l-block-mapping-entry(n) ::=
    s-indent(n)
    ns-block-mapping-key(n)
    s-white*
    c-mapping-value
    (
        s-white+ ns-block-node(n+1,BLOCK)       # Value on same line
      | s-b-comment                               # Value on next line(s)
        ns-block-node(n+1,BLOCK)
    )
```

```
l-block-mapping(n) ::=
    l-block-mapping-entry(n)
    ( b-break l-block-mapping-entry(n) )*
```

**Example:**

```
hr:  65
avg: 0.278
rbi: 147
```


## Flow Sequence

A flow sequence is a comma-separated list within square brackets `[]`.
Trailing commas are allowed. Flow sequences may span multiple lines.

```
ns-flow-seq-entry(n) ::= ns-flow-node(n,FLOW)
```

```
c-flow-sequence(n) ::=
    c-sequence-start
    s-white*
    (
        ns-flow-seq-entry(n)
        ( s-white* c-collect-entry s-white* ns-flow-seq-entry(n) )*
        ( s-white* c-collect-entry )?          # Optional trailing comma
    )?
    s-white*
    c-sequence-end
```

**Example:**

```
- [name, hr, avg]
- [Mark McGwire, 65, 0.278]
```


## Flow Mapping

A flow mapping is a comma-separated list of key/value pairs within curly
braces `{}`. Trailing commas are allowed. Flow mappings may span multiple
lines.

```
ns-flow-mapping-entry(n) ::=
    ns-scalar(n,FLOW)
    s-white*
    c-mapping-value
    s-white*
    ns-flow-node(n,FLOW)
```

```
c-flow-mapping(n) ::=
    c-mapping-start
    s-white*
    (
        ns-flow-mapping-entry(n)
        ( s-white* c-collect-entry s-white* ns-flow-mapping-entry(n) )*
        ( s-white* c-collect-entry )?          # Optional trailing comma
    )?
    s-white*
    c-mapping-end
```

**Example:**

```
Mark McGwire: {hr: 65, avg: 0.278}
Sammy Sosa: {
  hr: 63,
  avg: 0.288,
}
```


# Chapter: Node and Document Productions

## Nodes

A flow node is a scalar or flow collection:

```
ns-flow-node(n,c) ::=
    ns-scalar(n,c)
  | c-flow-sequence(n)
  | c-flow-mapping(n)
```

A block-indented node accounts for the content that appears after a block
sequence entry indicator (`- `):

```
ns-block-indented-node(n,c) ::=
    ns-flow-node(n,c)
  | l-block-sequence(n)
  | l-block-mapping(n)
```

A block node is any node that can appear as a block mapping value:

```
ns-block-node(n,c) ::=
    ns-flow-node(n,c)
  | l-block-sequence(n)
  | l-block-mapping(n)
```

> **Flag:** The interaction between block mappings and block sequences as
> values needs care. When a mapping value is a sequence, the examples show
> the sequence at the *parent* indentation level (e.g., `american:\n- Boston`).
> This means `ns-block-node(n+1,BLOCK)` for a mapping value must be able to
> match `l-block-sequence(n)` — the sequence entries start at the parent's
> level because the `-` provides implicit indentation. This may require an
> adjustment to the indentation parameter in the mapping entry production.


## AYML Directive

An AYML document MAY optionally begin with a version directive:

```
l-ayml-directive ::=
    "% AYML 1.0" b-break
```


## Document

An AYML file contains exactly one document. A document is an optional directive, optional leading comments, and a single root node:

```
l-ayml-document ::=
    l-ayml-directive?
    l-comment-block?
    ns-block-node(0,BLOCK)
    b-break?
    <end-of-input>
```

**Example:**

```
%AYML 1.0
# Server configuration
host: localhost
port: 8080
```
