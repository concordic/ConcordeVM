use libffi::{
    middle::Type,
    raw::{ffi_call, ffi_cif, ffi_prep_cif, ffi_status_FFI_OK, ffi_type},
};
use libloading;
use std::{collections::HashMap, fmt::UpperHex};

pub unsafe fn generic_ffi_call(
    func_ptr: *const (),
    arg_types: &mut [*mut ffi_type], // Array of ffi_type pointers
    ret_type: *mut ffi_type,         // Return type ffi_type pointer
    ret_buffer: *mut u8,             // Buffer to store the return value
    input_buffer: &[u8],             // Packed arguments
) {
    let mut cif = ffi_cif::default();

    // 1. Initialize the Call Interface (CIF)
    let status = unsafe {
        ffi_prep_cif(
            &mut cif,
            libffi::raw::ffi_abi_FFI_DEFAULT_ABI,
            arg_types.len() as u32,
            ret_type,
            arg_types.as_mut_ptr(),
        )
    };

    if status != ffi_status_FFI_OK {
        panic!("FFI CIF prep failed");
    }

    // 2. Prepare pointers to the arguments within the input_buffer
    // C expects an array of pointers to the actual data
    let mut arg_values: Vec<*mut core::ffi::c_void> = Vec::with_capacity(arg_types.len());
    let mut offset = 0;

    for &t in arg_types.iter() {
        let size = (unsafe { *t }).size;
        let alignment = (unsafe { *t }).alignment as usize;

        // Align the offset for the next type (C-style padding)
        offset = (offset + alignment - 1) & !(alignment - 1);

        arg_values.push(unsafe { input_buffer.as_ptr().add(offset) } as *mut core::ffi::c_void);
        offset += size;
    }

    // 4. THE CALL
    unsafe {
        ffi_call(
            &mut cif,
            Some(std::mem::transmute(func_ptr)),
            ret_buffer as *mut core::ffi::c_void,
            arg_values.as_mut_ptr(),
        )
    };
}

struct ByteVec(Vec<u8>);

impl UpperHex for ByteVec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in &self.0 {
            write!(f, "{:02X} ", byte)?;
        }
        Ok(())
    }
}

pub struct FFIFunctionInfo {
    key: usize,
    signature: FFIFunctionSignature,
}

impl FFIFunctionInfo {
    pub fn new(key: usize, name: String, arg_types: Vec<Type>, ret_type: Type) -> Self {
        return Self {
            key,
            signature: FFIFunctionSignature {
                name,
                arg_types,
                ret_type,
            },
        };
    }
}

pub struct FFIFunctionSignature {
    name: String,
    arg_types: Vec<Type>,
    ret_type: Type,
}

pub struct FFIFunction {
    name: String,
    arg_types: Vec<Type>,
    ret_type: Type,
    fn_ptr: *const (),
}

pub struct Domain {
    lib: libloading::Library,
    functions: HashMap<usize, FFIFunction>,
}

impl Domain {
    pub unsafe fn new(lib_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let lib = unsafe { libloading::Library::new(lib_path) }?;
        return Ok(Self {
            lib,
            functions: HashMap::new(),
        });
    }

    pub unsafe fn get_ffi_fn(
        &self,
        signature: &FFIFunctionSignature,
    ) -> Result<FFIFunction, Box<dyn std::error::Error>> {
        let fn_ptr = unsafe {
            self.lib
                .get::<*const ()>(signature.name.as_bytes())?
                .to_owned()
                .into_raw()
                .into_raw() as *const ()
        };
        let ffi_fn = FFIFunction {
            name: signature.name.clone(),
            arg_types: signature.arg_types.clone(),
            ret_type: signature.ret_type.clone(),
            fn_ptr,
        };
        return Ok(ffi_fn);
    }

    pub unsafe fn load_fn(
        &mut self,
        fn_id: usize,
        signature: &FFIFunctionSignature,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ffi_fn = unsafe { self.get_ffi_fn(signature)? };
        self.functions.insert(fn_id, ffi_fn);
        Ok(())
    }

    pub unsafe fn call_function(
        &self,
        fn_id: usize,
        args: &[u8],
        return_buf: *mut u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let ffi_fn = self.functions.get(&fn_id).ok_or("Function not found")?;
        let mut arg_types: Vec<*mut ffi_type> = ffi_fn
            .arg_types
            .iter()
            .map(|t: &Type| t.as_raw_ptr())
            .collect();
        let ret_type_ptr = ffi_fn.ret_type.as_raw_ptr();

        unsafe {
            generic_ffi_call(
                ffi_fn.fn_ptr,
                &mut arg_types,
                ret_type_ptr,
                return_buf,
                args,
            );
        }
        return Ok(());
    }
}

pub struct FFIFuncTable {
    domains: HashMap<usize, Domain>,
}

impl FFIFuncTable {
    pub fn new() -> FFIFuncTable {
        return FFIFuncTable {
            domains: HashMap::new(),
        };
    }

    pub unsafe fn add_domain(
        &mut self,
        domain_id: usize,
        so_path: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.domains.contains_key(&domain_id) {
            panic!("Reinserting domain with key {}", domain_id);
        }
        let domain = unsafe { Domain::new(&so_path)? };
        self.domains.insert(domain_id, domain);
        return Ok(());
    }

    pub unsafe fn load_function_from_so(
        &mut self,
        domain_id: usize,
        func: FFIFunctionInfo,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(domain) = self.domains.get_mut(&domain_id) {
            unsafe { domain.load_fn(func.key, &func.signature)? };
        }

        return Ok(());
    }

    pub unsafe fn call_function(
        &self,
        domain_id: usize,
        fn_id: usize,
        args: &[u8],
        return_buf: *mut u8,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(domain) = self.domains.get(&domain_id) {
            unsafe { domain.call_function(fn_id, args, return_buf)? };
        }

        return Ok(());
    }
}

fn str_to_ffi_type(s: &str) -> Type {
    match s {
        "i8" => Type::i8(),
        "i16" => Type::i16(),
        "i32" => Type::i32(),
        "i64" => Type::i64(),
        "u8" => Type::u8(),
        "u16" => Type::u16(),
        "u32" => Type::u32(),
        "u64" => Type::u64(),
        "f32" => Type::f32(),
        "f64" => Type::f64(),
        "void" => Type::void(),
        "*void* " => Type::pointer(),
        "usize" => Type::usize(),

        _ => panic!("Unknown type: {}", s),
    }
}

#[test]
fn test() -> Result<(), Box<dyn std::error::Error>> {
    let ret_type = Type::structure(vec![Type::u16(), Type::u16(), Type::u32()]);
    let fn_sig = FFIFunctionSignature {
        name: "add".to_string(),
        arg_types: vec![Type::u16(), Type::u16()],
        ret_type: ret_type,
    };

    let mut d = FFIFuncTable::new();
    unsafe { d.add_domain(1, "./ffi.so".to_string())? };
    unsafe {
        d.load_function_from_so(
            1,
            FFIFunctionInfo {
                key: 1,
                signature: fn_sig,
            },
        )?
    };

    let mut ret_buffer = ByteVec(vec![0u8; 12]); // Buffer to hold the return struct (3 i16s = 6 bytes)

    let x = (0xFFFFu16).to_ne_bytes();
    let y = (0xFFFFu16).to_ne_bytes();
    let input_buffer = [x.as_slice(), y.as_slice()].concat();

    unsafe {
        d.call_function(1, 1, &input_buffer, ret_buffer.0.as_mut_ptr())?;
    }

    print!("{:02X}\n", ret_buffer);

    Ok(())
}
