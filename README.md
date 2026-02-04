# TinyC Language

A simple C-style programming language implemented in Rust with a tree-walk interpreter.

## Table of Contents

- [Getting Started](#getting-started)
- [Language Features](#language-features)
  - [Types](#types)
  - [Variables](#variables)
  - [Functions](#functions)
  - [Control Flow](#control-flow)
  - [Operators](#operators)
  - [Comments](#comments)
  - [String Literals](#string-literals)
- [Standard Library](#standard-library)
  - [Console I/O](#console-io)
  - [File I/O](#file-io)
  - [String Formatting](#string-formatting)
  - [File System Operations](#file-system-operations)
- [Examples](#examples)

## Getting Started

### Running a Program

```bash
cargo run -- <filename>.tc
```

### Example Hello World

```c
printf("Hello, World!\n");
```

## Language Features

### Types

Currently, the language supports a single type keyword:

- `int` - Used for all values (integers, strings, booleans, functions, files)

### Variables

Variables must be declared with the `int` keyword and initialized with a value:

```c
int x = 42;
int name = "Alice";
int result = x + 10;
```

**Note:** Variables must be initialized at declaration; uninitialized declarations are not supported.

### Functions

Functions are declared using the `int` keyword followed by the function name, parameters, and body:

```c
// Function with no parameters
int greet() {
    printf("Hello!\n");
    return 0;
}

// Function with parameters
int add(int a, int b) {
    return a + b;
}

// Calling functions
int result = add(5, 10);
greet();
```

**Note:** All parameters must be declared with the `int` keyword.

### Control Flow

#### If Statements

```c
int x = 10;
if (x > 5) {
    printf("x is greater than 5\n");
}

// If-else
if (x == 10) {
    printf("x is 10\n");
} else {
    printf("x is not 10\n");
}
```

#### While Loops

```c
int i = 0;
while (i < 5) {
    printf("i = %d\n", i);
    i = i + 1;
}
```

### Operators

#### Arithmetic Operators

- `+` - Addition
- `-` - Subtraction
- `*` - Multiplication
- `/` - Division
- `-` - Unary negation

#### Comparison Operators

- `==` - Equal to
- `!=` - Not equal to
- `<` - Less than
- `>` - Greater than

#### Example

```c
int a = 10;
int b = 20;
int sum = a + b;
int product = a * b;
int isEqual = a == b;
int isLess = a < b;
```

### Comments

Single-line comments start with `//`:

```c
// This is a comment
int x = 42; // Comment after code
```

### String Literals

Strings are enclosed in double quotes and support escape sequences:

```c
int message = "Hello, World!\n";
int path = "C:\\Users\\name";
int quote = "She said \"Hello\"";
```

**Supported escape sequences:**

- `\n` - Newline
- `\t` - Tab
- `\r` - Carriage return
- `\"` - Double quote
- `\\` - Backslash

### Built-in Constants

- `null` - Represents null/empty values
- `true` - Boolean true
- `false` - Boolean false

## Standard Library

### Console I/O

#### `printf(format, ...)`

Prints formatted output to stdout.

**Format specifiers:**

- `%s` - String
- `%d` - Integer
- `%%` - Literal %

```c
printf("Hello, %s!\n", "World");
printf("Number: %d\n", 42);
printf("Progress: 50%%\n");
```

#### `puts(string)`

Prints a string to stdout followed by a newline.

```c
puts("Hello, World!");
```

#### `putchar(char)`

Prints the first character of a string to stdout.

```c
putchar("H"); // Prints "H"
```

#### `getchar()`

Reads a single character from stdin.

```c
int c = getchar();
printf("You entered: %s\n", c);
```

### File I/O

#### `fopen(path, mode)`

Opens a file and returns a file handle.

**Modes:**

- `"r"` - Read mode
- `"w"` - Write mode

```c
int file = fopen("data.txt", "w");
if (file == null) {
    printf("Failed to open file\n");
}
```

#### `fclose(file)`

Closes a file handle.

```c
fclose(file);
```

#### `fputs(string, file)`

Writes a string to a file.

```c
fputs("Hello, File!\n", file);
```

#### `fputc(char, file)`

Writes a character to a file.

```c
fputc("A", file);
```

#### `fgets(file)`

Reads a line from a file (up to and including `\n`).

```c
int line = fgets(file);
if (line != null) {
    printf("Read: %s", line);
}
```

#### `fgetc(file)`

Reads a single character from a file.

```c
int c = fgetc(file);
if (c != null) {
    printf("Char: %s\n", c);
}
```

#### `fprintf(file, format, ...)`

Writes formatted output to a file.

```c
fprintf(file, "Name: %s, Age: %d\n", "Alice", 30);
```

#### `feof(file)`

Returns `true` if the end-of-file indicator is set.

```c
while (feof(file) == false) {
    int c = fgetc(file);
}
```

#### `ferror(file)`

Returns `true` if an error occurred during file operations.

```c
if (ferror(file) == true) {
    printf("File error occurred\n");
}
```

#### `ftell(file)`

Returns the current file position.

```c
int pos = ftell(file);
printf("Position: %d\n", pos);
```

#### `fseek(file, offset, whence)`

Moves the file position.

**Whence values:**

- `0` - Seek from start of file (SEEK_SET)
- `1` - Seek from current position (SEEK_CUR)
- `2` - Seek from end of file (SEEK_END)

```c
fseek(file, 0, 0);     // Rewind to start
fseek(file, 10, 1);    // Move 10 bytes forward
fseek(file, -5, 2);    // Move 5 bytes before end
```

#### `rewind(file)`

Resets the file position to the beginning and clears error flags.

```c
rewind(file);
```

#### `getc(file)`

Alias for `fgetc(file)`.

#### `putc(char, file)`

Alias for `fputc(char, file)`.

### String Formatting

#### `sprintf(format, ...)`

Returns a formatted string.

```c
int result = sprintf("User: %s, Score: %d", "Bob", 100);
printf("%s\n", result);
```

### File System Operations

#### `rename(oldpath, newpath)`

Renames a file.

```c
rename("old.txt", "new.txt");
```

#### `remove(path)`

Deletes a file.

```c
remove("temp.txt");
```

## Examples

### Fibonacci Sequence

```c
int fib(int n) {
    if (n < 2) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

int main() {
    int result = fib(10);
    printf("fib(10) = %d\n", result);
    return 0;
}

main();
```

### File I/O Example

```c
int main() {
    // Write to file
    int file = fopen("output.txt", "w");
    if (file == null) {
        printf("Failed to open file\n");
        return 1;
    }

    fprintf(file, "Hello, %s!\n", "File");
    fputs("This is a test.\n", file);
    fclose(file);

    // Read from file
    int file2 = fopen("output.txt", "r");
    if (file2 == null) {
        printf("Failed to open file\n");
        return 1;
    }

    while (feof(file2) == false) {
        int line = fgets(file2);
        if (line != null) {
            printf("%s", line);
        }
    }

    fclose(file2);
    return 0;
}

main();
```

### Loop Example

```c
int main() {
    int sum = 0;
    int i = 1;

    while (i < 11) {
        sum = sum + i;
        i = i + 1;
    }

    printf("Sum of 1 to 10: %d\n", sum);
    return 0;
}

main();
```

### Conditional Example

```c
int checkNumber(int n) {
    if (n > 0) {
        printf("%d is positive\n", n);
    } else {
        if (n < 0) {
            printf("%d is negative\n", n);
        } else {
            printf("%d is zero\n", n);
        }
    }
    return 0;
}

int main() {
    checkNumber(10);
    checkNumber(-5);
    checkNumber(0);
    return 0;
}

main();
```

## Limitations (most of these will be removed in the future)

- No arrays or pointers
- No structs or user-defined types
- No `scanf` family (input formatting requires pass-by-reference)
- No `for` loops (use `while` instead)
- No logical operators (`&&`, `||`, `!`) - use nested conditions
- Single type system (everything is `int`, types are determined at runtime)
- No multi-line comments

## Building and Running

```bash
# Build the interpreter
cargo build --release

# Run a program
cargo run -- myprogram.tc

# Or use the compiled binary
./target/release/tcc myprogram.tc
```

## License

This project is licensed under the [MIT LICENSE](LICENSE)
