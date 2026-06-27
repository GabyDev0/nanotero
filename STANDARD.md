# Tero v0.1

This document defines the official syntax, parsing rules, and expected behavior for **.tero** configuration files.

## 1. Type System and Values

**Tero** classifies its components into two categories: **Primitive Values** (basic immutable data) and **Data Structures** (complex containers).

### 1.1 Primitive Values

| Type | Syntax / Representation | Technical Description |
| --- | --- | --- |
| **Integer** | `42`, `-1024` | Natively stored as a 64-bit signed integer (`i64`). |
| **Float** | `3.1416`, `-0.005` | Stored as a double-precision floating-point number (`f64`). |
| **Boolean** | `true` or `false` | Traditional logical values used for flow control or flags. |
| **String** | `"text"` | A sequence of UTF-8 characters strictly wrapped in double quotes. |
| **Nil** | `nil` | Represents the intentional absence of any value or an uninitialized state. |

#### Special Numeric Constants

Tero recognizes two keywords inherited from the IEEE 754 standard for exceptional mathematical operations:

* `NaN`: (*Not a Number*) The result of an invalid or undefined mathematical operation.
* `Infinity`: Represents a numeric value that overflows the limits of the `f64` type.

---

### 1.2 Data Structures (Objects and Collections)

Data structures allow grouping primitive values or other sub-structures, supporting deep nesting.

#### Objects `{}`

Ordered collections of key-value pairs. Keys within an object must be unique at their specific hierarchy level.

```text
configuration: {
    active: true,
    attempts: 3
}

```

#### Arrays `[]`

Indexed and ordered lists of values. They can natively contain mixed data types.

```text
allowed_ports: [80, 443, 8080],
server_statuses: [true, "standby", nil]

```

## 2. Version Declaration

**Tero** allows specifying which version of the standard the configuration file was designed under.

### 2.1 Structure and Rules

* **Placement:** If included, it must be declared **strictly on the first line** of the file.
* **Syntax:** It consists of the keyword `Tero` (respecting the initial capital letter), followed by exactly one whitespace and a numeric constant (`Float`). It must terminate immediately with a newline (`\n`).
* **Optionality:** The version declaration is **optional**. If omitted, the *parser* will default to the most recent version supported by the client library.

### Usage Example

```text
Tero 0.1
Users [{
    Name: "GabyDev0"
}]

```

## 3. Data Structures: Objects `{}`

An object is an indexed and ordered collection of key-value pairs. Keys within the same object must be unique at their specific hierarchy level.

### 3.1 Key Identifiers

Keys are declared without quotes (plain text style). They must strictly begin with a letter or an underscore (`_`) and can only contain alphanumeric characters (`[a-zA-Z0-9_]`).

### 3.2 Assignment Separator

* **General Rule:** A colon (`:`) is strictly used to separate a key from its value.
* **Structural Exception:** If the assigned value is a complex data structure (**Object** or **Array**), the use of the colon is **completely optional**.

### 3.3 Element Separator and Commas

* **Delimiter:** Key-value pairs within an object are separated by a comma (`,`) **or** by a newline (`\n`).
* **Trailing Comma:** A comma after the last element of an object is optional and completely valid.

### Syntax Examples

```text
# Variant 1: Single-line using commas and optional assignment in nested object
user { name: "GabyDev0", role: "admin" }

# Variant 2: Block style using newlines (no commas or colons)
server {
    ip: "127.0.0.1"
    port: 8080
    
    # The format allows mixing the omission of colons in structures
    modules [ "auth", "api", "db" ]
}

```

## 4. Data Structures: Arrays `[]`

An array is an ordered and indexed list of values.

### 4.1 Syntax Rules

* **Multi-type:** An array can natively contain any combination of data types (integers, floats, strings, booleans, objects, `nil`, or other arrays).
* **Element Separator:** Just like objects, elements within an array can be separated by a comma (`,`) **or** by a newline (`\n`).
* **Trailing Comma:** Allowed and optional after the last element.

### Syntax Examples

```text
# Variant 1: Single-line separated by commas
ports: [80, 443, 8080]

# Variant 2: Multi-line using newlines (no commas)
servers [
    "api.gabydev0.com"
    "auth.gabydev0.com"
]

# Variant 3: Mixed array with a trailing comma
data: [
    "production",
    true,
    100,
]

```

## 5. Comments

**Tero** allows adding annotations and explanatory text within the file using the hash or hashtag character (`#`).

### 5.1 Parsing Rules

* **Ignored by the Parser:** Any text found to the right of a `#` character is completely ignored by the reading library and has zero impact on the final object in memory.
* **Placement:** Comments can occupy an entire line or be placed at the end of a line of code (*inline* comments).

### Syntax Examples

```text
# This is a full-line comment documenting the block
server {
    port: 8080 # Inline comment: Default port for the API
    secure: true
}
```
