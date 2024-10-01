# Bugi Wasm Plugin Spec V0

This section describes the ABI of the plugin and the functions exported by the plugin.

## ABIs
### System ABI
Here we describe the System ABI. This ABI currently uses the stable Message Pack. It is planned to eventually switch to rkyv or bitcode, but the timing is undecided.  
In the System ABI, all arguments are converted into a single array type, which is then converted into a pointer and byte length and passed to the function.
Additionally, the return value itself needs to be serialized in Message Pack format.  

Example:  
`fn some_func(some_string: String, some_int: i32): SomeDataType`  
-> `fn some_func(arg: [some_string=String, some_int=i32]): SomeDataType`  
-> `fn some_func(arg_ptr: u32, arg_len: u32): (ptr=u32, len=u32)`  

`fn func2(some_str: String): ()`  
-> `fn func2(some_str_ptr: u32, some_str_len: u32): ()`

This is currently used in the `__bugi_v0_provide_desc` function, but it can also be used in regular functions.

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
* `fn __bugi_v0_called_func_<func_name>(args: AnyArray): Any`: call function in plugin

### Importable Function
#### Use ABI
* `fn "bugi@v0" call_univ_func(id: string, name: string, args: AnyArray): Any`: call function in host universe
* 