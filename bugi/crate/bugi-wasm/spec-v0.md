# Bugi-WASM Plugin Spec

## Overview
This document describes the plugin communication specification using WASM Import/Export.

There are two ABIs available in this specification:
1. System ABI -- Communication using standard WASM Types
2. Serialization ABI -- ABI using a specific serialization standard

Additionally, it describes embedding metadata using custom sections.

## Plugin Exports
### Must
#### `bugi@v0_plugin_id`: Custom Section Data
Serialized plugin information using MessagePack is embedded here.

#### `bugi@v0_low_malloc(byte_len: i32): (ok: i32, ptr: i32)`: System ABI Function
Allocates memory for passing arguments.
`byte_len`: The length in bytes to allocate memory

`ok`: 0 indicates success, otherwise it indicates an error occurred during memory allocation

`ptr`: Memory pointer. Use it only after confirming `ok` is 0

#### `bugi@v0_low_free(byte_ptr: i32, byte_len: i32): ok:i32`: System ABI Function
Memory is freed.

`byte_ptr`: Pointer to the start of the memory to be freed

`byte_len`: Length of the memory

`ok`: 0 indicates success, otherwise it indicates an error occurred

### Plugin Functions

#### `bugi@v0_plugin_function_<name>(arg_ptr: i32, arg_len: i32, abi: i32): (ok: i32, result_ptr: i32, result_len: i32)`: Serialization ABI Function
Calls the plugin function.

`arg_ptr`: Pointer to the serialized argument data. It must be allocated by `bugi@v0_low_malloc`. After reading, the memory is automatically discarded.

`arg_len`: Byte length of the argument data.

`abi`: Serialization type of the argument data. An error occurs if it does not match the actual ABI.

`ok`: Whether it was processed successfully. 0 means normal. If an ABI error occurs or the function being executed panics or throws an exception, it will be non-zero.

`result_ptr`: Pointer to the return value. If `ok` is non-zero, the data serialized in Messagepack is placed. After reading, the memory must be freed.

`result_len`: Byte length of the return value.

## Plugin Imports

### `bugi@v0` `call_univ(arg_ptr: i32, arg_len: i32): (ok: i32, result_ptr: i32, result_len: i32)`: Serialization ABI Function
#### ARG Type
```jsonc
{
    "id": "Plugin ID",
    "name": "Function Name",
    "abi": 0, // ABI ID, type:u32
    "detail": [/* Binary Type: Arg Data serialized something format of "abi" id */]
}
```
`arg_ptr`: `ARG Type`'s data(serialized messagepack)

`arg_len`: byte length

