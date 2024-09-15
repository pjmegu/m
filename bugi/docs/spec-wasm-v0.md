# Bigi Wasm Plugin Spec V0

## ABIs
### Function Args
Function arguments must be serialized to MessagePack format, except for WebAssembly primitive types, and must be called in (pointer: i32,byte length: i32) format.  

Example:  
`fn some_func(some_string: String, some_int: i32): SomeDataType`  
-> `fn some_func(some_string_byte: i32, some_string_len: i32, some_int: i32): (i32, i32)`


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

- `<func_name>`: exported plugin func name

* `fn __bugi_v0_provide_desc(): Desc`: provide description (e.g. stringID)
* `fn __bugi_v0_provide_func_desc_<func_name>(): FuncDesc`: provide func description
* `fn __bugi_v0_called_func_<func_name>(args: AnyArray): Any`: call function in plugin

### Importable Function
* `fn "bugi@v0" call_univ_func(id: string, name: string, args: AnyArray): Any`: call function in host universe
* 