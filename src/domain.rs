use libloading::{Library, Symbol};
use libffi::middle::{Cif, Type, CodePtr, Arg};
use std::collections::HashMap;

pub struct DomainFunction {
    pub cif: Cif,
    pub fn_ptr: CodePtr,
}

pub struct Domain {
    pub library: Library,
    pub functions: HashMap<String, DomainFunction>,
}

impl Domain {
    pub fn new(path: &str) -> Self {
        Domain {
            library: unsafe { Library::new(path).unwrap() },
            functions: HashMap::new(),
        }
    }

    pub fn add_function(&mut self, fn_name: String, type_signature: Vec<String>) -> Void {

        // First element is return type, rest are argument types
        let return_type = str_to_ffi_type(&type_signature[0]);

        let arg_types: Vec<Type> = type_signature[1..]
            .iter()
            .map(|s| str_to_ffi_type(s))
            .collect();

        // Get function pointer
        let func_ptr: *const () = unsafe {
            *self.library
                .get::<*const ()>(fn_name.as_bytes())
                .expect("Failed to load function")
        };

        let cif = Cif::new(arg_types, return_type);

        self.functions.insert(fn_name, DomainFunction {
            cif,
            fn_ptr: CodePtr::from_ptr(func_ptr as *const _),
        });
    }

    pub unsafe fn call_function<T>(&self, fn_name: &str, args: &[Arg]) -> T {
        let func_info = self.functions
            .get(fn_name)
            .expect("Function not found");

        func_info.cif.call(func_info.fn_ptr, args)
    }
}

fn str_to_ffi_type(s: &str) -> Type {
    match s {
        "i32" => Type::i32(),
        "i64" => Type::i64(),
        "f32" => Type::f32(),
        "f64" => Type::f64(),
        "void" => Type::void(),
        _ => panic!("Unknown type: {}", s),
    }
}
