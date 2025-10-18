// bucket_brigade_preallocated_chunking.rs
use std::{
    env,
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::PathBuf,
};

/*


# Bucket Brigade pre-allocated processing of stdin

POC programs to read input iteratively (in chunks)
over and including multiple newlines
(given the edge case of the leftover pizza problem)
using a pre-allocated buffer.

## This scope includes:
- must use pre-allocated buffer
- must not use heap
- must not use read_line()
- must not halt at first newline
- stdin input can/will contain multiple newlines (e.g. cut and past multiple lines)
- must handle all newlines *in the input
  -- user must enter 'Enter' key if multiline does not end in newline
- must never load all of input into memory at once, only one chunk

# Requires exit signal
A kind of ~halting problem: we cannot know the state of stdin
Use a specific exit command: -q -n -v
when changing mode or quitting, etc.
stdin is a process that has no end to predict.

This:
```
if bytes_read == 0 {
    println!("bytes_read == 0");
    break;
}
```
is never triggered (or would never be triffered).
stdin requests more input.
The program never "finishes."

# The Leftover Pizza Problem

stdin sometimes has "left over pizza,"
a problem where (for whatever reasons)
a newline does not appear at the end text entry,
and is in stdin,
so that content is stuck
until there is another addition.

Because of the riddle of ~blocked stdin
and not being able to tell when it is empty,
or being able to guarantee that it contains
input followed by the \n newline that it needs,
and the side effect of it sometimes asking
for more text before it is empty:

If there is multiline input that does not end in a newline,
the user needs to press the Enter key one more time,
even though the multi-line input did go into stdin,
e.g. there is no lingering text in the terminal waitng
for the user to hit enter.

 */

/// Reads user input from stdin in pre-allocated chunks and displays to terminal.
///
/// # Purpose
/// Demonstrates bucket brigade pattern for processing stdin
/// without loading all input into heap memory.
/// This is a test/demonstration function showing chunk-by-chunk
/// processing with diagnostic output.
/// You can experiment with chunk sizes (1 byte won't accept "-q")
///
/// # Memory Safety
/// - Uses pre-allocated 64-byte buffer (no heap allocation for input processing)
/// - Never loads entire stdin stream into memory
/// - Processes input chunk-by-chunk using bucket brigade pattern
/// - Buffer is cleared between reads to prevent data leakage
///
/// # Interactive Behavior
/// - Prompts user for input
/// - Displays diagnostic information for each chunk read
/// - Shows both raw bytes and UTF-8 text representation
/// - Marks when newlines are detected in input
/// - Continues reading until user enters exit command
///
/// # The "Leftover Pizza Problem"
/// stdin may retain incomplete input when multi-line text doesn't end with newline.
/// If pasting multiple lines without trailing newline:
/// - Content goes into stdin correctly
/// - User must press Enter to flush the final chunk
/// - This is a characteristic (a 'feature!') of stdin blocking behavior, not a 'bug'
/// (aka...this is a horrible bug in rust or posix, but not in this program)
///
/// # Exit Conditions
/// User must enter one of these commands on its own line: (or exit with ctrl+c)
/// - `-q` to quit
/// - `--quite` to quit (note: typo preserved for backwards compatibility)
///
/// stdin is a process with no natural end, so giving an explicit exit signal is required.
/// The condition `bytes_read == 0` will never trigger for stdin (would require more input),
/// there is always something in the never-ending stdin stream (except that last newline..)
///
/// # Display Format
/// For each chunk, displays:
/// - Chunk number (sequential counter)
/// - Bytes read (actual count, up to buffer size)
/// - Raw byte array representation
/// - UTF-8 text representation (if valid)
/// - Newline detection marker
///
/// # Edge Cases
/// - Invalid UTF-8: Shows raw bytes, skips text display (uses `unwrap_or("")`)
/// - Multi-line paste without trailing newline: Requires extra Enter press
/// - Empty input followed by Enter: Processes as single newline chunk
/// - Very long input: Automatically chunked into 64-byte segments
/// - Exit command mid-stream: Stops immediately, may leave unprocessed data in stdin
///
/// # Safety Limits
/// - Maximum chunks: Uses usize counter (effectively unlimited on modern systems)
/// - No MAX_CHUNKS limit (unlike file operations) because stdin is user-interactive
/// - User controls termination via exit command
///
/// # Known Issues
///   - TODO: Consider using `unwrap_or_default()` or explicit match for better practice
///
/// # Returns
/// - `Ok(())` on successful completion (user entered exit command)
/// - `Err(io::Error)` if stdin read fails
///
/// # Example
/// ```no_run
/// # use std::io;
/// # fn userinput_to_tui_test() -> io::Result<()> { Ok(()) }
/// // User types: "Hello\nWorld\n-q\n"
/// let result = userinput_to_tui_test();
/// assert!(result.is_ok());
/// // Output shows:
/// // Chunk 1: "Hello\n"
/// // Chunk 2: "World\n"
/// // Chunk 3: "-q\n" (triggers exit)
/// ```
fn userinput_to_tui_test() -> io::Result<()> {
    println!("(Program will read in 2-byte chunks:");
    println!("This is for demonstration/inspection purposes");
    println!("production likely to use >=256 byte)\n");
    println!("Type something and press Enter");
    println!("Type -q or --quit to exit\n");

    let stdin = io::stdin();
    let mut stdin_handle = stdin.lock(); // Lock stdin once for entire session

    const SIZE_OF_BUCKET_BRIGADE_BUFFER: usize = 64;

    let mut main_bucket_brigade_buffer = [0u8; SIZE_OF_BUCKET_BRIGADE_BUFFER]; // optional tip: test with = 2

    let mut chunk_number = 0;
    let mut total_bytes = 0;

    loop {
        // Clear buffer before reading
        for i in 0..SIZE_OF_BUCKET_BRIGADE_BUFFER {
            main_bucket_brigade_buffer[i] = 0;
        }

        chunk_number += 1;

        let bytes_read = stdin_handle.read(&mut main_bucket_brigade_buffer)?;

        println!("\n\nbytes_read {}", bytes_read);

        // get exit signal
        // get exit signal Parse command from bytes
        let text_input_str =
            std::str::from_utf8(&main_bucket_brigade_buffer[..bytes_read]).unwrap_or(""); // Ignore invalid UTF-8

        // get exit signal trimmed
        let trimmed = text_input_str.trim();

        // get exit signal Check for exit insert mode commands
        if trimmed == "-q" || trimmed == "--quite" {
            break;
        }

        // This will not happen.
        if bytes_read == 0 {
            println!("bytes_read == 0... Why did this happen???");
        }

        total_bytes += bytes_read;

        println!(
            "Chunk {}: read {} bytes: {:?}",
            chunk_number,
            bytes_read,
            &main_bucket_brigade_buffer[..bytes_read]
        );

        // Show as string if valid UTF-8
        if let Ok(s) = std::str::from_utf8(&main_bucket_brigade_buffer[..bytes_read]) {
            println!("  As text: {:?}", s);
        }

        // detect newline
        if main_bucket_brigade_buffer[..bytes_read].contains(&b'\n') {
            println!("\n[Mark: Newline Detected]");
        }
    }

    println!("\nTotal bytes read: {}", total_bytes);
    println!("Total chunks: {}", chunk_number);
    println!("All Done!");
    Ok(())
}

/// Reads user input from stdin in pre-allocated chunks and appends to demo.txt file.
///
/// # Memory Safety
/// - Uses pre-allocated buffer (no heap allocation for input processing)
/// - Never loads entire input or file into memory
/// - Processes input chunk-by-chunk using bucket brigade pattern
///
/// # File Behavior
/// - Appends to existing file (creates if doesn't exist)
/// - File is opened once and kept open for session
/// - Each chunk is written immediately (no buffering entire input)
/// - File is flushed after each write for durability
///
/// # Exit Conditions
/// - User types "-q" or "--quit" on a line to exit
/// - Returns error if file operations fail
///
/// # Edge Cases
/// - Handles multi-line input (preserves all newlines in input)
/// - Handles "leftover pizza problem" - user must press Enter after paste
/// - Invalid UTF-8 bytes are written as-is (binary safe)
/// - File write errors are propagated immediately
///
/// # Returns
/// - `Ok(())` on successful completion
/// - `Err(io::Error)` if file cannot be created/opened/written
///
/// # Example
/// ```no_run
/// # use std::io;
/// # fn userinput_to_file_test() -> io::Result<()> { Ok(()) }
/// let result = userinput_to_file_test();
/// assert!(result.is_ok());
/// ```
fn userinput_to_file_test() -> io::Result<()> {
    // use std::env;
    // use std::fs::OpenOptions;
    // use std::io::Write;

    println!("(Program will read in chunks and append to demo.txt)");
    println!("Type something and press Enter.");
    println!("Type -q or --quit to exit and save.\n");

    // Get absolute path to demo.txt in current working directory
    let current_dir = env::current_dir()?;
    let file_path = current_dir.join("demo.txt");

    println!("Writing to: {}", file_path.display());

    // Open file in append mode (create if doesn't exist)
    // File handle held for entire session to avoid repeated open/close overhead
    let mut file = OpenOptions::new()
        .create(true) // Create file if it doesn't exist
        .append(true) // Append to existing content
        .open(&file_path)?;

    let stdin = io::stdin();
    let mut stdin_handle = stdin.lock(); // Lock stdin once for entire session

    // Pre-allocated buffer for bucket brigade processing
    const SIZE_OF_BUCKET_BRIGADE_BUFFER: usize = 64;
    let mut main_bucket_brigade_buffer = [0u8; SIZE_OF_BUCKET_BRIGADE_BUFFER];

    // Counters for diagnostic feedback
    let mut chunk_number = 0;
    let mut total_bytes_written = 0;

    // Safety: Maximum iterations to prevent infinite loop (e.g., from cosmic ray)
    // Allows 1GB of input at 64-byte chunks = ~16 million chunks
    const MAX_CHUNKS: usize = 16_777_216;

    loop {
        // Defensive: prevent infinite loop
        if chunk_number >= MAX_CHUNKS {
            eprintln!(
                "ERROR: Maximum chunk limit reached ({}). Exiting for safety.",
                MAX_CHUNKS
            );
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Maximum iteration limit exceeded",
            ));
        }

        // Clear buffer before reading (defensive: prevent data leakage between reads)
        for i in 0..SIZE_OF_BUCKET_BRIGADE_BUFFER {
            main_bucket_brigade_buffer[i] = 0;
        }

        chunk_number += 1;

        // Read next chunk from stdin
        let bytes_read = stdin_handle.read(&mut main_bucket_brigade_buffer)?;

        // Defensive assertion: bytes_read should never exceed buffer size
        assert!(
            bytes_read <= SIZE_OF_BUCKET_BRIGADE_BUFFER,
            "bytes_read ({}) exceeded buffer size ({})",
            bytes_read,
            SIZE_OF_BUCKET_BRIGADE_BUFFER
        );

        // Check for exit command before writing to file
        // Only check if we have valid UTF-8 (don't fail on binary data)
        if let Ok(text_input_str) = std::str::from_utf8(&main_bucket_brigade_buffer[..bytes_read]) {
            let trimmed = text_input_str.trim();

            // Exit commands: -q or --quit
            if trimmed == "-q" || trimmed == "--quit" {
                println!("\nExit command received. Finalizing file...");
                break;
            }
        }

        // Write chunk directly to file (never buffer entire input in memory)
        let bytes_written = file.write(&main_bucket_brigade_buffer[..bytes_read])?;

        // Defensive assertion: all bytes should be written
        assert_eq!(
            bytes_written, bytes_read,
            "File write incomplete: wrote {} of {} bytes",
            bytes_written, bytes_read
        );

        // Flush to disk immediately for durability (survive crashes/power loss)
        file.flush()?;

        total_bytes_written += bytes_written;

        println!(
            "Chunk {}: wrote {} bytes (total: {})",
            chunk_number, bytes_written, total_bytes_written
        );

        // Optional diagnostic: show content if valid UTF-8
        if let Ok(s) = std::str::from_utf8(&main_bucket_brigade_buffer[..bytes_read]) {
            println!("  Content: {:?}", s);
        }
    }

    // Final flush to ensure all data is on disk
    file.flush()?;

    println!("\n=== Summary ===");
    println!("Total bytes written: {}", total_bytes_written);
    println!("Total chunks: {}", chunk_number);
    println!("File location: {}", file_path.display());
    println!("All Done!");

    Ok(())
}

/// Reads a file in pre-allocated chunks and displays content to terminal.
///
/// # Memory Safety
/// - Uses pre-allocated 64-byte buffer (no heap allocation)
/// - Never loads entire file into memory
/// - Processes file chunk-by-chunk using bucket brigade pattern
///
/// # File Behavior
/// - Opens demo.txt from current working directory in read-only mode
/// - Reads sequentially from start to EOF
/// - Closes file automatically when function exits
///
/// # Display Behavior
/// - Shows each chunk with diagnostics (chunk number, byte count)
/// - Attempts UTF-8 conversion for text display
/// - Shows hex representation for invalid UTF-8 sequences
/// - UTF-8 multi-byte characters may be split across chunks (shown as hex at boundaries)
///
/// # Edge Cases
/// - Empty file: displays "File is empty" message
/// - File not found: returns io::Error
/// - Binary data: displayed as hex bytes
/// - UTF-8 boundary splits: partial characters shown as hex
/// - Very large files: chunked processing prevents memory exhaustion
///
/// # Safety Limits
/// - Maximum chunks: 16,777,216 (allows ~1GB at 64-byte chunks)
/// - Prevents infinite loops from filesystem corruption or cosmic ray errors
///
/// # Returns
/// - `Ok(())` on successful read and display
/// - `Err(io::Error)` if file cannot be opened or read fails
///
/// # Example
/// ```no_run
/// let current_dir = env::current_dir()?;
/// let file_path = current_dir.join("demo.txt");
/// let result_tui = file_to_tui_test(file_path);
/// println!("result_tui -> {:?}", result_tui);
/// assert!(result_tui.is_ok());
/// ```
fn file_to_tui_test(file_path: PathBuf) -> io::Result<()> {
    // use std::env;
    // use std::fs::File;

    println!("=== File to Terminal Display ===\n");

    println!("Reading from: {}\n", file_path.display());

    // Defensive: Check file exists before attempting to open
    if !file_path.exists() {
        eprintln!("ERROR: File does not exist: {}", file_path.display());
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File not found: {}", file_path.display()),
        ));
    }

    // Open file in read-only mode
    let mut file = File::open(&file_path)?;

    // Pre-allocated buffer for bucket brigade processing
    const SIZE_OF_BUCKET_BRIGADE_BUFFER: usize = 64;
    let mut main_bucket_brigade_buffer = [0u8; SIZE_OF_BUCKET_BRIGADE_BUFFER];

    // Counters for diagnostic feedback
    let mut chunk_number = 0;
    let mut total_bytes_read = 0;

    // Safety: Maximum iterations to prevent infinite loop
    // Allows 1GB of file at 64-byte chunks = ~16 million chunks
    const MAX_CHUNKS: usize = 16_777_216;

    loop {
        // Defensive: prevent infinite loop from filesystem corruption or cosmic ray
        if chunk_number >= MAX_CHUNKS {
            eprintln!(
                "ERROR: Maximum chunk limit reached ({}). Exiting for safety.",
                MAX_CHUNKS
            );
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Maximum iteration limit exceeded",
            ));
        }

        // Clear buffer before reading (defensive: prevent data leakage between reads)
        for i in 0..SIZE_OF_BUCKET_BRIGADE_BUFFER {
            main_bucket_brigade_buffer[i] = 0;
        }

        chunk_number += 1;

        // Read next chunk from file
        let bytes_read = file.read(&mut main_bucket_brigade_buffer)?;

        // Defensive assertion: bytes_read should never exceed buffer size
        assert!(
            bytes_read <= SIZE_OF_BUCKET_BRIGADE_BUFFER,
            "bytes_read ({}) exceeded buffer size ({})",
            bytes_read,
            SIZE_OF_BUCKET_BRIGADE_BUFFER
        );

        // EOF detection: Unlike stdin, bytes_read == 0 reliably signals end of file
        if bytes_read == 0 {
            println!("[End of file reached]");
            break;
        }

        total_bytes_read += bytes_read;

        println!("--- Chunk {}: read {} bytes ---", chunk_number, bytes_read);

        // Attempt UTF-8 conversion for text display
        match std::str::from_utf8(&main_bucket_brigade_buffer[..bytes_read]) {
            Ok(text) => {
                // Valid UTF-8: display as text
                println!("  As text: {:?}", text);

                // Optional: show actual text without quotes for readability
                println!("  Display:");
                print!("{}", text);

                // Add newline if chunk doesn't end with one (for clean formatting)
                if !text.ends_with('\n') {
                    println!();
                }
            }
            Err(e) => {
                // Invalid UTF-8: show what we can + hex for invalid bytes
                println!("  [UTF-8 decode error at byte {}: {}]", e.valid_up_to(), e);

                // Show valid UTF-8 portion if any
                let valid_portion = &main_bucket_brigade_buffer[..e.valid_up_to()];
                if !valid_portion.is_empty() {
                    if let Ok(valid_text) = std::str::from_utf8(valid_portion) {
                        println!("  Valid text portion: {:?}", valid_text);
                    }
                }

                // Show problematic bytes as hex
                let invalid_portion = &main_bucket_brigade_buffer[e.valid_up_to()..bytes_read];
                println!(
                    "  Invalid/incomplete bytes as hex: {:02X?}",
                    invalid_portion
                );

                // Show all bytes as hex for complete picture
                println!(
                    "  Full chunk as hex: {:02X?}",
                    &main_bucket_brigade_buffer[..bytes_read]
                );
            }
        }

        println!(); // Blank line between chunks for readability
    }

    // Handle empty file case
    if total_bytes_read == 0 {
        println!("File is empty (0 bytes).");
    }

    println!("=== Summary ===");
    println!("Total bytes read: {}", total_bytes_read);
    println!("Total chunks: {}", chunk_number - 1); // Subtract 1 because last iteration hit EOF
    println!("File location: {}", file_path.display());
    println!("All Done!");

    Ok(())
}

/// Reads a file in pre-allocated chunks and appends content to another file.
///
/// # Memory Safety
/// - Uses pre-allocated 64-byte buffer (no heap allocation)
/// - Never loads entire source or destination file into memory
/// - Processes file chunk-by-chunk using bucket brigade pattern
///
/// # File Behavior
/// - Opens source file in read-only mode
/// - Opens destination file in append mode (creates if doesn't exist)
/// - Reads sequentially from source start to EOF
/// - Appends each chunk immediately to destination (no buffering)
/// - Flushes after each write for durability
/// - Both files closed automatically when function exits
///
/// # Use Cases
/// - Combining log files
/// - Appending data without loading entire files
/// - Safe file concatenation for large files
/// - Incremental backups
///
/// # Edge Cases
/// - Empty source file: creates/touches destination, writes 0 bytes (valid operation)
/// - Source file not found: returns io::Error
/// - Destination doesn't exist: creates new file
/// - Destination exists: appends to end (preserves existing content)
/// - Source and destination are same file: allowed but NOT RECOMMENDED (will double content)
/// - Very large files: chunked processing prevents memory exhaustion
/// - Filesystem full: returns io::Error on write
///
/// # Safety Limits
/// - Maximum chunks: 16,777,216 (allows ~1GB at 64-byte chunks)
/// - Prevents infinite loops from filesystem corruption or cosmic ray errors
///
/// # Parameters
/// - `from_path`: Absolute path to source file to read from
/// - `to_path`: Absolute path to destination file to append to
///
/// # Returns
/// - `Ok(())` on successful copy
/// - `Err(io::Error)` if source cannot be opened, destination cannot be written, or read/write fails
///
/// # Example
/// ```no_run
/// # use std::io;
/// # use std::path::PathBuf;
/// # use std::env;
/// # fn file_append_to_file(from_path: PathBuf, to_path: PathBuf) -> io::Result<()> { Ok(()) }
/// let current_dir = env::current_dir()?;
/// let source_path = current_dir.join("demo.txt");
/// let dest_path = current_dir.join("append_to_this.txt");
/// let result = file_append_to_file(source_path, dest_path);
/// assert!(result.is_ok());
/// # Ok::<(), io::Error>(())
/// ```
fn file_append_to_file(from_path: PathBuf, to_path: PathBuf) -> io::Result<()> {
    // use std::fs::{File, OpenOptions};
    // use std::io::Write;

    println!("=== File to File Append ===\n");
    println!("Reading from: {}", from_path.display());
    println!("Appending to: {}\n", to_path.display());

    // Defensive: Check source file exists before attempting operations
    if !from_path.exists() {
        eprintln!("ERROR: Source file does not exist: {}", from_path.display());
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Source file not found: {}", from_path.display()),
        ));
    }

    // Defensive: Warn if source and destination are the same file
    // (This is allowed but dangerous - will double the file content)
    if from_path == to_path {
        eprintln!("WARNING: Source and destination are the same file!");
        eprintln!("This will append file to itself, doubling its content.");
        eprintln!("Proceeding anyway, but this is likely not intended.\n");
    }

    // Open source file in read-only mode
    let mut source_file = File::open(&from_path)?;

    // Open destination file in append mode (create if doesn't exist)
    let mut dest_file = OpenOptions::new()
        .create(true) // Create file if it doesn't exist
        .append(true) // Append to existing content
        .open(&to_path)?;

    // Pre-allocated buffer for bucket brigade processing
    const SIZE_OF_BUCKET_BRIGADE_BUFFER: usize = 64;
    let mut main_bucket_brigade_buffer = [0u8; SIZE_OF_BUCKET_BRIGADE_BUFFER];

    // Counters for diagnostic feedback
    let mut chunk_number = 0;
    let mut total_bytes_processed = 0;

    // Safety: Maximum iterations to prevent infinite loop
    // Allows 1GB of file at 64-byte chunks = ~16 million chunks
    const MAX_CHUNKS: usize = 16_777_216;

    loop {
        // Defensive: prevent infinite loop from filesystem corruption or cosmic ray
        if chunk_number >= MAX_CHUNKS {
            eprintln!(
                "ERROR: Maximum chunk limit reached ({}). Exiting for safety.",
                MAX_CHUNKS
            );
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Maximum iteration limit exceeded",
            ));
        }

        // Clear buffer before reading (defensive: prevent data leakage between reads)
        for i in 0..SIZE_OF_BUCKET_BRIGADE_BUFFER {
            main_bucket_brigade_buffer[i] = 0;
        }

        chunk_number += 1;

        // Read next chunk from source file
        let bytes_read = source_file.read(&mut main_bucket_brigade_buffer)?;

        // Defensive assertion: bytes_read should never exceed buffer size
        assert!(
            bytes_read <= SIZE_OF_BUCKET_BRIGADE_BUFFER,
            "bytes_read ({}) exceeded buffer size ({})",
            bytes_read,
            SIZE_OF_BUCKET_BRIGADE_BUFFER
        );

        // EOF detection: bytes_read == 0 reliably signals end of source file
        if bytes_read == 0 {
            println!("[End of source file reached]");
            break;
        }

        // Write chunk to destination file (never buffer entire content in memory)
        let bytes_written = dest_file.write(&main_bucket_brigade_buffer[..bytes_read])?;

        // Defensive assertion: all bytes should be written
        assert_eq!(
            bytes_written, bytes_read,
            "Destination write incomplete: wrote {} of {} bytes",
            bytes_written, bytes_read
        );

        // Flush to disk immediately for durability (survive crashes/power loss)
        dest_file.flush()?;

        total_bytes_processed += bytes_written;

        println!(
            "Chunk {}: copied {} bytes (total: {})",
            chunk_number, bytes_written, total_bytes_processed
        );

        // Optional diagnostic: show content if valid UTF-8
        if let Ok(s) = std::str::from_utf8(&main_bucket_brigade_buffer[..bytes_read]) {
            println!("  Content: {:?}", s);
        }
    }

    // Final flush to ensure all data is on disk
    dest_file.flush()?;

    // Handle empty file case
    if total_bytes_processed == 0 {
        println!("Source file was empty (0 bytes copied).");
    }

    println!("\n=== Summary ===");
    println!("Total bytes copied: {}", total_bytes_processed);
    println!("Total chunks: {}", chunk_number - 1); // Subtract 1 because last iteration hit EOF
    println!("Source: {}", from_path.display());
    println!("Destination: {}", to_path.display());
    println!("All Done!");

    Ok(())
}

// Four Tests
fn main() -> io::Result<()> {
    // Test 1: stdin to TUI
    let result_tui = userinput_to_tui_test();
    println!("result_tui -> {:?}", result_tui);

    // Test 2: stdin to file
    let result_tui = userinput_to_file_test();
    println!("result_tui -> {:?}", result_tui);

    // Get absolute path to demo.txt in current working directory
    let current_dir = env::current_dir()?;
    let demo_path1 = current_dir.join("demo.txt");

    // Test 3: file to TUI
    let result_tui = file_to_tui_test(demo_path1);
    println!("result_tui -> {:?}", result_tui);

    // Get absolute paths
    let append_dest_path = current_dir.join("append_to_this.txt");
    let demo_path2 = current_dir.join("demo.txt");

    // Test 4: file to file (append)
    let result_file_to_file = file_append_to_file(demo_path2, append_dest_path);
    println!("result_file_to_file -> {:?}", result_file_to_file);

    println!("main() All Done!");
    Ok(())
}

/*
sample output: note, input contains multiple newlines
program does not finish, asks for more user input


$ cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running `target/debug/bucket_brigade_stdin_poc`
(Program will read in 2-byte chunks:
This is for demonstration/inspection purposes
production likely to use >=256 byte)

Type something and press Enter:


aa
bbb
cccc


bytes_read 2
Chunk 1: read 2 bytes: [97, 10]
  As text: "a\n"

[Mark: Newline Detected]


bytes_read 2
Chunk 2: read 2 bytes: [98, 98]
  As text: "bb"


bytes_read 1
Chunk 3: read 1 bytes: [10]
  As text: "\n"

[Mark: Newline Detected]


bytes_read 2
Chunk 4: read 2 bytes: [99, 99]
  As text: "cc"


bytes_read 2
Chunk 5: read 2 bytes: [99, 10]
  As text: "c\n"

[Mark: Newline Detected]

*/

/*
oops@oops-Precision-7780:~/code/bucket_brigade_stdin_poc$ cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running `target/debug/bucket_brigade_stdin_poc`
(Program will read in 2-byte chunks:
This is for demonstration/inspection purposes
production likely to use >=256 byte)

Type something and press Enter:

        if state.mode == EditorMode::Insert {
            /* For another command area, also see:
            fn parsed_commands(){
            if current_mode == ... */

            // Clear buffer before reading
            for i in 0..TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE {
                text_buffer[i] = 0;
            }

            // Read single command (no chunking)
            let bytes_read = stdin_handle.read(&mut text_buffer)?;

            if bytes_read == 0 {
                continue;
            }

bytes_read 1
Chunk 1: read 1 bytes: [10]
  As text: "\n"

[Mark: Newline Detected]


bytes_read 46
Chunk 2: read 46 bytes: [32, 32, 32, 32, 32, 32, 32, 32, 105, 102, 32, 115, 116, 97, 116, 101, 46, 109, 111, 100, 101, 32, 61, 61, 32, 69, 100, 105, 116, 111, 114, 77, 111, 100, 101, 58, 58, 73, 110, 115, 101, 114, 116, 32, 123, 10]
  As text: "        if state.mode == EditorMode::Insert {\n"

[Mark: Newline Detected]


bytes_read 51
Chunk 3: read 51 bytes: [32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 47, 42, 32, 70, 111, 114, 32, 97, 110, 111, 116, 104, 101, 114, 32, 99, 111, 109, 109, 97, 110, 100, 32, 97, 114, 101, 97, 44, 32, 97, 108, 115, 111, 32, 115, 101, 101, 58, 10]
  As text: "            /* For another command area, also see:\n"

[Mark: Newline Detected]


bytes_read 34
Chunk 4: read 34 bytes: [32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 102, 110, 32, 112, 97, 114, 115, 101, 100, 95, 99, 111, 109, 109, 97, 110, 100, 115, 40, 41, 123, 10]
  As text: "            fn parsed_commands(){\n"

[Mark: Newline Detected]


bytes_read 38
Chunk 5: read 38 bytes: [32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 105, 102, 32, 99, 117, 114, 114, 101, 110, 116, 95, 109, 111, 100, 101, 32, 61, 61, 32, 46, 46, 46, 32, 42, 47, 10]
  As text: "            if current_mode == ... */
\n"

[Mark: Newline Detected]


bytes_read 1
Chunk 6: read 1 bytes: [10]
  As text: "\n"

[Mark: Newline Detected]


bytes_read 43
Chunk 7: read 43 bytes: [32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 47, 47, 32, 67, 108, 101, 97, 114, 32, 98, 117, 102, 102, 101, 114, 32, 98, 101, 102, 111, 114, 101, 32, 114, 101, 97, 100, 105, 110, 103, 10]
As text: " // Clear buffer before reading\n"

[Mark: Newline Detected]


bytes_read 64
Chunk 8: read 64 bytes: [32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 102, 111, 114, 32, 105, 32, 105, 110, 32, 48, 46, 46, 84, 69, 88, 84, 95, 66, 85, 67, 75, 69, 84, 95, 66, 82, 73, 71, 65, 68, 69, 95, 67, 72, 85, 78, 75, 73, 78, 71, 95, 66, 85, 70, 70, 69, 82, 95, 83, 73, 90, 69]
  As text: "            for i in 0..TEXT_BUCKET_BRIGADE_CHUNKING_BUFFER_SIZE"


bytes_read 3
Chunk 9: read 3 bytes: [32, 123, 10]
  As text: " {\n"

[Mark: Newline Detected]


bytes_read 36
Chunk 10: read 36 bytes: [32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 116, 101, 120, 116, 95, 98, 117, 102, 102, 101, 114, 91, 105, 93, 32, 61, 32, 48, 59, 10]
  As text: "                text_buffer[i] = 0;\n"

[Mark: Newline Detected]


bytes_read 14
Chunk 11: read 14 bytes: [32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 125, 10]
  As text: "            }\n"

[Mark: Newline Detected]


bytes_read 1
Chunk 12: read 1 bytes: [10]
  As text: "\n"

[Mark: Newline Detected]


bytes_read 49
Chunk 13: read 49 bytes: [32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 47, 47, 32, 82, 101, 97, 100, 32, 115, 105, 110, 103, 108, 101, 32, 99, 111, 109, 109, 97, 110, 100, 32, 40, 110, 111, 32, 99, 104, 117, 110, 107, 105, 110, 103, 41, 10]
  As text: "            // Read single command (no chunking)\n"

[Mark: Newline Detected]


bytes_read 64
Chunk 14: read 64 bytes: [32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 108, 101, 116, 32, 98, 121, 116, 101, 115, 95, 114, 101, 97, 100, 32, 61, 32, 115, 116, 100, 105, 110, 95, 104, 97, 110, 100, 108, 101, 46, 114, 101, 97, 100, 40, 38, 109, 117, 116, 32, 116, 101, 120, 116, 95, 98, 117, 102, 102, 101, 114, 41]
  As text: "            let bytes_read = stdin_handle.read(&mut text_buffer)"


bytes_read 3
Chunk 15: read 3 bytes: [63, 59, 10]
  As text: "?;\n"

[Mark: Newline Detected]


bytes_read 1
Chunk 16: read 1 bytes: [10]
  As text: "\n"

[Mark: Newline Detected]


bytes_read 33
Chunk 17: read 33 bytes: [32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 105, 102, 32, 98, 121, 116, 101, 115, 95, 114, 101, 97, 100, 32, 61, 61, 32, 48, 32, 123, 10]
  As text: "            if bytes_read == 0 {\n"

[Mark: Newline Detected]


bytes_read 26
Chunk 18: read 26 bytes: [32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 99, 111, 110, 116, 105, 110, 117, 101, 59, 10]
  As text: "                continue;\n"

[Mark: Newline Detected]
-q


bytes_read 16
Chunk 19: read 16 bytes: [32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32, 125, 45, 113, 10]
  As text: "            }-q\n"

[Mark: Newline Detected]
-q


bytes_read 3

Total bytes read: 524
Total chunks: 20
All Done!
result_tui -> Ok(())
main() All Done!

 */
