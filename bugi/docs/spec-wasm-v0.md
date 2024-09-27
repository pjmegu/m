# Bigi Wasm Plugin Spec V0

## ABIs
### Function Args
Function arguments must be serialized to MessagePack format, except for WebAssembly primitive types, and must be called in (pointer: u32,byte_length: u32) format.  

Example:  
`fn some_func(some_string: String, some_int: i32): SomeDataType`  
-> `fn some_func(arg: {some_string: String, some_int:i32}): SomeDataType`  
-> `fn some_func(arg_ptr: u32, arg_len: u32): (ptr=u32, len=u32)`  

`fn func2(some_str: String): ()`  
-> `fn func2(some_str_ptr: u32, some_str_len: u32): ()`


## Types

### `Desc` type
```jsonc
{
    // plugin identifer
    "string_id": `type string`,
}
```
### `Any` type
```jsonc
`type any (e.g. object, array...)`
```
### `AnyArray` type
```jsonc
[
    ...`type Any`
]
```
### `FuncDesc` type
```jsonc
{
    "cacheable": `type bool`,
}
```

## On Plugin

### Expect Exported Function

#### Variable
- `<func_name>`: exported plugin func name

#### Functions(not Use ABI)
* `fn __bugi_v0_low_mem_malloc(len: i32): ptr=i32`: allocate memory for pass args
* `fn __bugi_v0_low_mem_free(ptr: i32, len: i32): ()`: free memory

#### Functions(Use ABI)
* `fn __bugi_v0_provide_desc(): Desc`: provide description (e.g. stringID)
* `fn __bugi_v0_provide_func_desc_<func_name>(): FuncDesc`: provide func description
* `fn __bugi_v0_called_func_<func_name>(args: AnyArray): Any`: call function in plugin

### Importable Function
#### Use ABI
* `fn "bugi@v0" call_univ_func(id: string, name: string, args: AnyArray): Any`: call function in host universe
* 