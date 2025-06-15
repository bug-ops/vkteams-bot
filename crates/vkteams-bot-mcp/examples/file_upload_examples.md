# File Upload Examples for VKTeams Bot MCP Server

This document provides comprehensive examples of how to use the enhanced file upload capabilities of the VKTeams Bot MCP Server.

## Table of Contents

- [Basic File Upload](#basic-file-upload)
- [Text to File Conversion](#text-to-file-conversion)
- [JSON File Creation](#json-file-creation)
- [CSV File Generation](#csv-file-generation)
- [Code File Upload](#code-file-upload)
- [Batch File Upload](#batch-file-upload)
- [File Validation](#file-validation)
- [Format Conversion](#format-conversion)

---

## Basic File Upload

### Upload from Base64

Upload any file that has been encoded as base64:

```json
{
  "tool": "upload_file_from_base64",
  "params": {
    "file_name": "document.pdf",
    "base64_content": "JVBERi0xLjQKJdPr6eEKMSAwIG9iag...",
    "caption": "📄 Important document uploaded by AI assistant",
    "reply_msg_id": "12345"
  }
}
```

### Upload Image

```json
{
  "tool": "upload_file_from_base64",
  "params": {
    "file_name": "chart.png",
    "base64_content": "iVBORw0KGgoAAAANSUhEUgAAA...",
    "caption": "📊 Generated chart showing quarterly results"
  }
}
```

---

## Text to File Conversion

### Create Text File

Convert any text content into a downloadable file:

```json
{
  "tool": "upload_text_as_file",
  "params": {
    "file_name": "meeting_notes.txt",
    "text_content": "Meeting Notes - Project Alpha\n\nDate: 2024-01-15\nAttendees: John, Sarah, Mike\n\nAction Items:\n1. Complete API documentation\n2. Set up CI/CD pipeline\n3. Schedule user testing\n\nNext meeting: 2024-01-22",
    "caption": "📝 Meeting notes from today's discussion"
  }
}
```

### Create Configuration File

```json
{
  "tool": "upload_text_as_file",
  "params": {
    "file_name": "config.yaml",
    "text_content": "server:\n  host: localhost\n  port: 8080\n  ssl: true\n\ndatabase:\n  url: postgresql://localhost/mydb\n  pool_size: 10\n\nlogging:\n  level: info\n  file: /var/log/app.log",
    "caption": "⚙️ Application configuration file"
  }
}
```

---

## JSON File Creation

### Create Simple JSON

```json
{
  "tool": "upload_json_file",
  "params": {
    "file_name": "user_data",
    "json_data": "{\"users\": [{\"id\": 1, \"name\": \"John Doe\", \"email\": \"john@example.com\"}, {\"id\": 2, \"name\": \"Jane Smith\", \"email\": \"jane@example.com\"}]}",
    "pretty_print": true,
    "caption": "👥 User database export"
  }
}
```

### Create API Response Mock

```json
{
  "tool": "upload_json_file",
  "params": {
    "file_name": "api_response_mock.json",
    "json_data": "{\"status\": \"success\", \"data\": {\"products\": [{\"id\": \"prod_001\", \"name\": \"Laptop\", \"price\": 999.99, \"in_stock\": true}, {\"id\": \"prod_002\", \"name\": \"Mouse\", \"price\": 29.99, \"in_stock\": false}]}, \"meta\": {\"total\": 2, \"page\": 1, \"per_page\": 10}}",
    "pretty_print": true,
    "caption": "🔌 API response mock for testing"
  }
}
```

---

## CSV File Generation

### Create User Report

```json
{
  "tool": "upload_csv_file",
  "params": {
    "file_name": "user_report",
    "headers": "ID,Name,Email,Registration Date,Status",
    "rows_json": "[[1, \"John Doe\", \"john@example.com\", \"2024-01-01\", \"Active\"], [2, \"Jane Smith\", \"jane@example.com\", \"2024-01-02\", \"Inactive\"], [3, \"Bob Johnson\", \"bob@example.com\", \"2024-01-03\", \"Active\"]]",
    "caption": "📊 Monthly user activity report"
  }
}
```

### Create Sales Data

```json
{
  "tool": "upload_csv_file",
  "params": {
    "file_name": "sales_q1_2024.csv",
    "headers": "Month,Product,Revenue,Units Sold",
    "rows_json": "[[\"January\", \"Laptop\", 15000, 15], [\"January\", \"Mouse\", 2500, 84], [\"February\", \"Laptop\", 18000, 18], [\"February\", \"Mouse\", 2100, 70], [\"March\", \"Laptop\", 21000, 21], [\"March\", \"Mouse\", 2800, 93]]",
    "caption": "💰 Q1 2024 Sales Report"
  }
}
```

---

## Code File Upload

### Upload Python Script

```json
{
  "tool": "upload_code_file",
  "params": {
    "file_name": "data_processor.py",
    "code_content": "#!/usr/bin/env python3\n\nimport pandas as pd\nimport numpy as np\nfrom datetime import datetime\n\ndef process_data(input_file, output_file):\n    \"\"\"\n    Process CSV data and generate summary statistics.\n    \"\"\"\n    # Read data\n    df = pd.read_csv(input_file)\n    \n    # Calculate statistics\n    summary = {\n        'total_records': len(df),\n        'processed_at': datetime.now().isoformat(),\n        'columns': list(df.columns),\n        'statistics': df.describe().to_dict()\n    }\n    \n    # Save results\n    with open(output_file, 'w') as f:\n        json.dump(summary, f, indent=2)\n    \n    print(f\"Processed {len(df)} records\")\n    return summary\n\nif __name__ == '__main__':\n    process_data('input.csv', 'output.json')",
    "description": "Data processing script for CSV analysis",
    "language": "Python"
  }
}
```

### Upload Rust Code

```json
{
  "tool": "upload_code_file",
  "params": {
    "file_name": "server.rs",
    "code_content": "use tokio::net::TcpListener;\nuse std::io::Result;\n\n#[tokio::main]\nasync fn main() -> Result<()> {\n    let listener = TcpListener::bind(\"127.0.0.1:8080\").await?;\n    println!(\"Server running on http://127.0.0.1:8080\");\n    \n    loop {\n        let (socket, addr) = listener.accept().await?;\n        println!(\"New connection from: {}\", addr);\n        \n        tokio::spawn(async move {\n            handle_connection(socket).await;\n        });\n    }\n}\n\nasync fn handle_connection(socket: tokio::net::TcpStream) {\n    // Handle connection logic here\n}",
    "description": "Simple async TCP server implementation"
  }
}
```

---

## Batch File Upload

### Upload Multiple Related Files

```json
{
  "tool": "upload_multiple_files",
  "params": {
    "files_json": "[{\"name\": \"README.md\", \"content\": \"IyBQcm9qZWN0IEFscGhhCgpUaGlzIGlzIGEgc2FtcGxlIHByb2plY3Qu\"}, {\"name\": \"package.json\", \"content\": \"ewogICJuYW1lIjogInByb2plY3QtYWxwaGEiLAogICJ2ZXJzaW9uIjogIjEuMC4wIgp9\"}, {\"name\": \"main.js\", \"content\": \"Y29uc29sZS5sb2coJ0hlbGxvIFdvcmxkIScpOw==\"}]",
    "caption": "📦 Project Alpha - Initial files"
  }
}
```

### Upload Documentation Set

```json
{
  "tool": "upload_multiple_files",
  "params": {
    "files_json": "[{\"name\": \"api_docs.md\", \"content\": \"IyBBUEkgRG9jdW1lbnRhdGlvbgoKIyMgRW5kcG9pbnRzCi0gR0VUIC9hcGkvdXNlcnMKLSBQT1NUIC9hcGkvdXNlcnM=\"}, {\"name\": \"user_guide.md\", \"content\": \"IyBVc2VyIEd1aWRlCgojIyBHZXR0aW5nIFN0YXJ0ZWQKMS4gSW5zdGFsbCB0aGUgYXBwCjIuIENyZWF0ZSBhbiBhY2NvdW50\"}, {\"name\": \"changelog.md\", \"content\": \"IyBDaGFuZ2Vsb2cKCiMjIHYxLjAuMAotIEluaXRpYWwgcmVsZWFzZQ==\"}]",
    "caption": "📚 Complete documentation package"
  }
}
```

---

## File Validation

### Validate Before Upload

```json
{
  "tool": "validate_file_before_upload",
  "params": {
    "file_name": "large_dataset.csv",
    "base64_content": "bmFtZSxhZ2UsY2l0eQpKb2huLDI1LE5ldyBZb3JrCkphbmUsMzAsQ2hpY2Fnbw=="
  }
}
```

**Expected Response:**

```json
{
  "valid": true,
  "filename": "large_dataset.csv",
  "file_size": 42,
  "file_size_formatted": "42 B",
  "mime_type": "text/csv",
  "is_text": true,
  "is_image": false,
  "is_audio": false,
  "errors": [],
  "warnings": [],
  "can_upload": true
}
```

---

## Format Conversion

### Convert to Text

```json
{
  "tool": "upload_with_conversion",
  "params": {
    "file_name": "data.json",
    "base64_content": "eyJuYW1lIjogIkpvaG4iLCAiYWdlIjogMzB9",
    "target_format": "txt",
    "caption": "Converting JSON data to readable text format"
  }
}
```

### Convert to Pretty JSON

```json
{
  "tool": "upload_with_conversion",
  "params": {
    "file_name": "minified.json",
    "base64_content": "eyJuYW1lIjoiSm9obiIsImFnZSI6MzAsImNpdHkiOiJOZXcgWW9yayJ9",
    "target_format": "json",
    "caption": "Reformatting JSON with proper indentation"
  }
}
```

---

## Error Handling Examples

### Invalid Base64

```json
{
  "tool": "upload_file_from_base64",
  "params": {
    "file_name": "test.txt",
    "base64_content": "invalid-base64-content!"
  }
}
```

**Expected Error:**

```json
{
  "error": "Invalid base64 content: Invalid character '!' at position 22"
}
```

### File Too Large

```json
{
  "tool": "validate_file_before_upload",
  "params": {
    "file_name": "huge_file.bin",
    "base64_content": "..."
  }
}
```

**Expected Response:**

```json
{
  "valid": false,
  "errors": ["File too large (15.2 MB)"],
  "can_upload": false
}
```

---

## Best Practices

1. **Always validate files** before uploading to avoid errors
2. **Use descriptive filenames** with proper extensions
3. **Add meaningful captions** to provide context
4. **Check file size limits** before encoding large files
5. **Use batch upload** for related files to maintain organization
6. **Choose appropriate formats** for your data type
7. **Include error handling** in your implementation

---

## Supported File Extensions

### Text Files

- `.txt`, `.md`, `.json`, `.xml`, `.csv`, `.log`
- `.yml`, `.yaml`, `.toml`, `.ini`, `.cfg`

### Code Files

- `.rs`, `.py`, `.js`, `.ts`, `.html`, `.css`
- `.sql`, `.sh`, `.dockerfile`, `.makefile`

### Media Files

- **Images**: `.png`, `.jpg`, `.jpeg`, `.gif`, `.svg`, `.webp`
- **Audio**: `.mp3`, `.wav`, `.ogg`, `.m4a`, `.aac`
- **Video**: `.mp4`, `.avi`, `.mov`, `.mkv`

### Document Files

- `.pdf`, `.doc`, `.docx`, `.xls`, `.xlsx`
- `.ppt`, `.pptx`, `.rtf`, `.odt`

### Archive Files

- `.zip`, `.tar`, `.gz`, `.7z`, `.rar`

---

For more information, see the main [README.md](../README.md) file.
