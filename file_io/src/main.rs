// main.rs

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

fn main() -> io::Result<()> {
    // Create directory to store our files if it doesn't exist
    let dir_path = "data";
    if !Path::new(dir_path).exists() {
        fs::create_dir(dir_path)?;
        println!("Created directory: {}", dir_path);
    }

    // Basic file creation and writing
    let file_path = format!("{}/sample.txt", dir_path);
    basic_write(&file_path, "Hello, Rust file I/O!")?;
    
    // Read file content
    let content = basic_read(&file_path)?;
    println!("File content: {}", content);
    
    // Append to file
    append_to_file(&file_path, "\nThis line was appended.")?;
    
    // Read again to see appended content
    let updated_content = basic_read(&file_path)?;
    println!("Updated file content:\n{}", updated_content);
    
    // Write and read binary data
    let bin_path = format!("{}/binary.dat", dir_path);
    let bytes = [0, 1, 2, 3, 4, 5];
    binary_write(&bin_path, &bytes)?;
    let read_bytes = binary_read(&bin_path)?;
    println!("Binary data: {:?}", read_bytes);
    
    // Reading directory contents
    list_directory(dir_path)?;
    
    // Copy a file
    let copy_path = format!("{}/sample_copy.txt", dir_path);
    fs::copy(&file_path, &copy_path)?;
    println!("Copied {} to {}", file_path, copy_path);
    
    // Reading file metadata
    display_file_metadata(&file_path)?;
    
    println!("All operations completed successfully!");
    Ok(())
}

// Basic file write operation
fn basic_write(path: &str, content: &str) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(content.as_bytes())?;
    println!("Wrote to file: {}", path);
    Ok(())
}

// Basic file read operation
fn basic_read(path: &str) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

// Append to an existing file
fn append_to_file(path: &str, content: &str) -> io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(path)?;
    file.write_all(content.as_bytes())?;
    println!("Appended to file: {}", path);
    Ok(())
}

// Write binary data
fn binary_write(path: &str, data: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(data)?;
    println!("Wrote binary data to file: {}", path);
    Ok(())
}

// Read binary data
fn binary_read(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

// List contents of a directory
fn list_directory(path: &str) -> io::Result<()> {
    println!("\nListing contents of directory: {}", path);
    let entries = fs::read_dir(path)?;
    
    for entry in entries {
        let entry = entry?;
        let file_name = entry.file_name();
        let file_type = if entry.file_type()?.is_dir() {
            "Directory"
        } else {
            "File"
        };
        
        println!("{}: {}", file_type, file_name.to_string_lossy());
    }
    
    Ok(())
}

// Display file metadata
fn display_file_metadata(path: &str) -> io::Result<()> {
    let metadata = fs::metadata(path)?;
    
    println!("\nMetadata for file: {}", path);
    println!("Size: {} bytes", metadata.len());
    println!("Is directory: {}", metadata.is_dir());
    println!("Is file: {}", metadata.is_file());
    println!("Permissions: {:?}", metadata.permissions());
    
    if let Ok(modified) = metadata.modified() {
        println!("Modified: {:?}", modified);
    }
    
    Ok(())
}